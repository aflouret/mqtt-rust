use std::sync::Arc;
use common::logging::logger::Logger;
use crate::config::Config;
use crate::server::Server;

mod config;
mod server;
mod session;
mod client_handler;
mod packet_processor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new();
    let logger = Logger::new(config.get_logfilename());
    let server = Server::new(config, Arc::new(logger.unwrap()))?;
    server.server_run()?;

    Ok(())
}
