use common::logging::logger::Logger;
use server::config::Config;
use server::server::Server;
use std::env;
use std::process;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Error al leer parametros: {}", err);
        process::exit(1);
    });
    let logger = Logger::new(&config.log_filename);
    let server = Server::new(config, Arc::new(logger?))?;
    server.server_run()?;

    Ok(())
}
