use crate::config::Config;
use common::all_packets::connack::Connack;
use common::packet::{Packet, WritePacket};
use common::parser;
use std::io;
use std::net::{TcpListener, TcpStream};
use crate::session::Session;
use std::collections::HashMap;

pub struct Server {
    config: Config,
    clients: HashMap<String, Session>,
}
//guardar sesion de un cliente
impl Server {
    pub fn new(config: Config) -> io::Result<Self> {
        Ok(Self { config, clients: HashMap::new() })
    }

    pub fn server_run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        let address = self.config.get_address() + &*self.config.get_port();

        let listener = TcpListener::bind(&address)?;

        println!("Servidor escuchando en: {} ", &address);

        for stream in listener.incoming() {
            if let Ok(mut client_stream) = stream {
                self.handle_client(client_stream)?;

            }
        }

        Ok(())
    }

    // Leemos y escribimos packets, etc.
    fn handle_client(
        &mut self,
        mut client_stream: TcpStream,
    ) -> Result<(), Box<dyn std::error::Error>> {
        //Chequear que el cliente sea nueva y que el primer paquete sea el connect
        let received_packet = parser::read_packet(&mut client_stream)?;
        println!("Se recibió el connect packet");
        //Si es connect el primer paquete del cliente creamos session
        //Preguntar si connack y los otros paquetes los manda el servidor o la sesion
        if let Packet::Connect(received_packet) = received_packet {
            let mut session = Session::new(client_stream, received_packet, &self.clients)?;
            let connack_packet = Connack::new(false, 0);
            connack_packet.write_to(session.get_socket())?;
            println!("Se envió el connack packet");
            self.clients.insert(session.get_client_id().to_string(), session);
        }

        /*loop {
            let received_packet = parser::read_packet(&mut client_stream)?;
            match received_packet {
                Packet::Connect(connect_packet) => {handle_connect_packet(connect_packet)},
                Packet::Publish(publish_packet) => {handle_publish_packet(publish_packet)},
                _ => {return Err("Invalid packet".into())},
            }
        }*/

        Ok(())
    }
}
