use std::io::Write;
use std::net::TcpStream;
use common::all_packets::connect::Connect;
use common::packet::WritePacket;

// Para usar cualquier funcion/cosa de common, hacemos "common::archivo::algo"
fn main() -> Result<(), ()> {

    let address = "127.0.0.1:8080";
    println!("Conectándome a {:?}", address);
    
    // Para probar la conexión entre cliente-servidor, hicimos que desde el cliente
    // se escriba por stdin, se lo mande al server y que este lo imprima. En el futuro
    // le vamos a estar enviando packets como el Connect. 
    client_run(&address).unwrap();
    Ok(())
}

fn client_run(address: &str) -> std::io::Result<()> {
    let mut socket = TcpStream::connect(address)?;

    let connect_packet = Connect::new("pepito".to_owned(), "u".to_owned(), "p".to_owned(), "connect_flags".to_owned(), "last_will_message".to_owned(), "last_will_topic".to_owned());

    connect_packet.write_to(&mut socket).expect("No se pudo escribir en el socket");    
    
    Ok(())
}