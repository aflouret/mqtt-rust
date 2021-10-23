// Para usar cualquier funcion/cosa de common, hacemos "common::archivo::algo"
use std::io::{Read};
use std::net::{TcpListener, TcpStream};
use common::parser;

fn main() -> Result<(), ()> {   
        let address = "0.0.0.0:8080".to_owned(); 
        server_run(&address).unwrap();
        Ok(())
}

fn server_run(address: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(address).unwrap();
    let connection = listener.accept().unwrap();
    let mut client_stream : TcpStream = connection.0;
    handle_client(&mut client_stream)?;
    Ok(())
}

// Leemos el packet desde el TcpStream.
fn handle_client(stream: &mut dyn Read) -> std::io::Result<()> {
    let mut num_buffer = [0u8; 1];
    stream.read_exact(&mut num_buffer)?;
    let num = u8::from_be_bytes(num_buffer);
    println!("Recibido: {:#0x}", &num);

    //let packet = parser::read_from_server(stream).unwrap();
    Ok(())
}
