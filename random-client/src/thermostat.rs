use std::net::TcpStream;
use common::all_packets::connect::{Connect, ConnectPayload};
use common::all_packets::publish::{Publish, PublishFlags};
use common::packet::{WritePacket, Subscription};
use rand::prelude::*;
use std::thread;
use std::time::Duration;

pub struct Thermostat {
    socket: Option<TcpStream>,
    subscriptions: Vec<Subscription>,
}

impl Thermostat {
    pub fn new() -> Thermostat {
        Thermostat {
            socket: None,
            subscriptions: Vec::new(),
        }
    }

    pub fn connect_to(&mut self, address: String) -> Result<(), Box<dyn std::error::Error>>{
        let mut socket = TcpStream::connect(address.clone())?;
        
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
        self.socket = Some(socket);

        Ok(())
    }

    pub fn subscribe_to(&mut self, subscription: Subscription){
        self.subscriptions.push(subscription);
    }

    pub fn run(&self){
        println!("Empecé a correr");
        loop {
            let mut rng = thread_rng();
            let current_temp = rng.gen_range(18..30);

            for subscription in &self.subscriptions {
                let publish = Publish::new(
                    PublishFlags::new(0b0100_0000), 
                    subscription.topic_filter.clone(),
                    None,
                    "Current Temperature: ".to_string() + &current_temp.to_string(),
                );
                
                publish.write_to(&mut self.socket.as_ref().unwrap());
            }

            thread::sleep(Duration::new(10, 0));
        }
    }
}