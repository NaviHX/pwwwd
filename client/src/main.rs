use clap::Parser;
use common::{cli, ipc};
use tracing::debug;

fn main() {
    tracing_subscriber::fmt::init();
    let args = cli::client::Args::parse();
    let message = ipc::Message::from_cli_command(args.subcommand);

    debug!("Message to be sent: {message:?}");

    // TODO: Send the message to the daemon.
}
