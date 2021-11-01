pub struct Config {
    port: u16,

    address: String,
}

impl Config {
    pub fn new() -> Config {
        Config {
            port: 8080,
            address: "0.0.0.0:".to_string(),
        }
    }

    pub fn get_port(&self) -> String {
        self.port.to_string()
    }

    pub fn get_address(&self) -> String {
        self.address.to_string()
    }
}
