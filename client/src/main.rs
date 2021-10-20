use std::io::stdin;
use std::io::Write;
use std::io::{BufRead, BufReader, Read};
use std::net::TcpStream;

fn main() -> Result<(), ()> {

    let address = "127.0.0.1:8080"; // Concatenamos la dir. ip + : + puerto
    println!("Conectándome a {:?}", address);

    client_run(&address, &mut stdin()).unwrap();
    Ok(())
}

fn client_run(address: &str, stream: &mut dyn Read) -> std::io::Result<()> {
    let reader = BufReader::new(stream);
    let mut socket = TcpStream::connect(address)?;
    for line in reader.lines() {
        if let Ok(line) = line {
            println!("Enviando: {:?}", line);
            socket.write(line.as_bytes())?;
            socket.write("\n".as_bytes())?;
        }
    }
    Ok(())
}