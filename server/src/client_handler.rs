use std::io::{Read, Write};
use std::net::TcpStream;
use common::packet::{Packet, WritePacket};
use std::sync::{Mutex, mpsc};
use std::sync::mpsc::{Receiver, Sender, SendError};
use std::thread::{self, JoinHandle};
use std::sync::{RwLock, Arc};
use std::collections::HashMap;

pub struct ClientHandler {
    id: u32,
    stream_write: Option<Box<dyn Write + Send>>,
    stream_read: Option<Box<dyn Read + Send>>,
    sender: Option<Sender<(u32, Result<Packet, Box<dyn std::error::Error + Send>>)>>,
    receiver: Option<Receiver<Result<Packet, Box<dyn std::error::Error + Send>>>>,
}

impl ClientHandler {
    pub fn new(
        id: u32,
        stream_write: Box<dyn Write + Send>,
        stream_read: Box<dyn Read + Send>,
        senders_to_c_h_writers: Arc<RwLock<HashMap<u32, Arc<Mutex<Sender<Result<Packet, Box<dyn std::error::Error + Send>>>>>>>>,
        sender: Sender<(u32, Result<Packet, Box<dyn std::error::Error + Send>>)>,
    ) -> ClientHandler {
        let (server_tx, c_h_writer_rx) = mpsc::channel::<Result<Packet, Box<dyn std::error::Error + Send>>>();
        let mut hash = senders_to_c_h_writers.write().unwrap();
        hash.insert(id, Arc::new(Mutex::new(server_tx)));

        ClientHandler {
            id,
            stream_write: Some(stream_write),
            stream_read: Some(stream_read),
            sender: Some(sender),
            receiver: Some(c_h_writer_rx),
        }
    }

    pub fn run(mut self) -> Result<JoinHandle<()>, Box<dyn std::error::Error>> {
        let stream_write = self.stream_write.take().unwrap();
        let stream_read = self.stream_read.take().unwrap();
        //stream shutdown

        let receiver = self.receiver.take().unwrap();
        let sender = self.sender.take().unwrap();

        let mut client_handler_writer = ClientHandlerWriter::new(stream_write , receiver);
        let mut client_handler_reader = ClientHandlerReader::new(self.id, stream_read, sender);

        let writer_join_handle = thread::spawn(move || {

            let reader_join_handle = thread::spawn(move || {
                loop {
                    if let Err(_) = client_handler_reader.receive_packet() {
                        //stream.shutdown() a alguno clonado y se cierran los demas.
                        break;
                    }
                }
                println!("reader destroyed");
            });

            loop {
                if let Err(_) = client_handler_writer.send_packet() {
                    break;
                }
            }
            reader_join_handle.join().unwrap();
            println!("writer destroyed");
        });

        Ok(writer_join_handle)
    }
}


//LEE EN CHANNEL, ESCRIBE EN SOCKET
struct ClientHandlerWriter {
    //Maneja la conexion del socket
    socket: Box<dyn Write + Send>,
    receiver: Receiver<Result<Packet, Box<dyn std::error::Error + Send>>>, //Por acá recibe los paquetes que escribe en el socket
}

impl ClientHandlerWriter {
    pub fn new(socket: Box<dyn Write + Send>, receiver: Receiver<Result<Packet, Box<dyn std::error::Error + Send>>>) -> ClientHandlerWriter {
        ClientHandlerWriter {
            socket,
            receiver,
        }
    }

    pub fn send_packet(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(packet) = self.receiver.recv()? {
                packet.write_to(&mut self.socket)
        } else {
            Err("No se pudo enviar el packet".into())
        }
    }
}

//LEE DE SOCKET, ESCRIBE EN CHANNEL
struct ClientHandlerReader {
    id: u32,
    socket: Box<dyn Read + Send>,
    sender: Sender<(u32, Result<Packet, Box<dyn std::error::Error + Send>>)>,//Por acá manda paquetes al sv
}

impl ClientHandlerReader {
    pub fn new(id: u32, socket: Box<dyn Read + Send>, sender: Sender<(u32, Result<Packet, Box<dyn std::error::Error + Send>>)>) -> ClientHandlerReader {
        ClientHandlerReader {
            id,
            socket,
            sender,
        }
    }

    pub fn receive_packet(&mut self) -> Result<(), Box<dyn std::error::Error + Send>> {
        match Packet::read_from(&mut self.socket) {
            Ok(packet) => if let Err(error) = self.sender.send((self.id, Ok(packet))) {
                return Err(Box::new(error));
            },
            Err(_) => {
                self.sender.send((self.id, Err(Box::new(SendError("Socket Disconnect"))) )).unwrap();
                return Err(Box::new(SendError("Socket Disconnect")))
            },

        }

        Ok(())
    }

    /*        let packet = parser::read_packet(&mut self.socket);
            // mandar tupla (id, packet)
            self.sender.send((self.id, packet))?;

            Ok(())*/
}


/*
use std::io::Read;

fn main() {
    println!("Hello, world!");
    let mut s = std::net::TcpStream::connect("localhost:8088").unwrap();
    let s_clone = s.try_clone().unwrap();
    let handler = std::thread::spawn(move || {
        let mut buf = [0u8;1];
        s.read_exact(&mut buf).unwrap();
        println!("Leí");
    });
    std::thread::sleep(std::time::Duration::from_millis(500));
    s_clone.shutdown(std::net::Shutdown::Both).unwrap();
    let res = handler.join();
    println!("{:?}",
rror { kind: UnexpectedEof, message: "failed to fill whole buffer" }
 */