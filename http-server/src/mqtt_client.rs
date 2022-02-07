use common::all_packets::connect::{Connect, ConnectPayload};
use common::all_packets::connack::{CONNACK_CONNECTION_ACCEPTED};
use common::all_packets::puback::{Puback};
use common::all_packets::subscribe::{Subscribe};
use common::all_packets::suback::{SubackReturnCode};
use common::packet::{WritePacket, Packet, Qos, Subscription};
use std::io::{Error, ErrorKind};
use std::net::TcpStream;
use std::thread;
use std::sync::mpsc::Sender;

const ERROR_NOT_CONNECTED: &str = "The client is not connected";
const ERROR_IN_CONNECTION: &str = "Connection not accepted";
const ERROR_CONNACK_NOT_RECEIVED: &str = "Didn't received the connack packet";
const ERROR_SUBACK_NOT_RECEIVED: &str = "Didn't received the suback packet";
const ERROR_FAILED_SUBSCRIPTION: &str = "Failure in suback return code";

pub struct MQTTClient {
    socket: Option<TcpStream>,
    sender: Sender<String>
}

impl MQTTClient {
    pub fn new(sender: Sender<String>) -> MQTTClient{
        MQTTClient {
            socket: None,
            sender
        }
    }

    pub fn connect_to(&mut self, address: String) -> Result<(), Box<dyn std::error::Error>>{
        let mut socket = TcpStream::connect(address)?;
        
        let connect_packet = Connect::new(
            ConnectPayload::new(
                "serverhttp".to_string(),
                None,
                None,
                None,
                None,
            ),
            0,
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

    pub fn subscribe_to(&mut self, topic: String, qos: Qos) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(socket) = &mut self.socket {
            let mut subscribe = Subscribe::new(73);
            subscribe.add_subscription(Subscription {
                topic_filter: topic,
                max_qos: qos,
            });

            subscribe.write_to(socket)?;

            let suback = Packet::read_from(socket)?;
            match suback {
                Packet::Suback(suback) => {
                    println!("Suback received!");
                    if suback.return_codes.get(0).unwrap() == &SubackReturnCode::Failure {
                        return Err(Box::new(Error::new(ErrorKind::Other, ERROR_FAILED_SUBSCRIPTION)));
                    }


                },
                _ => return Err(Box::new(Error::new(ErrorKind::Other, ERROR_SUBACK_NOT_RECEIVED))),
            }

            return Ok(());
        }

        Err(Box::new(Error::new(ErrorKind::Other, ERROR_NOT_CONNECTED)))
    }

    pub fn run(self){
        thread::spawn(|| {
            if let Some(mut socket) = self.socket {
                loop {
                    if let Ok(Packet::Publish(publish)) = Packet::read_from(&mut socket){
                        self.sender.send(publish.application_message).unwrap();
                        if let Some(packet_id) = publish.packet_id {
                            let puback = Puback::new(packet_id);
                                Packet::Puback(puback).write_to(&mut socket).unwrap();
                        }                          
                    }

                }
            }
        });
    }
}
