use std::io;
use std::net::TcpStream;
use std::sync::mpsc::{Receiver, Sender};
use common::packet::Packet;

pub struct ClientProcessor {
    socket: TcpStream,

}


impl ClientProcessor {
    pub fn new(socket: TcpStream) -> ClientProcessor {
        ClientProcessor{socket}
    }

    pub fn run(self, recv: Receiver<Packet>) -> io::Result<()>{



        Ok(())
    }
}