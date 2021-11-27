use std::sync::Arc;
use std::thread;
use std::time::Duration;
use common::all_packets::connack::Connack;
use common::all_packets::connect::{Connect, ConnectPayload};
use common::logging::logger::Logger;
use server::config::Config;
use server::server::Server;

use common::packet::{Packet, WritePacket};
use std::io::{self, Cursor};
use std::net::{TcpListener};
use std::collections::HashMap;
use server::client_handler::{ClientHandler};
use server::packet_processor::PacketProcessor;

use std::sync::{Mutex, mpsc};
use std::sync::mpsc::{Sender};
use std::sync::{RwLock};
use common::logging::logger::{LogMessage};

#[test]
fn main() {
        let senders_to_c_h_writers = Arc::new(RwLock::new(HashMap::<u32, Arc<Mutex<Sender<Result<Packet,Box<dyn std::error::Error + Send>>>>>>::new()));
        let (c_h_reader_tx, server_rx) = mpsc::channel::<(u32, Result<Packet,Box<dyn std::error::Error + Send>>)>();

        let config = Config::new();

        let logger = Logger::new(config.get_logfilename());
        let logger = Arc::new(logger.unwrap());

        let packet_processor = PacketProcessor::new(server_rx, senders_to_c_h_writers.clone(), logger.clone());
        let packet_processor_join_handle = packet_processor.run();
        
        let mut cursor_write = Cursor::new(Vec::new());
        let mut cursor_read = Cursor::new(Vec::new());

        let client_handler = ClientHandler::new(0, Box::new(cursor_write.clone()), Box::new(cursor_read.clone()), senders_to_c_h_writers.clone(), c_h_reader_tx.clone());
        client_handler.run();

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

        let handle = thread::spawn(move || {
            
            connect_packet.write_to(&mut cursor_read).unwrap();
            cursor_read.set_position(0);
            thread::sleep(Duration::from_millis(1000));
            cursor_write.set_position(0);

            let received_connack_packet = Packet::read_from(&mut cursor_write).unwrap();
            let test_connack_packet = Connack::new(true, 0);

            if let Packet::Connack(received_connack_packet) = received_connack_packet {
                assert_eq!(received_connack_packet.session_present, test_connack_packet.session_present);
                assert_eq!(
                    received_connack_packet.connect_return_code,
                    test_connack_packet.connect_return_code
                )
            }
            
        });

        handle.join().unwrap();

}