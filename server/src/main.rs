use common::all_packets::connack::Connack;
use common::all_packets::connect::{Connect, ConnectFlags, ConnectPayload};
use common::packet::Packet;
use crate::config::Config;
use crate::server::Server;

mod config;
mod server;
mod session;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new();
    let server = Server::new(config)?;
    server.server_run()?;

    Ok(())
}
