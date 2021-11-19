use std::net::TcpStream;
use common::all_packets::connect::Connect;
use crate::session::Session;
use std::sync::mpsc::{Sender, Receiver};
use common::packet::{Packet, ReadPacket, WritePacket};
use std::io::{Read, Write};
use common::parser;

//LEE EN CHANNEL, ESCRIBE EN SOCKET
pub struct ClientHandlerWriter {
    //Maneja la conexion del socket
    id: u32,
    socket: TcpStream,
    //TODO:Option<Tcp> -> CONSULTAR:no iria porque  si se desconecta se destruye el client_handler
    receiver: Receiver<Result<Packet,Box<dyn std::error::Error + Send>>>, //Por acá recibe los paquetes que escribe en el socket
}

impl ClientHandlerWriter {
    pub fn new(id: u32, socket: TcpStream, receiver: Receiver<Result<Packet,Box<dyn std::error::Error + Send>>>) -> ClientHandlerWriter {
        ClientHandlerWriter {
            id,
            socket,
            receiver,
        }
    }

    pub fn send_packet(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let received = self.receiver.recv()?;

        /* TODO: los packets, al ser polimorficos, deberiamos poder hacer received.write_to(socket)
             en vez de hacer el match. Algo asi: (hay que hacer algunas cositas)

        if let Ok(packet) = self.receiver.recv() {
            packet.write_to(socket)?;
        }
        */

        match received {
            Ok(Packet::Connack(connack)) => {
                println!("Se manda el connack...");
                connack.write_to(&mut self.socket)?;
            }

            Ok(Packet::Puback(puback)) => {
                println!("Se manda el puback...");
                puback.write_to(&mut self.socket)?;
            }

            //...

            _ => println!("Packet desconocido")
        }

        Ok(())
    }
}

//LEE DE SOCKET, ESCRIBE EN CHANNEL
pub struct ClientHandlerReader {
    //Maneja la conexion del socket
    id: u32,
    socket: TcpStream,
    //TODO:Option<Tcp> -> CONSULTAR:no iria porque  si se desconecta se destruye el client_handler
    sender: Sender<(u32, Result<Packet,Box<dyn std::error::Error + Send>>)>,//Por acá manda paquetes al sv
}

impl ClientHandlerReader {
    pub fn new(id: u32, socket: TcpStream, sender: Sender<(u32, Result<Packet,Box<dyn std::error::Error + Send>>)>) -> ClientHandlerReader {
        ClientHandlerReader {
            id,
            socket,
            sender,
        }
    }

    pub fn receive_packet(&mut self) -> Result<(), Box<dyn std::error::Error + Send>>{
            if let Ok(packet) = parser::read_packet(&mut self.socket) {
                
                if let Err(error) = self.sender.send((self.id, Ok(packet))) {
                    return Err(Box::new(error));
                }
            }
        Ok(())
    }

/*        let packet = parser::read_packet(&mut self.socket);
        // mandar tupla (id, packet)
        self.sender.send((self.id, packet))?;

        Ok(())*/

}