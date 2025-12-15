pub use clap_complete;

fn canonicalize_path(s: &str) -> Result<std::path::PathBuf, String> {
    std::fs::canonicalize(s).map_err(|e| format!("{}: {}", s, e))
}

pub mod server {
    use anyhow::{Result, anyhow};
    use clap_complete::Shell;
    use directories::BaseDirs;
    use std::path::PathBuf;

    #[derive(clap::Parser)]
    #[command(name = "pwwwd")]
    #[command(version = "0.1.0")]
    #[command(about = "Phillips's wgpu-based Wayland wallpaper daemon")]
    pub struct Args {
        /// Which image to load as the first wallpaper since startup
        #[command(subcommand)]
        pub subcommand: ServerSubcommand,
    }

    #[derive(clap::Subcommand)]
    pub enum ServerSubcommand {
        /// Load image from specified path
        #[command(name = "load")]
        FromPath {
            #[arg(value_parser = super::canonicalize_path)]
            path: PathBuf,

            /// How to resize the image
            #[command(flatten)]
            resize: Resize,

            /// Which color to fill the padding with when loaded image does not fill the screen
            #[arg(long ,short, value_parser = parse_rgb)]
            fill_rgb: Option<(u8, u8, u8)>,
        },
        /// Restore last used image
        Restore,

        /// Generate shell completion
        Completion {
            #[arg()]
            shell: Shell,
        },
    }

    /// Get the restore file path. Create parent directory if it doesn't exist.
    pub fn default_restore_path() -> Result<PathBuf> {
        let dirs = BaseDirs::new().ok_or(anyhow!(
            "Cannot create `BaseDirs` to get default restore path"
        ))?;

        let dir = dirs
            .state_dir()
            .ok_or(anyhow!("Cannot find XDG state dir"))?
            .join("pwwwd");

        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
        }

        let restore_file = dir.join("restore-path").to_owned();

        Ok(restore_file)
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
        /// Do not resize the image. Equavalent to `--resize no`
        #[arg(long)]
        pub no_resize: bool,

        /// How to resize image
        #[arg(long)]
        pub resize: Option<ResizeOption>,
    }

    #[derive(
        Copy, Clone, clap::ValueEnum, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq,
    )]
    pub enum ResizeOption {
        /// Do not resize the image
        No,
        /// Resize the image to fill the entire screen, cropping out parts that don't fit. Default
        /// resize option
        Crop,
        /// Resize the image to fit inside the screen, preserving the original aspect ratio
        Fit,
        /// Stretch the image to fit inside the screen, without preserving the original aspect
        /// ratio
        Stretch,
    }

    pub const DEFAULT_RESIZE: ResizeOption = ResizeOption::Crop;

    pub use super::client::{DEFAULT_TRANSITION_DURATION, DEFAULT_TRANSITION_FPS};
}

pub mod client {
    use anyhow::{Result, anyhow};
    use clap_complete::Shell;
    use std::path::PathBuf;

    pub use super::server::{Resize, ResizeOption};

    #[derive(clap::Parser)]
    #[command(name = "pwww")]
    #[command(version = "0.1.0")]
    #[command(about = "CLI controller of pwwwd")]
    pub struct Args {
        #[command(subcommand)]
        pub subcommand: ClientSubcommand,
    }

    #[derive(clap::Subcommand)]
    pub enum ClientSubcommand {
        #[command(name = "img")]
        /// Send a new image path for pwwwd to display
        SwitchImage {
            /// The path of the new image
            #[arg(value_parser = super::canonicalize_path)]
            image: PathBuf,

            /// How to resize the image
            #[command(flatten)]
            resize: Resize,

            /// Set the type of transition
            #[command(flatten)]
            transition: Transition,

            /// Set the options of transition
            #[command(flatten)]
            transition_options: TransitionOptions,

            /// Set the options for easing function of transition
            #[command(flatten)]
            ease: Ease,
        },

        /// Kill pwwwd daemon
        Kill,

        /// Generate shell completion
        Completion {
            #[arg()]
            shell: Shell,
        },
    }

