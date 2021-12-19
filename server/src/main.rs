use common::logging::logger::Logger;
use server::config::Config;
use server::server::Server;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new();
    let logger = Logger::new(config.get_logfilename());
    let server = Server::new(config, Arc::new(logger?))?;
    server.server_run()?;

    Ok(())
}
