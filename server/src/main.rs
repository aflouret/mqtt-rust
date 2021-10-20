mod ../common/src/parser; //TODO ver como importar los modulos de common
use std::io::{BufRead, BufReader, Read};
use std::net::{TcpListener, TcpStream};

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

fn handle_client(stream: &mut dyn Read) -> Result<()> {
    let packet = Packet::read_from(stream)?;
    Ok(())
}