    #[derive(clap::Args)]
    #[group(required = false, multiple = false)]
    pub struct Transition {
        /// Switch to the new image immediately
        #[arg(long)]
        pub no_transition: bool,

        /// Set the type of transition
        #[arg(long)]
        pub transition: Option<TransitionKind>,
    }

    #[derive(
        Copy, Clone, clap::ValueEnum, serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq,
    )]
    pub enum TransitionKind {
        /// Switch to the new image immediately
        No,
        /// Fade into the new image
        Xfd,
        /// Make transition happen from one side of the screen. Can be controlled by `--wipe-angle
        /// <ANGLE>`
        Wipe,
    }

    #[derive(clap::Args)]
    #[group(required = false, multiple = false)]
    pub struct Ease {
        /// Use linear easing function
        #[arg(long)]
        pub no_ease: bool,

        /// Set the type of builtin easing function
        #[arg(long)]
        pub ease: Option<EaseKind>,

        /// Cubic bezier curve to use for easing function
        #[arg(long, value_parser = parse_cubic_bezier_control_points)]
        pub cubic_curve: Option<(f64, f64, f64, f64)>,
    }

    pub fn parse_cubic_bezier_control_points(s: &str) -> Result<(f64, f64, f64, f64)> {
        let point_str: Vec<&str> = s.split(",").collect();

        if point_str.len() != 4 {
            return Err(anyhow!(
                "Parameter of cubic bezier control points must be in the form of `<PX1>,<PY1>,<PX2>,<PY2>`"
            ));
        }

        Ok((
            point_str[0]
                .parse::<f64>()
                .map_err(|e| anyhow!("Failed to parse control point coordinate: {e}"))?,
            point_str[1]
                .parse::<f64>()
                .map_err(|e| anyhow!("Failed to parse control point coordinate: {e}"))?,
            point_str[2]
                .parse::<f64>()
                .map_err(|e| anyhow!("Failed to parse control point coordinate: {e}"))?,
            point_str[3]
                .parse::<f64>()
                .map_err(|e| anyhow!("Failed to parse control point coordinate: {e}"))?,
        ))
    }

    #[derive(Copy, Clone, clap::ValueEnum, serde::Serialize, serde::Deserialize, Debug)]
    pub enum EaseKind {
        /// Default
        No,

        #[value(skip)]
        CubicBezier(f64, f64, f64, f64),

        Linear,
        Hold,
        Step,

        EaseInQuad,
        EaseOutQuad,
        EaseInOutQuad,
        EaseInCubic,
        EaseOutCubic,
        EaseInOutCubic,
        EaseInQuart,
        EaseOutQuart,
        EaseInOutQuart,
        EaseInQuint,
        EaseOutQuint,
        EaseInOutQuint,
        EaseInSine,
        EaseOutSine,
        EaseInOutSine,
        EaseInExpo,
        EaseOutExpo,
        EaseInOutExpo,
        EaseInCirc,
        EaseOutCirc,
        EaseInOutCirc,
    }

    #[derive(Copy, Clone, clap::Args, serde::Serialize, serde::Deserialize, Debug)]
    pub struct TransitionOptions {
        /// How long the transition will take in seconds. Default: 3
        #[arg(long, name = "transition-duration")]
        pub duration: Option<f64>,

        /// Frame rate for the transition. Default: 30
        #[arg(long, name = "transition-fps")]
        pub fps: Option<f64>,

        /// Whether the transition can be interrupted by a new transition. Default: false
        #[arg(long, name = "no-interrupt")]
        pub no_interrupt: bool,

        /// Wipe angle. Default: 0.0
        #[arg(long, name = "wipe-angle")]
        pub wipe_angle: Option<f64>,
    }

    pub const DEFAULT_TRANSITION_KIND: TransitionKind = TransitionKind::No;
    pub const DEFAULT_TRANSITION_DURATION: f64 = 3.0;
    pub const DEFAULT_TRANSITION_FPS: f64 = 30.0;
    pub const DEFAULT_WIPE_ANGLE: f64 = 0.0;
    pub const DEFAULT_EASE_KIND: EaseKind = EaseKind::No;
}
