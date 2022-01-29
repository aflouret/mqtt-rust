
/*
Parte B) Implementar un programa cliente MQTT que se suscriba al tópico anterior para recibir los
 datos y que actúe como servidor HTTP para publicar y exponer esos datos a través de una web
 accesible desde el browser. Es decir, este programa debe actuar como servidor HTTP.

Investigar para ello, los lineamientos del protocolo de comunicación HTTP y del formato HTML.
No es necesario que la página mostrada se actualice automáticamente.

Nota importante: No se permite el uso de crates externos de frameworks HTTP.
Se debe implementar la comunicación y el servidor a partir del uso de sockets TCP,
como se ha trabajado en el desarrollo del curso.
 */

use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::fs;

const HTML_PATH: &str = "http-server/src/publishing_page.html";
const IP: &str = "0.0.0.0";
const PORT: &str = "8081";

fn main() {
    let listener = TcpListener::bind(IP.to_string() + ":" + PORT).unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    stream.read(&mut buffer).unwrap();

    // Imprime el request del navegador para conectarse al server (yo)
    println!("Request: {}", String::from_utf8_lossy(&buffer[..]));

    // Generamos la response con la temperatura que nos llegó del client MQTT
    let html_in_string = fs::read_to_string(HTML_PATH).unwrap();
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
        html_in_string.len(),
        html_in_string
    );

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}