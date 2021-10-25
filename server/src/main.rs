// Para usar cualquier funcion/cosa de common, hacemos "common::archivo::algo"
use std::io::{Read};
use std::net::{TcpListener, TcpStream};
use common::parser;
use common::packet::{Packet, WritePacket};
use common::all_packets::connack::Connack;

fn main() -> Result<(), ()> {   
        let address = "0.0.0.0:8080".to_owned(); 
        server_run(&address).unwrap();
        Ok(())
}

fn server_run(address: &str) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(address).unwrap(); //Seria mejor usar "?" en vez de unwrap?
    let connection = listener.accept().unwrap(); // ídem ant
    let mut client_stream : TcpStream = connection.0;
    handle_client(&mut client_stream)?;
    Ok(())
}

// Leemos el packet desde el TcpStream.
fn handle_client(stream: &mut TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    //let packet = parser::read_packet(stream)?;
    let connack_packet = Connack::new(false, 0);
    connack_packet.write_to(stream)?;
    Ok(())
}
