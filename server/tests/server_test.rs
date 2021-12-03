use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use common::all_packets::connack::Connack;
use common::all_packets::connect::{Connect, ConnectPayload};
use common::logging::logger::Logger;
use server::config::Config;
use server::server::Server;

use common::packet::{Packet, WritePacket};
use std::io::{self, Cursor, Write};
use std::net::{TcpListener, TcpStream};
use std::collections::HashMap;
use server::client_handler::{ClientHandler};
use server::packet_processor::PacketProcessor;

use std::sync::{Mutex, mpsc};
use std::sync::mpsc::{Sender};
use std::sync::{RwLock};
use common::logging::logger::{LogMessage};

#[test]
fn main() {
        
        run_server();

        thread::sleep(Duration::from_millis(1000));

        let join_handle = run_client();
        
        join_handle.join().unwrap();
}

fn run_server() {
    let config = Config::new();
    let logger = Logger::new(config.get_logfilename());
    let server = Server::new(config, Arc::new(logger.unwrap())).unwrap();
    thread::spawn(move || {
        server.server_run().unwrap();
    });
}

fn run_client() -> JoinHandle<()> {
    thread::spawn(move || {
        let mut socket = TcpStream::connect("127.0.0.1:8080").unwrap();

        let connect_packet = Connect::new(
            ConnectPayload::new(
                "u".to_owned(),
                Some("u".to_owned()),
                Some("u".to_owned()),
                Some("u".to_owned()),
                Some("u".to_owned()),
            ),
            60,
            true,
            true,
            true,
        );
    
        connect_packet.write_to(&mut socket).unwrap();

        let received_connack_packet = Packet::read_from(&mut socket).unwrap();
        let expected_connack_packet = Connack::new(false, 0);

        if let Packet::Connack(received_connack_packet) = received_connack_packet {
            assert_eq!(received_connack_packet.session_present, expected_connack_packet.session_present);
            assert_eq!(
                received_connack_packet.connect_return_code,
                expected_connack_packet.connect_return_code
            )
        }
    })
}