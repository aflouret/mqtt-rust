use crate::config::Config;
use common::packet::{Packet, WritePacket};
use std::io;
use std::net::{TcpListener, TcpStream};
use std::collections::HashMap;
use crate::client_handler::{ClientHandlerReader, ClientHandlerWriter};
use crate::packet_processor::{PacketProcessor};

use std::sync::{Mutex, MutexGuard, mpsc};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::sync::{RwLock, Arc};


pub struct Server {
    config: Config,
}
impl Server {
    pub fn new(config: Config) -> io::Result<Self> {
        Ok(Self { config })
    }

    pub fn server_run(self) -> Result<(), Box<dyn std::error::Error>> {
        //Inicializacion
        let address = self.config.get_address() + &*self.config.get_port();
        let listener = TcpListener::bind(&address)?;
        println!("Servidor escuchando en: {} ", &address);

        let senders_to_c_h_writers = Arc::new(RwLock::new(HashMap::<u32, Arc<Mutex<Sender<Result<Packet,Box<dyn std::error::Error + Send>>>>>>::new()));
        let (c_h_reader_tx, server_rx) = mpsc::channel::<(u32, Result<Packet,Box<dyn std::error::Error + Send>>)>();

        let packet_processor = PacketProcessor::new(server_rx, senders_to_c_h_writers.clone());
        packet_processor.run();

        self.handle_connections(listener, senders_to_c_h_writers, c_h_reader_tx);
        
        Ok(())
    }

    fn handle_connections(&self, listener: TcpListener, senders_to_c_h_writers:  Arc<RwLock<HashMap<u32, Arc<Mutex<Sender<Result<Packet,Box<dyn std::error::Error + Send>>>>>>>>, c_h_reader_tx: Sender<(u32, Result<Packet,Box<dyn std::error::Error + Send>>)>) {
        let mut id: u32 = 0;
        for stream in listener.incoming() {
            if let Ok(client_stream) = stream {
                let (server_tx, c_h_writer_rx) = mpsc::channel::<Result<Packet,Box<dyn std::error::Error + Send>>>();
                let mut hash = senders_to_c_h_writers.write().unwrap();
                hash.insert(id, Arc::new(Mutex::new(server_tx)));
                id += 1;

                let mut client_handler_writer = ClientHandlerWriter::new(id, client_stream.try_clone().unwrap(), c_h_writer_rx);
                let mut client_handler_reader = ClientHandlerReader::new(id, client_stream, c_h_reader_tx.clone());

                thread::spawn(move || {

                    let (error_tx, error_rx) = mpsc::channel();
    
                    let handler_reader = thread::spawn(move || {
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
    
                    handler_reader.join().unwrap();
                });
            }
        }
    }
    
}