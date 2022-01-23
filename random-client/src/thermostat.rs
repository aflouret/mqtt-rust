use std::net::TcpStream;
use common::all_packets::connect::{Connect, ConnectPayload};
use common::all_packets::connack::{CONNACK_CONNECTION_ACCEPTED};
use common::all_packets::publish::{Publish, PublishFlags};
use common::packet::{WritePacket, Packet};
use rand::prelude::*;
use std::thread;
use std::time::Duration;
use std::io::{Error, ErrorKind};

const ERROR_NOT_CONNECTED: &str = "The client is not connected";
const ERROR_IN_CONNECTION: &str = "Connection not accepted";
const ERROR_CONNACK_NOT_RECEIVED: &str = "Didn't received the connack packet";

pub struct Thermostat {
    socket: Option<TcpStream>,
    topics: Vec<String>,
}

impl Thermostat {
    pub fn new() -> Thermostat {
        Thermostat {
            socket: None,
            topics: Vec::new(),
        }
    }

    pub fn connect_to(&mut self, address: String) -> Result<(), Box<dyn std::error::Error>>{
        let mut socket = TcpStream::connect(address)?;
        
        let connect_packet = Connect::new(
            ConnectPayload::new(
                "Thermostat".to_string(),
                None,
                None,
                None,
                None,
            ),
            60,
            false,
            false,
            false,
        );

        connect_packet.write_to(&mut socket)?;

        let connack_packet = Packet::read_from(&mut socket)?;
        match connack_packet {
            Packet::Connack(connack) => {
                if connack.connect_return_code != CONNACK_CONNECTION_ACCEPTED {
                    return Err(Box::new(Error::new(ErrorKind::Other, ERROR_IN_CONNECTION)))
                }
                println!("Connack received!");
            },
            _ => return Err(Box::new(Error::new(ErrorKind::Other, ERROR_CONNACK_NOT_RECEIVED))),
        }

        self.socket = Some(socket);
        Ok(())
    }

    pub fn publish_in(&mut self, topic: String){
        self.topics.push(topic);
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>>{
        println!("Empecé a correr");

        if let Some(socket) = &mut self.socket {
            loop {
                let mut rng = thread_rng();
                let current_temp = rng.gen_range(18..30);
    
                for topic in &self.topics {
                    let publish = Publish::new(
                        PublishFlags::new(0b0100_0000), 
                        topic.clone(),
                        None,
                        "Current Temperature: ".to_string() + &current_temp.to_string(),
                    );
                    
                    publish.write_to(socket)?;
                }
    
                thread::sleep(Duration::new(10, 0));
            }
        }

        Err(Box::new(Error::new(ErrorKind::Other, ERROR_NOT_CONNECTED)))
    }
}