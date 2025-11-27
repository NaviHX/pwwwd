use crate::cli::{
    client::{
        ClientSubcommand, DEFAULT_TRANSITION_KIND, ResizeOption, TransitionKind, TransitionOptions,
    },
    server::DEFAULT_RESIZE,
};
use serde::{Deserialize, Serialize};

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
}
