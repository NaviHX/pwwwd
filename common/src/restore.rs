use crate::cli::server::ResizeOption;
use anyhow::{Result, anyhow};
use rmp_serde::Deserializer;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Restore {
    pub file_path: PathBuf,
    pub resize_option: ResizeOption,
}

impl Restore {
    pub fn new(path: impl AsRef<Path>, resize_option: ResizeOption) -> Self {
        Restore {
            file_path: path.as_ref().to_owned(),
            resize_option,
        }
    }

    pub fn deserialize_from_buf(buf: &[u8]) -> Result<Self> {
        let res = Restore::deserialize(&mut Deserializer::from_read_ref(&buf))
            .map_err(|e| anyhow!("Cannot deserialize `Restore`: {e}"))?;
        Ok(res)
    }
}
