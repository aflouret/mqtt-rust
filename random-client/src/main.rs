/*
Parte A) Implementar un programa que simule la generación de datos de un dispositivo (por ejemplo,
la medición de la temperatura ambiente) cada determinado tiempo y lo publique como cliente al 
servidor MQTT utilizando un tópico predefinido. Este programa puede ser de consola, sin necesidad 
de interfaz gráfica, ni de interacción con el usuario. 
Se permite utilizar el crate rand para la generación de valores.
*/

mod thermostat;
use common::packet::{Subscription, Qos};

fn main() {
    let mut thermostat = thermostat::Thermostat::new();
    thermostat.connect_to("0.0.0.0:8080".to_string()).unwrap();
    println!("Me conecté con exito");
    thermostat.subscribe_to(Subscription{ topic_filter: "topica".to_string(), max_qos: Qos::AtMostOnce });
    println!("Me subscribí con exito");
    thermostat.run();
}
