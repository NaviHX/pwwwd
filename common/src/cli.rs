pub mod server {
    use anyhow::{Result, anyhow};
    use directories::BaseDirs;

    #[derive(clap::Parser)]
    pub struct Args {
        #[command(subcommand)]
        pub load: Load,

        #[command(flatten)]
        pub resize: Resize,

        #[arg(long ,short, value_parser = parse_rgb)]
        pub fill_rgb: Option<(u8, u8, u8)>,
    }

    #[derive(clap::Subcommand)]
    pub enum Load {
        #[command(name = "load")]
        FromPath { path: String },
        Restore { path: Option<String> },
    }

    pub fn default_restore_path() -> Result<String> {
        let dirs = BaseDirs::new().ok_or(anyhow!(
            "Cannot create `BaseDirs` to get default restore path"
        ))?;

        let dir = dirs
            .state_dir()
            .ok_or(anyhow!("Cannot find XDG state dir"))?
            .to_str()
            .ok_or(anyhow!("Cannot create str from dir"))?
            .to_string();

        Ok(dir)
    }

    fn parse_rgb(s: &str) -> Result<(u8, u8, u8)> {
        if s.len() != 6 {
            return Err(anyhow!("RGBA must have 8 hex chars"));
        }

        let r = u8::from_str_radix(&s[0..2], 16)?;
        let g = u8::from_str_radix(&s[2..4], 16)?;
        let b = u8::from_str_radix(&s[4..6], 16)?;
        Ok((r, g, b))
    }

    pub const RGB: (u8, u8, u8) = (0x22, 0x44, 0x66);

    #[derive(clap::Args)]
    #[group(required = false, multiple = false)]
    pub struct Resize {
        #[arg(long, short)]
        pub no_resize: bool,

        #[arg(long, short)]
        pub resize: Option<ResizeOption>,
    }

    #[derive(Copy, Clone, clap::ValueEnum)]
    pub enum ResizeOption {
        No,
        Crop,
        Fit,
        Stretch,
    }

    pub const DEFAULT_RESIZE: ResizeOption = ResizeOption::Crop;
}
