use crate::config::Config;
use common::all_packets::connack::Connack;
use common::packet::{Packet, WritePacket};
use common::parser;
use std::io;
use std::net::{TcpListener, TcpStream};
use crate::session::Session;

pub struct Server {
    config: Config,
}
//guardar sesion de un cliente
impl Server {
    pub fn new(config: Config) -> io::Result<Self> {
        Ok(Self { config })
    }

    pub fn server_run(self) -> Result<(), Box<dyn std::error::Error>> {
        let address = self.config.get_address() + &*self.config.get_port();

        let listener = TcpListener::bind(&address)?;

        println!("Servidor escuchando en: {} ", &address);

        for stream in listener.incoming() {
            if let Ok(mut client_stream) = stream {
                self.handle_client(&mut client_stream)?;

            }
        }

        Ok(())
    }

    // Leemos y escribimos packets, etc.
    fn handle_client(
        &self,
        client_stream: &mut TcpStream,
    ) -> Result<(), Box<dyn std::error::Error>> {
        //Chequear que el cliente sea nueva y que el primer paquete sea el connect
        let received_packet = parser::read_packet(client_stream)?;
        println!("Se recibió el connect packet");
        //Si es connect el primer paquete del cliente creamos session
        if let Packet::Connect(received_packet) = received_packet {
            let session = Session::new(client_stream, received_packet);
        }
        let connack_packet = Connack::new(false, 0);
        connack_packet.write_to(client_stream)?;
        println!("Se envió el connack packet");

        Ok(())
    }
}
