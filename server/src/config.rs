use std::env;

const DEFAULT_LOGFILE: &str = "logfile.txt";
const DEFAULT_PORT: u16 = 8080;
const DEFAULT_ADDRESS: &str = "0.0.0.0:";

pub struct Config {
    pub port: u16,
    pub address: String,
    pub log_filename: String,
}

impl Config {
    pub fn new(mut args: env::Args) ->  Result<Config, String>  {
        
        args.next(); //args[0] no nos interesa, es el nombre del programa

        let port = match args.next() {
            Some(arg) => match str::parse::<u16>(&arg) {
                Ok(port) => port,
                Err(_) => return Err("El puerto no es valido".into())
            },
            None => DEFAULT_PORT,
        };

        let log_filename = match args.next() {
            Some(arg) => arg,
            None => DEFAULT_LOGFILE.to_string(),
        };
        
        Ok(Config {
            port,
            address: DEFAULT_ADDRESS.to_string(),
            //Path de archivo sobre el cuál se realizará un dump
            //Intervalo de tiempo para el cual se realizará el dump
            log_filename
        })
    }
}

impl Default for Config{
    fn default() -> Self {
        Config { port: DEFAULT_PORT, address: DEFAULT_ADDRESS.to_string(), log_filename: DEFAULT_LOGFILE.to_string() }
    }
}