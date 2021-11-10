use std::net::TcpStream;
use common::all_packets::connect::Connect;
use crate::session::Session;

pub struct ClientHandler {
    //Maneja la conexion del socket
    socket: TcpStream,//Option<Tcp>, cuando se desconecta queda en None
    //Crea la sesion(socket)  -> Server,
    //Info al server
    // channel - con el Server
}


/*pub fn new(client_stream: TcpStream, packet_connect: Connect) -> Result<Session, Box<dyn std::error::Error>> {
    let client_data = parse_connect_data(packet_connect);

    Ok(Session {
        socket: client_stream,
        client_packets: vec![],
        client: client_data,
        is_active: false,
        client_subscriptions: vec![],
        not_fully_transmitted_messages: vec![],

    })
}*/