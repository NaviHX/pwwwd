use crate::cli::server::ResizeOption;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Restore {
    file_path: PathBuf,
    resize_option: ResizeOption,
}

impl Restore {
    pub fn new(path: impl AsRef<Path>, resize_option: ResizeOption) -> Self {
        Restore {
            file_path: path.as_ref().to_owned(),
            resize_option,
        }
    }
}
