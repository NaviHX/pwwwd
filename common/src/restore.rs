use crate::cli::server::ResizeOption;
use anyhow::{Result, anyhow};
use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Restore {
    pub file_path: PathBuf,
    pub resize_option: ResizeOption,
    pub fill_rgb: (u8, u8, u8),
}

impl Restore {
    pub fn new(
        path: impl AsRef<Path>,
        resize_option: ResizeOption,
        fill_rgb: (u8, u8, u8),
    ) -> Self {
        Restore {
            file_path: path.as_ref().to_owned(),
            resize_option,
            fill_rgb,
        }
    }

    pub fn deserialize_from<R: Read>(reader: R) -> Result<Self> {
        let res = Restore::deserialize(&mut Deserializer::new(reader))
            .map_err(|e| anyhow!("Cannot deserialize `Restore`: {e}"))?;
        Ok(res)
    }

    pub fn serialize_to<W: Write>(&self, writer: W) -> Result<()> {
        self.serialize(&mut Serializer::new(writer))
            .map_err(|e| anyhow!("Cannot serialize `Restore`: {e}"))?;
        Ok(())
    }

    #[cfg(feature = "async")]
    pub async fn async_deserialize_from<R: tokio::io::AsyncReadExt + Unpin>(
        mut reader: R,
    ) -> Result<Self> {
        let mut buf = vec![];
        reader
            .read_to_end(&mut buf)
            .await
            .map_err(|e| anyhow!("Cannot async-read data to be deserialize: {e}"))?;

        let res = Restore::deserialize(&mut Deserializer::from_read_ref(&buf))
            .map_err(|e| anyhow!("Cannot deserialize `Restore`: {e}"))?;

        Ok(res)
    }

    #[cfg(feature = "async")]
    pub async fn async_serialize_to<W: tokio::io::AsyncWriteExt + Unpin>(
        &mut self,
        mut writer: W,
    ) -> Result<()> {
        let mut buf = vec![];
        self.serialize(&mut Serializer::new(&mut buf))
            .map_err(|e| anyhow!("Cannot serialize `Restore`: {e}"))?;

        writer
            .write_all(&buf)
            .await
            .map_err(|e| anyhow!("Cannot write serialized data into writer: {e}"))?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;

    #[test]
    fn test_serde_restore() {
        let restore = super::Restore::new(
            Path::new("/home/test/test.jpg"),
            crate::cli::server::ResizeOption::No,
            (0, 0, 0),
        );

        let mut buf = vec![];
        restore.serialize_to(&mut buf).unwrap();
        let new_restore = super::Restore::deserialize_from(&buf[..]).unwrap();

        assert_eq!(restore.file_path, new_restore.file_path);
        assert_eq!(restore.resize_option, new_restore.resize_option);
        assert_eq!(restore.fill_rgb, new_restore.fill_rgb);
    }
}
