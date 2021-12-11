use std::convert::TryInto;
use common::all_packets::connect::{Connect, ConnectPayload};
use common::all_packets::connack::Connack;
use common::all_packets::publish::{Publish, PublishFlags};
use common::all_packets::pingreq::Pingreq;
use std::time::Duration;

use common::packet::Packet;
use common::packet::WritePacket;
use common::parser;
use std::net::TcpStream;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::{io, thread};
use std::io::{BufRead, BufReader, Read};
use common::all_packets::puback::Puback;
use common::packet::Packet::Suback;
use crate::client::ClientStatus::{StatusOff, StatusOn};
use crate::handlers::{EventHandlers, HandleDisconnect, HandlePublish, HandleSubscribe, HandleUnsubscribe};
use crate::HandleConection;
use crate::response::{PubackResponse, PublishResponse, ResponseHandlers};

const MAX_KEEP_ALIVE : u16 = 65000; // KeepAlive muy grande para el caso que keep_alive es 0 => en este caso el server no espera ningun tiempo para que el client envíe paquetes.

#[derive(Debug)]
enum ClientStatus {
    StatusOn,
    StatusOff,
}

#[derive(Debug)]
pub struct Client {
    client_id: String,
    server_stream: Option<TcpStream>,
}

impl Client {
    /// Devuelve un client con un socket no conectado.
    pub fn new(client_id: String) -> Client {
        Client {
            client_id,
            server_stream: None,
        }
    }

    pub fn set_server_stream(&mut self, stream: TcpStream) {
        self.server_stream = Some(stream);
    }

    pub fn start_client(mut self, recv_conection: Receiver<EventHandlers>, sender_to_window: Sender<ResponseHandlers>) -> Result<(), Box<dyn std::error::Error>> {
        thread::spawn(move || {
            let mut keep_alive_sec: u16 = 0;
            loop {
                if let Ok(conection) = recv_conection.recv() { 
                    match conection {
                        EventHandlers::HandleConection(conec) => {
                            self.handle_conection(conec, sender_to_window.clone(), &mut keep_alive_sec).unwrap();
                            println!("Connected Client");
                            break;
                        }
                        _ => println!("Primero se debe conectar"),
                    };
                }
            }

            loop {
                if keep_alive_sec == 0 {
                    keep_alive_sec = MAX_KEEP_ALIVE;
                }
                match recv_conection.recv_timeout(Duration::new(keep_alive_sec as u64, 0)) {
                    Ok(EventHandlers::HandleConection(conec)) => {
                        self.handle_conection(conec, sender_to_window.clone(), &mut keep_alive_sec).unwrap();
                        //println!("Client already connected"); //Revisar que hacer en este caso
                    }
                    
                    Ok(EventHandlers::HandlePublish(publish)) => {
                        self.handle_publish(publish).unwrap();
                    }
                    Ok(EventHandlers::HandleSubscribe(subscribe)) => {
                        self.handle_subscribe(subscribe).unwrap();
                    },
                    Ok(EventHandlers::HandleUnsubscribe(unsubs)) => {
                        self.handle_unsubscribe(unsubs).unwrap();
                    },
                    Ok(EventHandlers::HandleDisconnect(disconnect)) => {
                        self.handle_disconnect(disconnect).unwrap();
                    }

                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        self.handle_pingreq().unwrap(); 
                    }

                    _ => ()
                };
            }
        });

