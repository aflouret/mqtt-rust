use std::io;
use std::net::{TcpListener, TcpStream};
use common::all_packets::connack::Connack;
use common::packet::WritePacket;
use common::parser;
use crate::config::Config;

pub struct Server {

    config: Config
}

impl Server {
    pub fn new(config: Config) -> io::Result<Self>{
        Ok(Self {
            config,
        })
    }


    pub fn server_run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        let address = self.config.get_address() + &*self.config.get_port();
        println!(" address {} ",address);
        self.run(&address);
        Ok(())
    }

    fn run(self, address: &str) -> io::Result<()> {
        let listener = TcpListener::bind(address)?;
        let connection = listener.accept()?;
        let mut client_stream: TcpStream = connection.0;
        Server::handle_client(&mut client_stream)?;
        Ok(())
    }

    // Leemos el connect packet y enviamos el connack desde el TcpStream.
    fn handle_client(stream: &mut TcpStream) -> io::Result<()> {
/*        let _received_packet = parser::read_packet(stream)?;*/
        println!("Se recibió el connect packet");

/*        let connack_packet = Connack::new(false, 0);
        connack_packet.write_to(stream)?;*/
        println!("Se envió el connack packet");

        Ok(())
    }
}