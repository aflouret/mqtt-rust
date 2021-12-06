use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use common::all_packets::connack::Connack;
use common::all_packets::connect::{Connect, ConnectPayload};
use common::all_packets::publish::{Publish, PublishFlags};
use common::all_packets::suback::{Suback, SubackReturnCode};
use common::all_packets::subscribe::Subscribe;
use common::logging::logger::Logger;
use server::config::Config;
use server::server::Server;

use common::packet::{Packet, WritePacket, Subscription, Qos};
use std::net::TcpStream;



#[test]
fn main() {
    //test01();
    test02();
}

fn test01() {
        
        run_server();

        thread::sleep(Duration::from_millis(1000));

        let client_handle = run_client01();
        
        client_handle.join().unwrap();
}

fn test02() {
        
    run_server();

    thread::sleep(Duration::from_millis(1000));

    let client_handle2 = run_client02();
    let client_handle3 = run_client03();
    
    client_handle2.join().unwrap();
    client_handle3.join().unwrap();

}

fn run_server() -> JoinHandle<()>{
    let config = Config::new();
    let logger = Logger::new(config.get_logfilename());
    let server = Server::new(config, Arc::new(logger.unwrap())).unwrap();
    thread::spawn(move || {
        server.server_run().unwrap();
    })
}

fn run_client01() -> JoinHandle<()> {
    thread::spawn(move || {
        let mut socket = TcpStream::connect("127.0.0.1:8080").unwrap();

        let connect_packet = Connect::new(
            ConnectPayload::new(
                "a".to_owned(),
                None,
                None,
                None,
                None,
            ),
            60,
            true,
            false,
            false,
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

fn run_client02() -> JoinHandle<()> {
    thread::spawn(move || {
        let mut socket = TcpStream::connect("127.0.0.1:8080").unwrap();

        let connect_packet = Connect::new(
            ConnectPayload::new(
                "b".to_owned(),
                None,
                None,
                None,
                None,
            ),
            60,
            true,
            false,
            false,
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

        let mut subscribe_packet = Subscribe::new(
            1,
        );
        subscribe_packet.add_subscription(Subscription{ 
            topic_filter: "topic_a".to_string(), 
            max_qos: Qos::AtLeastOnce 
        });

        subscribe_packet.write_to(&mut socket).unwrap();

        let received_suback_packet = Packet::read_from(&mut socket).unwrap();
        let mut expected_suback_packet = Suback::new(1);
        expected_suback_packet.add_return_code(SubackReturnCode::SuccessAtLeastOnce);

        if let Packet::Suback(received_suback_packet) = received_suback_packet {
            assert_eq!(received_suback_packet, expected_suback_packet);
        }

        // let received_publish_packet = Packet::read_from(&mut socket).unwrap();
        // let expected_publish_packet = Publish::new(
        //     PublishFlags::new(0b0011_0000),
        //     "topic_a".to_string(),
        //     None,
        //     "hola".to_string(),
        // );

        // if let Packet::Publish(received_publish_packet) = received_publish_packet {
        //     assert_eq!(received_publish_packet, expected_publish_packet);
        // }
        thread::sleep(Duration::from_millis(1000));
        let received_publish_packet = Packet::read_from(&mut socket).unwrap();
        let expected_publish_packet = Publish::new(
            PublishFlags::new(0b0011_0010),
            "topic_a".to_string(),
            Some(4),
            "hola".to_string(),
        );

        if let Packet::Publish(received_publish_packet) = received_publish_packet {
            assert_eq!(received_publish_packet, expected_publish_packet);
        }
    })
}


fn run_client03() -> JoinHandle<()> {
    thread::spawn(move || {
        let mut socket = TcpStream::connect("127.0.0.1:8080").unwrap();

        let connect_packet = Connect::new(
            ConnectPayload::new(
                "c".to_owned(),
                None,
                None,
                None,
                None,
            ),
            60,
            true,
            false,
            false,
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

        // let publish_packet_0 = Publish::new(
        //     PublishFlags::new(0b0011_0000),
        //     "topic_a".to_string(),
        //     None,
        //     "hola".to_string(),
        // );

        // publish_packet_0.write_to(&mut socket).unwrap();
        thread::sleep(Duration::from_millis(1000));
        let publish_packet_1 = Publish::new(
            PublishFlags::new(0b0011_0010),
            "topic_a".to_string(),
            None,
            "hola".to_string(),
        );

        publish_packet_1.write_to(&mut socket).unwrap();
    })
}