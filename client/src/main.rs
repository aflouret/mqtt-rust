mod client;

use common::all_packets::connect::Connect;
use common::all_packets::connect::ConnectFlags;
use common::packet::Packet;
use common::packet::WritePacket;
use common::parser;
use std::net::TcpStream;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = "127.0.0.1:8080";
    println!("Conectándome a {:?}", address);

    client_run(&address)?;
    Ok(())
}

fn client_run(address: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut socket = TcpStream::connect(address)?;

    let connect_packet = Connect::new(
        "pepito".to_owned(),
        Some("u".to_owned()),
        Some("u".to_owned()),
        ConnectFlags::new(false, false, false, false, false, false),
        Some("u".to_owned()),
        Some("u".to_owned()),
    );
    connect_packet.write_to(&mut socket)?;
    println!("Se envió el connect packet");

    let received_packet = parser::read_packet(&mut socket)?;
    if let Packet::Connack(connack_packet) = received_packet {
        println!(
            "Se recibió el connack packet. Session present: {}. Connect return code: {}",
            connack_packet.session_present, connack_packet.connect_return_code
        );
    }

    Ok(())
}
