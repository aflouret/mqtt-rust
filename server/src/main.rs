use common::all_packets::connack::Connack;
use common::all_packets::connect::{Connect, ConnectPayload};
use common::packet::Packet;
use crate::config::Config;
use crate::server::Server;

mod config;
mod server;
mod session;
mod client_handler;
mod packet_processor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new();
    let server = Server::new(config)?;
    server.server_run()?;

    Ok(())
}
