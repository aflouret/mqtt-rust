use std::collections::HashMap;
use std::net::TcpStream;
use common::all_packets::connect::Connect;
use common::packet::Packet;

pub struct Session {
    socket: TcpStream,
    client_packets: Vec<Packet>,
    client: ClientData,
    pub is_active: bool,
    client_subscriptions: Vec<Subscription>,
    not_fully_transmitted_messages: Vec<NotFullyTransmittedMessages>
}

impl Session {
    pub fn new(client_stream: TcpStream, packet_connect: Connect) -> Result<Session, Box<dyn std::error::Error>> {
        let client_data = parse_connect_data(packet_connect);

        Ok(Session {
            socket: client_stream,
            client_packets: vec![],
            client: client_data,
            is_active: false,
            client_subscriptions: vec![],
            not_fully_transmitted_messages: vec![],
            
        })
    }

    pub fn get_socket(&mut self) -> &mut TcpStream {
        &mut self.socket
    }

    pub fn get_client_id(&self) -> &String {
        &self.client.client_id
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn connect(&mut self) {
        self.is_active = true;
    }

    pub fn disconnect(&mut self) {
        self.is_active = false;
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

pub struct Subscription;

pub enum NotFullyTransmittedMessages {
    // QoS1 messages sent to the Client, but not been completely acknowledged
    SentButNotAcknowledged(ApplicationMessage),
    // QoS1 messages pending transmission to the Client
    NotSent(ApplicationMessage),
    // Optional: QoS0 messages pending transmission to the Client
}

pub struct ApplicationMessage;