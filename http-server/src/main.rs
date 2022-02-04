
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
use std::thread;
use common::packet::{Qos};
use std::sync::{mpsc, Mutex};
use std::sync::Arc;
mod mqtt_client;

const HTML_PATH: &str = "src/publishing_page.html";
const IP: &str = "0.0.0.0";
const PORT: &str = "8081";
const IP_MQTT: &str = "0.0.0.0";
const PORT_MQTT: &str = "8080";
const TOPIC_MQTT: &str = "topica";
const QOS_MQTT: Qos = Qos::AtMostOnce;

pub const HEADER: &str = r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <title>PublishingPage</title>
  </head>
"#;

pub const FOOTER: &str = r#"
</html>
"#;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (sender, receiver) = mpsc::channel::<String>();
    let mut client = mqtt_client::MQTTClient::new(sender);
    client.connect_to(IP_MQTT.to_string() + ":" + PORT_MQTT)?;
    println!("Me conecté con exito");
    client.subscribe_to(TOPIC_MQTT.to_string(), QOS_MQTT)?;
    println!("Me subscribí con exito a: {}", TOPIC_MQTT);
    client.run();

    let body = Arc::new(Mutex::new("    <h1>Current Temperature:</h1>".to_string()));
    let body_clone = body.clone();
    thread::spawn(move || {
        loop {
            if let Ok(message) = receiver.recv(){
                body_clone.lock().unwrap().push_str(&format!("\n        <p>{}</p>", &message));
            }
        }
    });

    let listener = TcpListener::bind(IP.to_string() + ":" + PORT).unwrap();

    for stream in listener.incoming() {
        let b = body.clone();
        let stream = stream.unwrap();

        thread::spawn(move || {
            handle_connection(stream, b);
        });
    }

    Ok(())
}

fn handle_connection(mut stream: TcpStream, body: Arc<Mutex<String>>) {
    let mut buffer = [0; 1024];

    stream.read(&mut buffer).unwrap();

    // Imprime el request del navegador para conectarse al server (yo)
    println!("Request: {}", String::from_utf8_lossy(&buffer[..]));

    // Generamos la response con la temperatura que nos llegó del client MQTT
    //let html_in_string = fs::read_to_string(HTML_PATH).unwrap();

    let html_in_string = HEADER.to_string() + &get_body(&body.lock().unwrap().clone()) + FOOTER;
    println!("el html actual es: {}", html_in_string);
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
        html_in_string.len(),
        html_in_string
    );

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

pub fn get_body(body: &str) -> String {
    format!(
        r#"  <body>
    {}
  </body>"#,
        body
    )
}