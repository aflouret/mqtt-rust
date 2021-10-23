use std::io::Write;
use std::net::TcpStream;
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
    let num: u8 = 0x10;
    println!("Enviando: {:0x}", &num);
    socket.write(&num.to_be_bytes()).expect("No se pudo escribir en el socket");    
    
    Ok(())
}