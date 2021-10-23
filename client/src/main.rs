use std::io::stdin;
use std::io::Write;
use std::io::{BufRead, BufReader, Read};
use std::net::TcpStream;
// Para usar cualquier funcion/cosa de common, hacemos "common::archivo::algo"
fn main() -> Result<(), ()> {

    let address = "127.0.0.1:8080";
    println!("Conectándome a {:?}", address);
    
    // Para probar la conexión entre cliente-servidor, hicimos que desde el cliente
    // se escriba por stdin, se lo mande al server y que este lo imprima. En el futuro
    // le vamos a estar enviando packets como el Connect. 
    client_run(&address, &mut stdin()).unwrap();
    Ok(())
}

fn client_run(address: &str, stream: &mut dyn Read) -> std::io::Result<()> {
    let reader = BufReader::new(stream);
    let mut socket = TcpStream::connect(address)?;
    let buf: [u8; 1] = [0x10];
    for line in reader.lines() {
        if let Ok(line) = line {
            println!("Enviando: {:?}", line);
/*            socket.write(line.as_bytes())?;
            socket.write("\n".as_bytes())?;*/
            socket.write(&buf);
        }
    }
    Ok(())
}