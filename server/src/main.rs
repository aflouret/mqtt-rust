use crate::config::Config;
use crate::server::Server;

mod config;
mod server;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new();
    let server = Server::new(config)?;
    server.server_run()?;

    Ok(())
}
