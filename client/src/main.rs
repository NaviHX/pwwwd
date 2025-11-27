use anyhow::Result;
use clap::Parser;
use common::{cli, ipc};
use std::os::unix::net::UnixStream;
use tracing::{debug, error, info};

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = cli::client::Args::parse();
    let message = ipc::Message::from_cli_command(args.subcommand);

    debug!("Message to be sent: {message:?}");

    debug!("Trying to connect pwwwd socked ...");
    let uds_path = ipc::default_uds_path()?;
    let mut socket = UnixStream::connect(uds_path)?;

    debug!("Trying to send the message to the daemon ...");
    message.send(&mut socket)?;

    debug!("Trying to receive reply from the daemon ...");
    let reply = ipc::Reply::receive(&mut socket)?;

    match reply {
        ipc::Reply::Ok => info!("Ok"),
        ipc::Reply::Error(e) => error!("Daemon encountered error when processing the request: {e}"),
    }

    Ok(())
}
