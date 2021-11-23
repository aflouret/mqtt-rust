

const LOGFILE: &str = "logfile.txt";

pub struct Config {
    port: u16,
    address: String,
    log_filename: String,
}

impl Config {
    pub fn new() -> Config {
        Config {
            port: 8080,
            address: "0.0.0.0:".to_string(),
            //Path de archivo sobre el cuál se realizará un dump
            //Intervalo de tiempo para el cual se realizará el dump
            log_filename: LOGFILE.to_string(),
        }
    }

    pub fn get_port(&self) -> String {
        self.port.to_string()
    }

    pub fn get_address(&self) -> String {
        self.address.to_string()
    }

    pub fn get_logfilename(&self) -> String {self.log_filename.to_string()}
}
