use common::all_packets::connect::Connect;
use common::packet::Packet;
use common::packet::WritePacket;
use common::parser;
use std::net::TcpStream;

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

        connect_packet.write_to(&mut self.server_stream)?;
        println!("Se envió el connect packet");

        let received_packet = parser::read_packet(&mut self.server_stream)?;
        if let Packet::Connack(_connack_packet) = received_packet {
            println!("Se recibió el connack packet");
        }

        Ok(())
    }
}
