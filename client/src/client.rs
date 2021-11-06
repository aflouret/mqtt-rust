use common::all_packets::connect::Connect;
use common::all_packets::connack::Connack;
use common::all_packets::publish::{Publish, PublishFlags};
use std::time::Duration;

use common::packet::Packet;
use common::packet::WritePacket;
use common::parser;
use std::net::TcpStream;
use std::thread;

pub struct Client {
    client_id: String,
    server_stream: TcpStream,
}
//hilo para enviarle cosas al servidor
impl Client {
    //Devuelve un cliente ya conectado al address
    pub fn new(client_id: String, address: String) -> Result<Client, Box<dyn std::error::Error>> {
        let server_stream = TcpStream::connect(&address)?;
        println!("Conectándome a {:?}", &address);

        Ok(Client {
            client_id,
            server_stream,
        })
    }

    // Por ahora solo manda el connect y se fija de recibir bien el connack
    pub fn client_run(&mut self, connect_packet: Connect,
    ) -> Result<(), Box<dyn std::error::Error>> {

        // handle connection, 
        connect_packet.write_to(&mut self.server_stream).unwrap();
        println!("Se envió el connect packet");
        let received_packet = parser::read_packet(&mut self.server_stream).unwrap();
        if let Packet::Connack(_connack_packet) = received_packet {
            println!("Se recibió el connack packet");
        }

        let mut server_stream_write = self.server_stream.try_clone()?;
        thread::spawn(move || {
            loop {
                println!("Entramos al loop");
                let publish_packet = Publish::new(
                    PublishFlags::new(0b0100_1011),
                    "Topic".to_string(),
                    Some(15),
                    "Message".to_string(),
                );
                publish_packet.write_to(&mut server_stream_write).unwrap();
                println!("Se envio el publish");
                thread::sleep(Duration::from_millis(1000));
            }
        });
        
        let mut server_stream_read = self.server_stream.try_clone()?;
        thread::spawn(move || {
            loop {

            }
        });
        

        Ok(())
    }
}
