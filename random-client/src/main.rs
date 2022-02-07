/*
Parte A) Implementar un programa que simule la generación de datos de un dispositivo (por ejemplo,
la medición de la temperatura ambiente) cada determinado tiempo y lo publique como cliente al 
servidor MQTT utilizando un tópico predefinido. Este programa puede ser de consola, sin necesidad 
de interfaz gráfica, ni de interacción con el usuario. 
Se permite utilizar el crate rand para la generación de valores.
*/

mod thermostat;
const IP: &str = "0.0.0.0";
const PORT: &str = "8080";
const TOPIC: &str = "topica";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut thermostat = thermostat::Thermostat::new();
    thermostat.connect_to(IP.to_string() + ":" + PORT)?;
    println!("Me conecté con exito");
    thermostat.publish_in(TOPIC.to_string());
    println!("Añadí topic con exito");
    thermostat.run()?;
    Ok(())
}
