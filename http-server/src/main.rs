
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

use std::cmp::max;
use std::net::TcpListener;
use std::net::TcpStream;
use std::fs;
use std::sync::MutexGuard;
use std::thread;
use common::packet::{Qos};
use std::sync::{mpsc, Mutex};
use std::sync::Arc;
use response::Response;
use crate::request::Request;
mod mqtt_client;
mod request;
mod response;

const ERROR_404_HTML_PATH: &str = "src/htmls/error404.html";
const ERROR_500_HTML_PATH: &str = "src/htmls/error500.html";
const HEADER_HTML_PATH: &str = "src/htmls/header.html";
const FOOTER_HTML_PATH: &str = "src/htmls/footer.html";
const IP: &str = "0.0.0.0";
const PORT: &str = "8081";
const IP_MQTT: &str = "0.0.0.0";
const PORT_MQTT: &str = "8080";
const TOPIC_MQTT: &str = "temperature";
const QOS_MQTT: Qos = Qos::AtMostOnce;
const SOCKET_WRITE_ERROR_MSG: &str = "Error de escritura en el socket";
const N_MESSAGES_TO_SHOW: i16 = 5;

const HTTP_VERSION : &str ="HTTP/1.1";
const OK_RETURN_CODE: &str = "200 OK";
const NOT_FOUND_RETURN_CODE: &str = "404 Not Found";
const SERVER_ERROR_RETURN_CODE: &str = " 500 Internal Server Error";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (sender, receiver) = mpsc::channel::<String>();
    let mut client = mqtt_client::MQTTClient::new(sender);
    client.connect_to(IP_MQTT.to_string() + ":" + PORT_MQTT)?;
    println!("Me conecté con exito");
    client.subscribe_to(TOPIC_MQTT.to_string(), QOS_MQTT)?;
    println!("Me subscribí con exito a: {}", TOPIC_MQTT);
    client.run();

    let title = format!("    <h2>Listening to topic: {}</h2>", TOPIC_MQTT);
    let messages = Arc::new(Mutex::new(vec![title]));
    let messages_clone = messages.clone();
    let join_handler = thread::spawn(move || {
        loop {
            if let Ok(message) = receiver.recv(){
                stack_mqtt_message(messages_clone.lock().unwrap(), &message);
            }
        }
    });

    let listener = TcpListener::bind(IP.to_string() + ":" + PORT)?;

    let mut join_handles = vec![];
    for stream in listener.incoming() {
        let messages_clone = messages.clone();
        let mut stream = stream?;

        let join = thread::spawn(move || -> Result<(), std::io::Error> {
            if let Err(_) = handle_connection(stream.try_clone()?, messages_clone) {
                let html_in_string = get_html(vec![fs::read_to_string(ERROR_500_HTML_PATH)?])?;
                create_response(SERVER_ERROR_RETURN_CODE, html_in_string).write_to(&mut stream)?;
            }
            Ok(())
        });
        join_handles.push(join);
    }

    join_handler.join().unwrap();
    for handle in join_handles {
        handle.join().unwrap()?;        
    }
    Ok(())
}

fn handle_connection(mut stream: TcpStream, messages: Arc<Mutex<Vec<String>>>) -> Result<(), Box<dyn std::error::Error>> {
    let request = Request::read_from(&mut stream)?;
    let response;
    let html_in_string;
    if request.wants_to_access_page() {
        html_in_string = get_html(messages.lock().unwrap().clone().to_owned())?;
        response = create_response(OK_RETURN_CODE, html_in_string);
    }
    else {
        println!("Request incorrecto. Enviando código de error 404...");
        html_in_string = get_html(vec![fs::read_to_string(ERROR_404_HTML_PATH)?])?;
        response = create_response(NOT_FOUND_RETURN_CODE, html_in_string);
    } 

    if let Err(_) = response.write_to(&mut stream) {
        return Err(SOCKET_WRITE_ERROR_MSG.into());
    }
    Ok(())
}

// Aux functions

fn stack_mqtt_message(mut messages_vector: MutexGuard<'_, Vec<String>, >, message: &str) {
    messages_vector.push(format!("\n        <p>{}</p>", message));
    let elements_to_remove = max(0_i16, messages_vector.len() as i16 - N_MESSAGES_TO_SHOW -1_i16) as usize;
    messages_vector.drain(1..elements_to_remove+1);
}

fn get_html(messages: Vec<String>) -> Result<String, std::io::Error> {
    let html_header = fs::read_to_string(HEADER_HTML_PATH)?;
    let html_body = format!(
        r#"  <body>
    {}
  </body>"#,
        messages.join("")
    );
    let html_footer = fs::read_to_string(FOOTER_HTML_PATH)?;
    Ok(html_header + &html_body + &html_footer)
}

fn create_response(error_code: &str, html: String) -> Response {
    Response::new(
        error_code,
        Some(vec![format!("Content-Length: {}", html.len())]),
        Some(html)
    )
}