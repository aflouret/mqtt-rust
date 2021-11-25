use std::collections::HashMap;
use std::net::TcpStream;
use common::all_packets::connect::Connect;
use common::packet::{Packet, Topic};

//Manjea datos del cliente
pub struct Session {
    client_handler_id: Option<u32>,
    client_data: ClientData,
    client_packets: Vec<Packet>,
    client_subscriptions: Vec<Topic>,
    not_fully_transmitted_messages: Vec<NotFullyTransmittedMessages>
}

impl Session {
    pub fn new(client_handler_id: u32, packet_connect: Connect) -> Result<Session, Box<dyn std::error::Error>> {
        let client_data = parse_connect_data(packet_connect);

        Ok(Session {
            client_handler_id: Some(client_handler_id),
            client_data,
            client_packets: vec![],
            client_subscriptions: vec![],
            not_fully_transmitted_messages: vec![],
            
        })
    }

    pub fn get_client_id(&self) -> &String {
        &self.client_data.client_id
    }

    pub fn get_client_handler_id(&self) -> Option<u32> {
        self.client_handler_id
    }

    pub fn is_active(&self) -> bool {
        self.client_handler_id.is_some()
    }

    pub fn connect(&mut self, client_handler_id: u32) {
        self.client_handler_id = Some(client_handler_id);
    }

    pub fn disconnect(&mut self) {
        self.client_handler_id = None;
    }

    pub fn is_subscribed_to(&self, topic_name: &String) -> bool {
        for topic in &self.client_subscriptions {
            if &topic.name == topic_name {
                return true;
            }
        }
        return false;
    }
}

fn parse_connect_data(packet_connect: Connect) -> ClientData {
    ClientData{
        client_id: packet_connect.connect_payload.client_id,
        username: packet_connect.connect_payload.username,
        password: packet_connect.connect_payload.password,
    }
}

pub struct ClientData{
    client_id: String,
    username: Option<String>,
    password: Option<String>,
}

/* ---------------------------------------------------------- */


pub enum NotFullyTransmittedMessages {
    // QoS1 messages sent to the Client, but not been completely acknowledged
    SentButNotAcknowledged(ApplicationMessage),
    // QoS1 messages pending transmission to the Client
    NotSent(ApplicationMessage),
    // Optional: QoS0 messages pending transmission to the Client
}

pub struct ApplicationMessage;