        Ok(())
    }

    pub fn handle_response(mut s: TcpStream, sender: Sender<ResponseHandlers>) {
        let mut subscriptions_msg: Vec<String> = Vec::new();
        thread::spawn(move || {
            loop {
                let receiver_packet = Packet::read_from(&mut s).unwrap();
                match receiver_packet {
                    Packet::Connack(connack) => {
                        println!("CLIENT: CONNACK packet successful received");
                       // sender.send("PONG".to_string());
                    }
                    Packet::Puback(puback) => {
                        println!("CLIENT: PUBACK packet successful received");
                        //sender.send("Topic Successfully published".to_string());
                    }
                    Packet::Suback(suback) => {
                        println!("CLIENT: SUBACK packet successful received");
                    },
                    Packet::Unsuback(unsuback ) => {
                        println!("CLIENT: UNSUBACK packet successful received");
                    },
                    Packet::Publish(publish) => {
                        println!("CLIENT: Recibi publish: msg: {:?}", &publish.application_message);
                        subscriptions_msg.push(publish.application_message.to_string() + " - topic:" + &*publish.topic_name.to_string() + " \n");
                        let response = ResponseHandlers::PublishResponse(PublishResponse::new(publish, subscriptions_msg.clone(), "Published Succesfull".to_string()));
                        sender.send(response);
                        let mut puback = Puback::new(10);
                        puback.write_to(&mut s);
                    },
                    Packet::Pingresp(_pingresp) => {
                        println!("CLIENT: Pingresp successful received");
                    }
                    _ => (),
                };
            }
        });
    }


    pub fn handle_pingreq(&mut self) -> io::Result<()>{
        if let Some(socket) = &mut self.server_stream {
            let mut s = socket.try_clone()?;
            let pingreq_packet = Pingreq::new(); //Usar new una vez mergeado
            println!("CLIENT: Send pinreq packet");
            pingreq_packet.write_to(&mut s);
        }

        Ok(())
    }

    pub fn handle_disconnect(&mut self, mut disconnect: HandleDisconnect) -> io::Result<()> {
        if let Some(socket) = &mut self.server_stream {
            let mut s = socket.try_clone()?;
            let disconnect_packet = disconnect.disconnect_packet;
            println!("CLIENT: Send disconnect packet: {:?}", &disconnect_packet);
            disconnect_packet.write_to(&mut s);
        }

        Ok(())
    }

    pub fn handle_unsubscribe(&mut self, mut unsubs: HandleUnsubscribe) -> io::Result<()> {
        if let Some(socket) = &mut self.server_stream {
            let mut s = socket.try_clone()?;
            let unsubscribe_packet = unsubs.unsubscribe_packet;
            println!("CLIENT: Send unsubscribe packet: {:?}", &unsubscribe_packet);
            unsubscribe_packet.write_to(&mut s);
        }

        Ok(())
    }

    pub fn handle_subscribe(&mut self, mut subscribe: HandleSubscribe) -> io::Result<()> {
        if let Some(socket) = &mut self.server_stream {
            let mut s = socket.try_clone()?;
            let subscribe_packet = subscribe.subscribe_packet;
            println!("CLIENT: Send subscribe packet: {:?}", &subscribe_packet);
            subscribe_packet.write_to(&mut s);
        }

        Ok(())
    }

    pub fn handle_publish(&mut self, mut publish: HandlePublish) -> io::Result<()> {
        if let Some(socket) = &mut self.server_stream {
            let mut s = socket.try_clone()?;
            let publish_packet = publish.publish_packet;
            println!("CLIENT: Send publish packet: {:?}", &publish_packet);
            publish_packet.write_to(&mut s);
        }
        Ok(())
    }

    pub fn handle_conection(&mut self, mut conec: HandleConection, sender_to_window: Sender<ResponseHandlers>, keep_alive_sec: &mut u16) -> io::Result<()> {
        let address = conec.get_address();
        let mut socket = TcpStream::connect(address.clone()).unwrap();
        println!("Connecting to: {:?}", address);
        let connect_packet = Connect::new(ConnectPayload::new(conec.client_id,conec.last_will_topic, conec.last_will_msg, conec.username, conec.password),3000, conec.clean_session, conec.last_will_retain, conec.last_will_qos);
        Client::handle_response(socket.try_clone().unwrap(), sender_to_window);
        *keep_alive_sec = connect_packet.keep_alive_seconds.clone();
        println!("CLIENT: Send connect packet: {:?}", &connect_packet);
        connect_packet.write_to(&mut socket);
        self.set_server_stream(socket);
        Ok(())
    }
}
