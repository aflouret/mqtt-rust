use crate::config::Config;
use crate::server::Server;

mod server;
mod config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
/*    let address = "0.0.0.0:8080".to_owned();
    println!("Esuchando en {:?}", &address);*/
    let config = Config::new();
    let server = Server::new(config)?;
    server.server_run()?;
    Ok(())
}

