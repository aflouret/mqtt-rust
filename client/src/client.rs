use std::convert::TryInto;
use common::all_packets::connect::Connect;
use common::all_packets::connack::Connack;
use common::all_packets::publish::{Publish, PublishFlags};
use std::time::Duration;

use common::packet::Packet;
use common::packet::WritePacket;
use common::parser;
use std::net::TcpStream;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::{io, thread};
use std::io::{BufRead, BufReader, Read};
use crate::client::ClientStatus::{StatusOff, StatusOn};
use crate::client_controller::ClientController;
use crate::client_processor::ClientProcessor;
use crate::handlers::{EventHandlers, HandlePublish, HandleSubscribe};
use crate::HandleConection;

#[derive(Debug)]
enum ClientStatus {
    StatusOn,
    StatusOff,
}

#[derive(Debug)]
pub struct Client {
    client_id: String,
    server_stream: Option<TcpStream>,
    client_status: bool,
    /*    recv: Receiver<EventHandlers>,*/
    /*    send_to_window: Sender<Packet>,*/
}

impl Client {
    //Devuelve un cliente ya conectado al address
    pub fn new(client_id: String, address: &str) -> Client {
        //let server_stream = TcpStream::connect(address)?;
        //println!("Conectándome a {:?}", &address);

        Client {
            client_id,
            server_stream: None,
            client_status: false,
            /*            recv,*/
            /*            send_to_window: sender*/
            // receiver_from_ch: None
        }
    }

    pub fn set_server_stream(&mut self, stream: TcpStream) {
        self.server_stream = Some(stream);
    }

    pub fn start_client(mut self, recv_conection: Receiver<EventHandlers>, sender_to_window: Sender<String>) -> Result<(), Box<dyn std::error::Error>> {
        thread::spawn(move || {
            
            if let Some(socket) = &mut self.server_stream {
                let mut socket_reader = socket.try_clone().unwrap();
                let handler_read = thread::spawn(move || {
                    loop {
                        let receiver_packet = Packet::read_from(&mut socket_reader).unwrap();
                        match receiver_packet {
                            Packet::Connack(connect) => {
                                println!("Client: Connack packet successfull received");
                                sender_to_window.send("PONG".to_string());
                            }
                            Packet::Puback(publish) => {
                                println!("Client: Connack packet successfull received");
                            }
                            Packet::Suback(subscribe) => {
                                println!("Client: Connack packet successfull received");
                            }
                            _ => (),
                        };
                    }
                });
            }

            loop {
                    if let Ok(conection) = recv_conection.recv() {
                        match conection {
                            EventHandlers::HandleConection(conec) => {
                                self.handle_conection(conec).unwrap();
                                println!("ClientConectado");
                                //self.client_status = false;
                            },
                            EventHandlers::HandlePublish(publish) => {
                                println!("Entro a publish conn");
                                //Client::handle_publish(&mut self.server_stream, publish).unwrap();
                                self.handle_publish(publish).unwrap();
                            },
                            EventHandlers::HandleSubscribe(subscribe) => {
                                self.handle_subscribe(subscribe).unwrap();
                            }
                            _ => ()
                        };
                    }
                }
        });

        Ok(())
    }

    pub fn handle_subscribe(&mut self, mut subscribe: HandleSubscribe) -> io::Result<()> {
        println!("{:?}", subscribe);
        if let Some(socket) =  &mut self.server_stream {
            let mut s = socket.try_clone()?;
            let subscribe_packet = subscribe.subscribe_packet;
            println!("Envio subscribe packet: {:?}", &subscribe_packet);
            subscribe_packet.write_to(&mut s);
            println!("SOCKET en SUBS: {:?}", &s);
        }

        Ok(())
    }

    pub fn handle_publish(&mut self, mut publish: HandlePublish) -> io::Result<()>  {
        println!("{:?}", publish);
        if let Some(socket) =  &mut self.server_stream {
            let mut s = socket.try_clone()?;
            let publish_packet = publish.publish_packet;
            println!("Envio publish packet: {:?}", &publish_packet);
            publish_packet.write_to(&mut s);
            println!("SOCKET en PUBLI: {:?}", &s);
        }
        Ok(())
    }

    pub fn handle_conection(&mut self, mut conec: HandleConection) -> io::Result<()> {
        println!("{:?}", conec);
        let address = conec.get_address();
        let mut socket = TcpStream::connect(address.clone()).unwrap();
        println!("Conectándome a {:?}", address);
        let connect_packet = conec.connect_packet;
        connect_packet.write_to(&mut socket);
        self.set_server_stream(socket);
        Ok(())
    }

    //, recv_from_window: Receiver<Packet>
    pub fn client_run(&mut self, recv: Receiver<Packet>) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(socket) = &mut self.server_stream {
            let mut server_stream_write = socket.try_clone()?;
            let mut server_stream_read = socket.try_clone()?;

            let handler_from_client_controller = thread::spawn(move || {
                loop {
                    let packet = recv.recv().unwrap();
                    //detectar que paquete es - process_packet cliente
                    match packet {
                        Packet::Connect(connect) => {
                            println!("{:?}", &connect);
                            connect.write_to(&mut server_stream_write)
                        }
                        Packet::Publish(publish) => {
                            println!("{:?}", &publish);
                            publish.write_to(&mut server_stream_write)
                        }
                        Packet::Subscribe(subscribe) => {
                            println!("{:?}", &subscribe);
                            subscribe.write_to(&mut server_stream_write)
                        }
                        _ => Err("Invalid packet".into()),
                    };
                }
                //}
            });

            //Lectura
            let handler_read = thread::spawn(move || {
                loop {
                    let receiver_packet = Packet::read_from(&mut server_stream_read).unwrap();
                    match receiver_packet {
                        Packet::Connack(connect) => {
                            println!("Client: Connack packet successfull received");
                            //sender_to_window.send("PONG".to_string());
                        }
                        Packet::Puback(publish) => {
                            println!("Client: Connack packet successfull received");
                        }
                        Packet::Suback(subscribe) => {
                            println!("Client: Connack packet successfull received");
                        }
                        _ => (),
                    };
                    //&self.send_to_window.send(receiver_packet);
                    //REcibimos la respuesta mandarsela por channel o por lo que fuera al clienthandler,
                    //para que se la muestra a la interfaz grafica
                }
            });

            handler_read.join().unwrap();
            handler_from_client_controller.join().unwrap();
        }

        Ok(())
    }
}
