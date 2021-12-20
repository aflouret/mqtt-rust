use std::{
    env,
    fs::File,
    io::{BufRead, BufReader},
};

const DEFAULT_LOGFILE: &str = "logfile.txt";
const DEFAULT_PORT: u16 = 8080;
const DEFAULT_ADDRESS: &str = "0.0.0.0:";

pub struct Config {
    pub port: u16,
    pub address: String,
    pub log_filename: String,
}

impl Config {
    pub fn new(mut args: env::Args) -> Result<Config, Box<dyn std::error::Error>> {
        args.next(); //args[0] no nos interesa, es el nombre del programa

        if let Some(config_file_path) = args.next() {
            let config_file = File::open(config_file_path)?;
            let reader = BufReader::new(config_file);
            let mut lines = reader.lines();

            let port = match lines.next() {
                Some(Ok(arg)) => match str::parse::<u16>(&arg) {
                    Ok(port) => port,
                    Err(_) => return Err("El puerto no es valido".into()),
                },
                Some(Err(_)) => return Err("El puerto no es valido".into()),
                None => DEFAULT_PORT,
            };

            let log_filename = match lines.next() {
                Some(Ok(arg)) => arg,
                Some(Err(_)) => return Err("El logfile no es valido".into()),
                None => DEFAULT_LOGFILE.to_string(),
            };

            return Ok(Config {
                port,
                address: DEFAULT_ADDRESS.to_string(),
                log_filename,
            });
        }

        Ok(Config {
            port: DEFAULT_PORT,
            address: DEFAULT_ADDRESS.to_string(),
            log_filename: DEFAULT_LOGFILE.to_string(),
        })
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            port: DEFAULT_PORT,
            address: DEFAULT_ADDRESS.to_string(),
            log_filename: DEFAULT_LOGFILE.to_string(),
        }
    }
}
