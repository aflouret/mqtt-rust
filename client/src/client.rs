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
use std::sync::mpsc::Receiver;
use std::thread;
use crate::client_controller::ClientController;

pub struct Client {
    client_id: String,
    server_stream: Option<TcpStream>
//    receiver_from_ch: Option<Receiver<Packet>>
}

impl Client {
    //Devuelve un cliente ya conectado al address
    pub fn new(client_id: String, address: String) -> Result<Client, Box<dyn std::error::Error>> {
        let server_stream = TcpStream::connect(&address)?;
        println!("Conectándome a {:?}", &address);

        Ok(Client {
            client_id,
            server_stream: Some(server_stream),
           // receiver_from_ch: None
        })
    }

    pub fn client_run(&mut self, connect_packet: Connect,
    ) -> Result<(), Box<dyn std::error::Error>> {

        // handle connection,
        if let Some(socket_writer) =  &mut self.server_stream {
            connect_packet.write_to(socket_writer)?;
        }
        println!("Se envió el connect packet");
        if let Some(socket_reader) = &mut self.server_stream {
            let received_packet = parser::read_packet(socket_reader)?;
            if let Packet::Connack(_connack_packet) = received_packet {
                println!("Se recibió el connack packet");
            }
        }

        if let Some(socket) = &mut self.server_stream {
            let mut server_stream_write = socket.try_clone()?;
            let mut server_stream_read = socket.try_clone()?;

            //Si la clonacion del socket estuvo Ok, creamos el clientHandler con el channel
            let (client_controller_sender, receiver_n) = mpsc::channel::<Packet>();
            //receiver_from_ch = Some(receiver);
            let client_controller = ClientController::new(client_controller_sender);
            let handler_from_client_controller = thread::spawn(move || {
                //if let Some(receiver_n) = &mut self.receiver_from_ch {
                    loop {
                        let packet = receiver_n.recv().unwrap();
                        //detectar que paquete es - process_packet cliente
                        match packet {
                            Packet::Connect(connect) => connect.write_to(&mut server_stream_write),
                            Packet::Publish(publish) => publish.write_to(&mut server_stream_write),
                            _ => Err("Invalid packet".into()),
                        };
                    }
                //}
            });

            //Lectura
            let handler_read = thread::spawn(move || {
                loop {
                    let receiver_packet = parser::read_packet(&mut server_stream_read).unwrap();
                    if let Packet::Puback(_puback_packet) = receiver_packet {
                        println!("Thread-Lectura: Se recibió el puback packet");
                    }
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
