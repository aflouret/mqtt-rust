use std::collections::HashMap;
use std::net::TcpStream;
use common::all_packets::connect::Connect;
use common::packet::Packet;



pub struct Session {
    socket: TcpStream,
    client_packets: Vec<Packet>,
    client: ClientData,
}


impl Session {
    pub fn new(client_stream: TcpStream, packet_connect: Connect, clients: &HashMap<String, Session>) -> Result<Session, Box<dyn std::error::Error>> {

        let client_data = parse_connect_data(packet_connect, clients)?;
        Ok(Session {
            socket: client_stream,
            client_packets: vec![],
            client: client_data,

        })
    }

    pub fn get_socket(&mut self) -> &mut TcpStream {
        &mut self.socket
    }

    pub fn get_client_id(&self) -> &String {
        &self.client.client_id
    }
}

fn parse_connect_data(packet_connect: Connect, clients: &HashMap<String, Session>) -> Result<ClientData, Box<dyn std::error::Error>> {
    
    if clients.contains_key(&packet_connect.connect_payload.client_id) {
        return Err("Client ID already exists".into());
    }

    Ok(ClientData{
        client_id: packet_connect.connect_payload.client_id,
        username: packet_connect.connect_payload.username,
        password: packet_connect.connect_payload.password,
    })
}


pub struct ClientData{
    client_id: String,
    username: Option<String>,
    password: Option<String>,
}