use std::net::TcpStream;
use common::all_packets::connect::Connect;
use crate::session::Session;
use std::sync::mpsc::{Sender, Receiver};
use common::packet::{Packet, ReadPacket, WritePacket};
use std::io::{Read, Write};
use common::parser;

//LEE EN CHANNEL, ESCRIBE EN SOCKET
pub struct ClientHandlerWriter{
    //Maneja la conexion del socket
    id: u32,
    socket: Option<TcpStream>, //Option<Tcp>, cuando se desconecta queda en None
    receiver: Receiver<Packet>, //Por acá recibe los paquetes que escribe en el socket
}

impl ClientHandlerWriter{
    pub fn new(id: u32, socket: Option<TcpStream>, receiver: Receiver<Packet>) -> ClientHandlerWriter {

        ClientHandlerWriter {
            id,
            socket,
            receiver,
        }
    }

    pub fn send_packet(&mut self) -> Result<(), Box<dyn std::error::Error>>{
        let received = self.receiver.recv()?;
        
        match received {
            Packet::Connect(connect) => {
                if let Some(socket) = &mut self.socket{
                    connect.write_to(socket)?;
                }
            }

            //...
            
            _ => println!("hola")
        }

        Ok(())
    }
}

//LEE DE SOCKET, ESCRIBE EN CHANNEL
pub struct ClientHandlerReader{
    //Maneja la conexion del socket
    id: u32,
    socket: Option<TcpStream>,
    sender: Sender<(u32, Packet)>, //Por acá manda paquetes al sv
}

impl ClientHandlerReader{
    pub fn new(id: u32, socket: Option<TcpStream>, sender: Sender<(u32, Packet)>) -> ClientHandlerReader {
        ClientHandlerReader {
            id,
            socket,
            sender,
        }
    }

    pub fn receive_packet(&mut self) -> Result<(), Box<dyn std::error::Error>>{
        loop {
            if let Some(socket) = &mut self.socket{
                let packet = parser::read_packet(socket)?;
                // mandar tupla (id, packet)
                self.sender.send((self.id, packet))?;
            }
        }
    }
}