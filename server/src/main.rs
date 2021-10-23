// Para usar cualquier funcion/cosa de common, hacemos "common::archivo::algo"
use std::io::{BufRead, BufReader, Read};
use std::net::{TcpListener, TcpStream};
use common::packets::Packet;

fn main() -> Result<(), ()> {   
        let address = "0.0.0.0:8080".to_owned(); 
        server_run(&address).unwrap();
        Ok(())
}

fn server_run(address: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(address)?;
    let connection = listener.accept()?; 
    let mut client_stream : TcpStream = connection.0;
    handle_client(&mut client_stream)?;
    Ok(())
}

// Leemos el packet desde el TcpStream.
fn handle_client(stream: &mut dyn Read) -> Result<(),()> {
    let packet = Packet::read_from(stream)?;
    Ok(())
}
