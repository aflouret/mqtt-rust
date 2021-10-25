use std::net::TcpStream;
use common::all_packets::connect::Connect;
use common::packet::WritePacket;
use common::parser;
use common::packet::Packet;

// Para usar cualquier funcion/cosa de common, hacemos "common::archivo::algo"
fn main() -> Result<(), Box<dyn std::error::Error>> {

    let address = "127.0.0.1:8080";
    println!("Conectándome a {:?}", address);
    
    client_run(&address)?; // misma duda de cómo hacer el main que en server
    Ok(())
}

fn client_run(address: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut socket = TcpStream::connect(address)?;

    let connect_packet = Connect::new("pepito".to_owned(), "u".to_owned(), "p".to_owned(), "connect_flags".to_owned(), "last_will_message".to_owned(), "last_will_topic".to_owned());
    connect_packet.write_to(&mut socket)?;
    println!("Se envió el connect packet");
    
    let received_packet = parser::read_packet(&mut socket)?;
    if let Packet::Connack(connack_packet) = received_packet {
        println!("Se recibió el connack packet. Session present: {}. Connect return code: {}", connack_packet.session_present, connack_packet.connect_return_code);
    }

    Ok(())
}