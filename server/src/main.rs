mod server;

use common::all_packets::connack::Connack;
use common::packet::WritePacket;
use common::parser;
use std::net::{TcpListener, TcpStream};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = "0.0.0.0:8080".to_owned();
    println!("Esuchando en {:?}", &address);

    server_run(&address)?;
    Ok(())
}

fn server_run(address: &str) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(address)?;
    let connection = listener.accept()?; // ídem ant
    let mut client_stream: TcpStream = connection.0;
    handle_client(&mut client_stream)?;
    Ok(())
}

// Leemos el connect packet y enviamos el connack desde el TcpStream.
fn handle_client(stream: &mut TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let _received_packet = parser::read_packet(stream)?;
    println!("Se recibió el connect packet");

    let connack_packet = Connack::new(false, 0);
    connack_packet.write_to(stream)?;
    println!("Se envió el connack packet");

    Ok(())
}
