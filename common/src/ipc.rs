use crate::cli::{
    client::{
        ClientSubcommand, DEFAULT_EASE_KIND, DEFAULT_TRANSITION_KIND, EaseKind, ResizeOption,
        TransitionKind, TransitionOptions,
    },
    server::DEFAULT_RESIZE,
};
use anyhow::{Result, anyhow};
use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    io::{Read, Write},
    path::PathBuf,
};

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
    pub path: PathBuf,
    pub resize: ResizeOption,
    pub transition: TransitionKind,
    pub transition_options: TransitionOptions,
    pub ease: EaseKind,
}

impl Message {
    pub fn from_cli_command(cli: ClientSubcommand) -> Self {
        match cli {
            ClientSubcommand::SwitchImage {
                image,
                resize,
                transition,
                transition_options,
                ease,
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

                let ease = if ease.no_ease {
                    EaseKind::No
                } else if let Some((px1, py1, px2, py2)) = ease.cubic_curve {
                    EaseKind::CubicBezier(px1, py1, px2, py2)
                } else {
                    ease.ease.unwrap_or(DEFAULT_EASE_KIND)
                };

                Self::Image {
                    args: ImageArgs {
                        path: image,
                        resize,
                        transition,
                        transition_options,
                        ease,
                    },
                }
            }
            ClientSubcommand::Kill => Self::Kill,
            ClientSubcommand::Completion { shell: _ } => {
                panic!("`Completion` is not a valid message to be sent")
            }
        }
    }

    pub fn send<T: Write>(&self, socket: &mut T) -> Result<()> {
        let mut buf = vec![];
        self.serialize(&mut Serializer::new(&mut buf))?;

        let len = buf.len() as u32;
        let len_buf = len.to_be_bytes();

        socket.write_all(&len_buf)?;
        socket.write_all(&buf)?;
        Ok(())
    }

    pub fn receive<T: Read>(socket: &mut T) -> Result<Self> {
        let mut len_buf = [0u8; 4];
        socket.read_exact(&mut len_buf)?;
        let len = u32::from_be_bytes(len_buf) as usize;

        let mut buf = vec![0; len];
        socket.read_exact(&mut buf)?;

        let message = Message::deserialize(&mut Deserializer::from_read_ref(&buf))?;
        Ok(message)
    }

    #[cfg(feature = "async")]
    pub async fn async_send<T: tokio::io::AsyncWriteExt + Unpin>(
        &self,
        socket: &mut T,
    ) -> Result<()> {
        let mut buf = vec![];
        self.serialize(&mut Serializer::new(&mut buf))?;

        let len = buf.len() as u32;
        let len_buf = len.to_be_bytes();

        socket.write_all(&len_buf).await?;
        socket.write_all(&buf).await?;
        Ok(())
    }

    #[cfg(feature = "async")]
    pub async fn async_receive<T: tokio::io::AsyncReadExt + Unpin>(socket: &mut T) -> Result<Self> {
        let mut len_buf = [0u8; 4];
        socket.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf) as usize;

        let mut buf = vec![0; len];
        socket.read_exact(&mut buf).await?;

        let message = Message::deserialize(&mut Deserializer::from_read_ref(&buf))?;
        Ok(message)
    }
}

pub fn default_uds_path() -> Result<PathBuf> {
    let dirs =
        directories::BaseDirs::new().ok_or(anyhow!("Cannot create `BaseDirs` to get uds path"))?;
    let mut dir = dirs
        .runtime_dir()
        .map(|p| p.to_owned())
        .ok_or(anyhow!("Didn't find XDG_RUNTIME_DIR"))?;
    dir.push("pwwwd.sock");
    Ok(dir)
}

/// The daemon's reply type. Following a 4-byte `length` big-endian message in socket stream.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Reply {
    Ok,
    Error(String),
}

impl Reply {
    pub fn send<T: Write>(&self, socket: &mut T) -> Result<()> {
        let mut buf = vec![];
        self.serialize(&mut Serializer::new(&mut buf))?;

        let len = buf.len() as u32;
        let len_buf = len.to_be_bytes();

        socket.write_all(&len_buf)?;
        socket.write_all(&buf)?;
        Ok(())
    }

    pub fn receive<T: Read>(socket: &mut T) -> Result<Self> {
        let mut len_buf = [0u8; 4];
        socket.read_exact(&mut len_buf)?;
        let len = u32::from_be_bytes(len_buf) as usize;

        let mut buf = vec![0; len];
        socket.read_exact(&mut buf)?;

        let reply = Reply::deserialize(&mut Deserializer::from_read_ref(&buf))?;
        Ok(reply)
    }

    #[cfg(feature = "async")]
    pub async fn async_send<T: tokio::io::AsyncWriteExt + Unpin>(
        &self,
        socket: &mut T,
    ) -> Result<()> {
        let mut buf = vec![];
        self.serialize(&mut Serializer::new(&mut buf))?;

        let len = buf.len() as u32;
        let len_buf = len.to_be_bytes();

        socket.write_all(&len_buf).await?;
        socket.write_all(&buf).await?;
        Ok(())
    }

    #[cfg(feature = "async")]
    pub async fn async_receive<T: tokio::io::AsyncReadExt + Unpin>(socket: &mut T) -> Result<Self> {
        let mut len_buf = [0u8; 4];
        socket.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf) as usize;

        let mut buf = vec![0; len];
        socket.read_exact(&mut buf).await?;

        let reply = Reply::deserialize(&mut Deserializer::from_read_ref(&buf))?;
        Ok(reply)
    }

    pub fn from_result<T, E: Display>(result: Result<T, E>) -> Self {
        match result {
            Ok(_) => Self::Ok,
            Err(e) => Self::Error(e.to_string()),
        }
    }
}
