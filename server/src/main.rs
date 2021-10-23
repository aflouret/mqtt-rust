// Para usar cualquier funcion/cosa de common, hacemos "common::archivo::algo"
use std::io::{Read};
use std::net::{TcpListener, TcpStream};
use common::parser;

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
fn handle_client(stream: &mut dyn Read) -> Result<(), Box<dyn std::error::Error>> {
    let packet = parser::read_packet(stream)?;
    Ok(())
}
