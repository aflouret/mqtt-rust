use std::net::TcpStream;
use common::packet::{Packet};
use common::all_packets::connect::{INCORRECT_PROTOCOL_LEVEL_ERROR_MSG, INCORRECT_PROTOCOL_LEVEL_RETURN_CODE};
use common::all_packets::connack::Connack;
use std::sync::{Mutex, mpsc};
use std::sync::mpsc::{Receiver, Sender, SendError};
use std::thread::{self, JoinHandle};
use std::sync::{RwLock, Arc};
use std::collections::HashMap;
use std::time::Duration;

const SOCKET_DISCONNECT_ERROR_MSG: &str = "Disconnectiong Socket due to Error";

pub struct ClientHandler {
    id: u32,
    stream: Option<TcpStream>,
    sender: Option<Sender<(u32, Result<Packet, Box<dyn std::error::Error + Send>>)>>,
    receiver: Option<Receiver<Result<Packet, Box<dyn std::error::Error + Send>>>>,
    reader_to_writer_tx: Sender<Result<Packet, Box<dyn std::error::Error + Send>>>
}

impl ClientHandler {
    pub fn new(
        id: u32,
        stream: TcpStream,
        senders_to_c_h_writers: Arc<RwLock<HashMap<u32, Arc<Mutex<Sender<Result<Packet, Box<dyn std::error::Error + Send>>>>>>>>,
        sender: Sender<(u32, Result<Packet, Box<dyn std::error::Error + Send>>)>,
    ) -> ClientHandler {
        let (server_tx, c_h_writer_rx) = mpsc::channel::<Result<Packet, Box<dyn std::error::Error + Send>>>();
        let sender_from_c_h_reader_to_c_h_w = server_tx.clone();
        let mut hash = senders_to_c_h_writers.write().unwrap();
        hash.insert(id, Arc::new(Mutex::new(server_tx)));

        ClientHandler {
            id,
            stream: Some(stream),
            sender: Some(sender),
            receiver: Some(c_h_writer_rx),
            reader_to_writer_tx: sender_from_c_h_reader_to_c_h_w
        }
    }

    pub fn run(mut self) -> Result<JoinHandle<()>, Box<dyn std::error::Error>> {
        let stream = self.stream.take().unwrap();
        //stream shutdown

        let receiver = self.receiver.take().unwrap();
        let sender = self.sender.take().unwrap();

        let mut client_handler_writer = ClientHandlerWriter::new(stream.try_clone()? , receiver);
        let mut client_handler_reader = ClientHandlerReader::new(self.id, stream, sender, self.reader_to_writer_tx.clone());

        let writer_join_handle = thread::spawn(move || {

            let reader_join_handle = thread::spawn(move || {
                loop {
                    if let Err(_) = client_handler_reader.receive_packet() {
                        println!("se elimina el reader");
                        break;
                    }
                }
            });

            loop {
                if let Err(_) = client_handler_writer.send_packet() {
                    println!("se elimina el writer");
                    break;
                }
            }

            reader_join_handle.join().unwrap();
            println!("client handler {} destroyed", self.id);
        });

        Ok(writer_join_handle)
    }
}


//LEE EN CHANNEL, ESCRIBE EN SOCKET
struct ClientHandlerWriter {
    //Maneja la conexion del socket
    socket: TcpStream,
    receiver: Receiver<Result<Packet, Box<dyn std::error::Error + Send>>>, //Por acá recibe los paquetes que escribe en el socket
}

impl ClientHandlerWriter {
    pub fn new(socket: TcpStream, receiver: Receiver<Result<Packet, Box<dyn std::error::Error + Send>>>) -> ClientHandlerWriter {
        ClientHandlerWriter {
            socket,
            receiver,
        }
    }

    pub fn send_packet(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(packet) = self.receiver.recv()? {
            //if let Ok(packet) = packet {
            packet.write_to(&mut self.socket)?;
            //}
            Ok(())
        } else {
            self.socket.shutdown(std::net::Shutdown::Write).unwrap();
            Err("No se pudo enviar el packet".into())
        }
    }
}

//LEE DE SOCKET, ESCRIBE EN CHANNEL
struct ClientHandlerReader {
    id: u32,
    socket: TcpStream,
    sender: Sender<(u32, Result<Packet, Box<dyn std::error::Error + Send>>)>,//Por acá manda paquetes al sv
    already_connected: bool,
    reader_to_writer_tx: Sender<Result<Packet, Box<dyn std::error::Error + Send>>>
}

impl ClientHandlerReader {
    pub fn new(id: u32, 
                socket: TcpStream, 
                sender: Sender<(u32, Result<Packet, Box<dyn std::error::Error + Send>>)>, 
                reader_to_writer_tx: Sender<Result<Packet, Box<dyn std::error::Error + Send>>>) 
                    -> ClientHandlerReader {
        socket.set_read_timeout(Some(Duration::new(5,0))).unwrap();
        ClientHandlerReader {
            id,
            socket,
            sender,
            already_connected: false,
            reader_to_writer_tx
        }
    }

    pub fn receive_packet(&mut self) -> Result<(), Box<dyn std::error::Error + Send>> {
        match Packet::read_from(&mut self.socket) {
            Ok(packet) => {
                // [MQTT-3.1.0-2]: Si es un Connect y ya había recibido un Connect antes, es un PROTOCOL VIOLATION: desconecto al client
                if let Packet::Connect(connect) = &packet {
                    if self.already_connected {
                        println!("PROTOCOL VIOLATION: Connect packet received twice");
                        return Err(Box::new(SendError("PROTOCOL VIOLATION: Connect packet received twice")));
                    }

                    //If the Keep Alive value is non-zero and the Server does not receive a Control Packet from the Client
                    //within one and a half times the Keep Alive time period, it MUST disconnect the Network Connection to the Client as if the network had failed
                    let mut keep_alive = connect.keep_alive_seconds as u64;
                    if keep_alive != 0 {
                        keep_alive *= 2;
                        self.socket.set_read_timeout(Some(Duration::new(keep_alive, 0))).unwrap();
                    } else {
                        self.socket.set_read_timeout(None).unwrap();
                    }
                    
                }
                if let Err(error) = self.sender.send((self.id, Ok(packet))) {
                    return Err(Box::new(error));
                }
            },
            Err(error) => {
                if error.to_string() == INCORRECT_PROTOCOL_LEVEL_ERROR_MSG.to_string() {
                    println!("{}", error.to_string());
                    // [MQTT-3.1.2-2]. Enviamos un connack con 0x1 y desconectamos. 
                    // [MQTT-3.2.2-4]. Por eso session_present = false
                    let connack = Connack::new(false, 
                        INCORRECT_PROTOCOL_LEVEL_RETURN_CODE);
                    self.reader_to_writer_tx.send(Ok(Packet::Connack(connack))).unwrap();
                
                    std::thread::sleep(std::time::Duration::from_millis(1000));
                }
                
                self.sender.send((self.id, Err(Box::new(SendError(SOCKET_DISCONNECT_ERROR_MSG))) )).unwrap();
                return Err(Box::new(SendError(SOCKET_DISCONNECT_ERROR_MSG)));
            },

        }

        Ok(())
    }
}
