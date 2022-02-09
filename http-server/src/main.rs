
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

use std::net::TcpListener;
use std::net::TcpStream;
use std::fs;
use std::thread;
use common::packet::{Qos};
use std::sync::{mpsc, Mutex};
use std::sync::Arc;
use response::Response;

use crate::request::Request;
mod mqtt_client;
mod request;
mod response;

const ERROR_HTML_PATH: &str = "src/error.html";
const HEADER_HTML_PATH: &str = "src/header.html";
const FOOTER_HTML_PATH: &str = "src/footer.html";
const IP: &str = "0.0.0.0";
const PORT: &str = "8081";
const IP_MQTT: &str = "0.0.0.0";
const PORT_MQTT: &str = "8080";
const TOPIC_MQTT: &str = "temperature";
const QOS_MQTT: Qos = Qos::AtMostOnce;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (sender, receiver) = mpsc::channel::<String>();
    let mut client = mqtt_client::MQTTClient::new(sender);
    client.connect_to(IP_MQTT.to_string() + ":" + PORT_MQTT)?;
    println!("Me conecté con exito");
    client.subscribe_to(TOPIC_MQTT.to_string(), QOS_MQTT)?;
    println!("Me subscribí con exito a: {}", TOPIC_MQTT);
    client.run();

    let title = format!("    <h1>Listening to: {}</h1>", TOPIC_MQTT);
    let body = Arc::new(Mutex::new(title));
    let body_clone = body.clone();
    let join_handler = thread::spawn(move || {
        loop {
            if let Ok(message) = receiver.recv(){
                body_clone.lock().unwrap().push_str(&format!("\n        <p>{}</p>", &message));
            }
        }
    });

    let listener = TcpListener::bind(IP.to_string() + ":" + PORT)?;

    let mut join_handles = vec![];
    for stream in listener.incoming() {
        let b = body.clone();
        let mut stream = stream.unwrap();

        let join = thread::spawn(move || {
            if let Err(server_error) = handle_connection(stream.try_clone().unwrap(), b) {
                let response = Response::new(
                    500,
                    "server error...",
                    Some(vec![format!("Content-Length: {}", "hola".len())]),
                    Some("hola".to_string()));
                response.write_to(&mut stream);
            }
        });
        join_handles.push(join);
    }

    join_handler.join().unwrap();
    for handle in join_handles {
        handle.join().unwrap();
    }
    Ok(())
}

fn handle_connection(mut stream: TcpStream, body: Arc<Mutex<String>>) -> Result<(), Box<dyn std::error::Error>> {
    let request = Request::read_from(&mut stream)?;
    let response;
    if request.is_simple_get() {
        // Generamos la response con la temperatura que nos llegó del client MQTT
        //let html_in_string = HEADER.to_string() + &get_body(&body.lock().unwrap().clone()) + FOOTER;
        let html_in_string = get_main_html(&body.lock().unwrap().clone());
        response = Response::new(
            200, 
            "OK",
            Some(vec![format!("Content-Length: {}", html_in_string.len())]),
        Some(html_in_string)); 
    }
    else {
        println!("Request incorrecto. Enviando código de error 404...");
        let html_in_string = fs::read_to_string(ERROR_HTML_PATH).unwrap();
        response = Response::new(
            404, 
            "Not Found",
            Some(vec![format!("Content-Length: {}", html_in_string.len())]),
        Some(html_in_string));
    }

    response.write_to(&mut stream)
}

fn get_main_html(body: &str) -> String {
    let html_header = fs::read_to_string(HEADER_HTML_PATH).unwrap();
    let html_body = format!(
        r#"  <body>
    {}
  </body>"#,
        body
    );
    let html_footer = fs::read_to_string(FOOTER_HTML_PATH).unwrap();
    html_header + &html_body + &html_footer
}