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
use std::io::{BufRead, BufReader};
use crate::client_controller::ClientController;

pub struct Client {
    client_id: String,
    server_stream: Option<TcpStream>,
/*    send_to_window: Sender<Packet>,*/
}

impl Client {
    //Devuelve un cliente ya conectado al address
    pub fn new(client_id: String, address: &str) -> io::Result<Self> {
        let server_stream = TcpStream::connect(address)?;
        println!("Conectándome a {:?}", &address);

        Ok(Self {
            client_id,
            server_stream: Some(server_stream),
/*            send_to_window: sender*/
            // receiver_from_ch: None
        })
    }

/*    pub fn client_run(&mut self, connect_packet: Connect,
    ) -> Result<(), Box<dyn std::error::Error>> {*/

    pub fn client_run(&mut self, recv_from_window: Receiver<Packet>, sender_to_window: Sender<String>) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(socket) = &mut self.server_stream {
            let mut server_stream_write = socket.try_clone()?;
            let mut server_stream_read = socket.try_clone()?;

/*            //Si la clonacion del socket estuvo Ok, creamos el clientHandler con el channel
            //let (client_controller_sender, receiver_n) = mpsc::channel::<Packet>();
            //receiver_from_ch = Some(receiver);
            //let client_controller = ClientController::new(client_controller_sender);*/
            let handler_from_client_controller = thread::spawn(move || {
                //if let Some(receiver_n) = &mut self.receiver_from_ch {
                    loop {
                        let packet =  recv_from_window.recv().unwrap();
                        //detectar que paquete es - process_packet cliente
                        match packet {
                            Packet::Connect(connect) => {
                                println!("{:?}",&connect);
                                connect.write_to(&mut server_stream_write)
                            },
                            Packet::Publish(publish) => {
                                println!("{:?}",&publish);
                                publish.write_to(&mut server_stream_write)
                            },
                            Packet::Subscribe(subscribe) => {
                                println!("{:?}",&subscribe);
                                subscribe.write_to(&mut server_stream_write)
                            },
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
                            sender_to_window.send("PONG".to_string());
                        },
                        Packet::Puback(publish) => {
                            println!("Client: Connack packet successfull received");
                        },
                        Packet::Suback(subscribe) => {
                            println!("Client: Connack packet successfull received");
                        },
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


/*
    pub fn client_run_window(&mut self, sender: glib::Sender<String>) {
        if let Some(socket) = &mut self.server_stream {
            println!("ENTRO A RUN WID");
            let mut sck = socket.try_clone().unwrap();
            thread::spawn(move || {
                let reader = BufReader::new(sck);
                for line in reader.lines() {
                    println!("LINE {:?}:",line);
                    sender
                        .send(line.unwrap())
                        .expect("Couldn't send data to channel");
                }
            });
        }
    }
 */
