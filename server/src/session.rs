use std::net::TcpStream;
use common::all_packets::connect::Connect;
use common::packet::Packet;



pub struct Session {
    socket: TcpStream,
    client_packets: Vec<Packet>,
    client: ClientData,
}


impl Session {
    pub fn new(client_stream: TcpStream, packet_connect: Connect) -> Session {
        Session {
            socket: client_stream,
            client_packets: vec![],
            client: parse_connect_data(packet_connect),
        }
    }
}

fn parse_connect_data(packet_connect: Connect) -> ClientData {

}

pub struct ClientData{
    client_id: String,
    username: Option<String>,
    password: Option<String>,
}