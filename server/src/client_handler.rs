use std::net::TcpStream;
use common::packet::{Packet, WritePacket};
use common::parser;
use std::sync::{Mutex, mpsc};
use std::sync::mpsc::{Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::sync::{RwLock, Arc};
use std::collections::HashMap;

pub struct ClientHandler {
    id: u32,
    stream: Option<TcpStream>,
    sender: Option<Sender<(u32, Result<Packet,Box<dyn std::error::Error + Send>>)>>,
    receiver: Option<Receiver<Result<Packet,Box<dyn std::error::Error + Send>>>>,
}

impl ClientHandler {
    pub fn new(
        id: u32, 
        stream: TcpStream, 
        senders_to_c_h_writers: Arc<RwLock<HashMap<u32, Arc<Mutex<Sender<Result<Packet,Box<dyn std::error::Error + Send>>>>>>>>,
        sender:Sender<(u32, Result<Packet,Box<dyn std::error::Error + Send>>)>,
    ) -> ClientHandler {
        
        let (server_tx, c_h_writer_rx) = mpsc::channel::<Result<Packet,Box<dyn std::error::Error + Send>>>();
        let mut hash = senders_to_c_h_writers.write().unwrap();
        hash.insert(id, Arc::new(Mutex::new(server_tx)));
        
        ClientHandler {
            id,
            stream: Some(stream),
            sender: Some(sender),
            receiver: Some(c_h_writer_rx),
        }
    }

    pub fn run(mut self) -> Result<JoinHandle<()>, Box<dyn std::error::Error>>{

        let stream = self.stream.take().unwrap();


        let receiver = self.receiver.take().unwrap();
        let sender = self.sender.take().unwrap();

        let mut client_handler_writer = ClientHandlerWriter::new(stream.try_clone()?, receiver);
        let mut client_handler_reader = ClientHandlerReader::new(self.id, stream, sender);

        let writer_join_handle = thread::spawn(move || {

            // channel para que el reader le avise al writer que el cliente se desconecto
            let (error_tx, error_rx) = mpsc::channel();

            let reader_join_handle = thread::spawn(move || {
                loop {
                    let result = client_handler_reader.receive_packet();
                    if result.is_err() {
                        error_tx.send(result).unwrap();
                        break;
                    }
                    error_tx.send(Ok(())).unwrap();
                }
            });
            
            loop {
                if let Ok(result) = error_rx.try_recv() {
                    if result.is_err() {
                        break;
                    }
                }
                client_handler_writer.send_packet().unwrap();
            }

            reader_join_handle.join().unwrap();
        });

        Ok(writer_join_handle)
    }
}


//LEE EN CHANNEL, ESCRIBE EN SOCKET
struct ClientHandlerWriter {
    //Maneja la conexion del socket
    socket: TcpStream,
    //TODO:Option<Tcp> -> CONSULTAR:no iria porque  si se desconecta se destruye el client_handler
    receiver: Receiver<Result<Packet,Box<dyn std::error::Error + Send>>>, //Por acá recibe los paquetes que escribe en el socket
}

impl ClientHandlerWriter {
    pub fn new(socket: TcpStream, receiver: Receiver<Result<Packet,Box<dyn std::error::Error + Send>>>) -> ClientHandlerWriter {
        ClientHandlerWriter {
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
struct ClientHandlerReader {
    id: u32,
    socket: TcpStream,
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