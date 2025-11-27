use crate::cli::{
    client::{
        ClientSubcommand, DEFAULT_TRANSITION_KIND, ResizeOption, TransitionKind, TransitionOptions,
    },
    server::DEFAULT_RESIZE,
};
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::{io::{Read, Write}, os::unix::net::UnixStream, path::PathBuf};
use rmp_serde::{Serializer, Deserializer};

/// The daemon's reply type. Following a 4-byte `length` big-endian message in socket stream.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    Kill,
    Image {
        #[serde(flatten)]
        args: ImageArgs,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImageArgs {
    path: String,
    resize: ResizeOption,
    transition: TransitionKind,
    transition_options: TransitionOptions,
}

impl Message {
    pub fn from_cli_command(cli: ClientSubcommand) -> Self {
        match cli {
            ClientSubcommand::SwitchImage {
                image,
                resize,
                transition,
                transition_options,
            } => {
                let resize = if resize.no_resize {
                    ResizeOption::No
                } else {
                    resize.resize.unwrap_or(DEFAULT_RESIZE)
                };

                let transition = if transition.no_transition {
                    TransitionKind::No
                } else {
                    transition.transition.unwrap_or(DEFAULT_TRANSITION_KIND)
                };

                Self::Image {
                    args: ImageArgs {
                        path: image,
                        resize,
                        transition,
                        transition_options,
                    },
                }
            }
            ClientSubcommand::Kill => Self::Kill,
        }
    }

    pub fn send(&self, socket: &mut UnixStream) -> Result<()> {
        let mut buf = vec![];
        self.serialize(&mut Serializer::new(&mut buf))?;

        let len = buf.len() as u32;
        let len_buf = len.to_be_bytes();

        socket.write_all(&len_buf)?;
        socket.write_all(&buf)?;
        Ok(())
    }

    pub fn receive(socket: &mut UnixStream) -> Result<Self> {
        let mut len_buf = [0u8; 4];
        socket.read_exact(&mut len_buf)?;
        let len = u32::from_be_bytes(len_buf) as usize;

        let mut buf = vec![0; len];
        socket.read_exact(&mut buf)?;

        let message = Message::deserialize(&mut Deserializer::from_read_ref(&buf))?;
        Ok(message)
    }
}

pub fn default_uds_path() -> Result<PathBuf> {
    let dirs =
        directories::BaseDirs::new().ok_or(anyhow!("Cannot create `BaseDirs` to get uds path"))?;
    dirs.runtime_dir()
        .map(|p| p.to_owned())
        .ok_or(anyhow!("Didn't find XDG_RUNTIME_DIR"))
}

/// The daemon's reply type. Following a 4-byte `length` big-endian message in socket stream.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Reply {
    Ok,
    Error(String),
}

impl Reply {
    pub fn send(&self, socket: &mut UnixStream) -> Result<()> {
        let mut buf = vec![];
        self.serialize(&mut Serializer::new(&mut buf))?;

        let len = buf.len() as u32;
        let len_buf = len.to_be_bytes();

        socket.write_all(&len_buf)?;
        socket.write_all(&buf)?;
        Ok(())
    }

    pub fn receive(socket: &mut UnixStream) -> Result<Self> {
        let mut len_buf = [0u8; 4];
        socket.read_exact(&mut len_buf)?;
        let len = u32::from_be_bytes(len_buf) as usize;

        let mut buf = vec![0; len];
        socket.read_exact(&mut buf)?;

        let reply = Reply::deserialize(&mut Deserializer::from_read_ref(&buf))?;
        Ok(reply)
    }
}
