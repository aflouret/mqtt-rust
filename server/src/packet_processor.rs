use crate::session::Session;
use common::all_packets::connack::Connack;
use common::all_packets::connect::Connect;
use common::packet::{Packet, WritePacket};
use std::net::{TcpStream};
use std::collections::HashMap;
use std::error::Error;
use common::all_packets::publish::Publish;
use common::all_packets::puback::Puback;
use std::sync::{Mutex};
use std::sync::mpsc::{Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::sync::{RwLock, Arc};
use common::logging::logger::{Logger, LogMessage};


pub struct PacketProcessor {
    clients: HashMap<String, Session>,
    rx: Receiver<(u32, Result<Packet,Box<dyn std::error::Error + Send>>)>,
    senders_to_c_h_writers: Arc<RwLock<HashMap<u32, Arc<Mutex<Sender<Result<Packet,Box<dyn std::error::Error + Send>>>>>>>>,
    logger: Arc<Logger>,
}

impl PacketProcessor {

    pub fn new(rx: Receiver<(u32, Result<Packet,Box<dyn std::error::Error + Send>>)>,
               senders_to_c_h_writers: Arc<RwLock<HashMap<u32, Arc<Mutex<Sender<Result<Packet,Box<dyn std::error::Error + Send>>>>>>>>,
               logger: Arc<Logger>) -> PacketProcessor {
        PacketProcessor {
            clients: HashMap::<String, Session>::new(),
            rx,
            senders_to_c_h_writers,
            logger,
        }
    }

    pub fn run(mut self) -> JoinHandle<()> {
        let join_handle = thread::spawn(move || {
            loop {
                // TODO: sacar unwraps del thread
                if let Ok((id, packet)) = self.rx.recv() {
                    if let Ok(packet) = packet {
                        self.process_packet(packet, id).unwrap();
                    }
                    // (TODO) else: se desconecto el handler, hay que avisarle a la sesion
                } else {
                    break;
                }  
            }
        });
        join_handle
    } 

    pub fn process_packet(&mut self, packet: Packet, id: u32) -> Result<(), Box<dyn std::error::Error>> {
        let response_packet = match packet {
                Packet::Connect(connect_packet) => {
                    self.logger.log_msg(LogMessage::new("Connect Packet received from:".to_string(),id.to_string()));
                    println!("Recibi el Connect (en process_pracket)");
                    let connack_packet = self.handle_connect_packet(connect_packet, id)?;
                    Ok(Packet::Connack(connack_packet))
                }

                Packet::Publish(publish_packet) => {
                    self.logger.log_msg(LogMessage::new("Publish Packet received from:".to_string(),id.to_string()));
                    //TODO: self.handle_publish_packet(publish_packet)?;
                    let puback_packet = Puback::new(1);
                    Ok(Packet::Puback(puback_packet))
                },

                _ => { return Err("Invalid packet".into()) },
            };

        let senders_hash = self.senders_to_c_h_writers.read().unwrap();
        let sender = senders_hash.get(&id).unwrap();
        let sender_mutex_guard = sender.lock().unwrap();
        sender_mutex_guard.send(response_packet).unwrap();
        
        Ok(())
    }
    
    /*pub fn handle_publish_packet(&mut self, publish_packet: Publish) -> Result<(), Box<dyn std::error::Error>> {
        println!("Se recibió el publish packet");
        //Sacamos el packet_id del pubblish
        //Sacar info del publish
        //Mandamos el puback al client.
        
        let packet_id = 1 as u16;
        let puback_packet_response = Puback::new(packet_id);
        let current_session = self.clients.get_mut("u").unwrap(); //TODO: sacar unwrap
        let mut socket = current_session.get_socket().try_clone().unwrap();
        println!("{:?}",socket);
        puback_packet_response.write_to(&mut socket)?;
        println!("Se envio correctamente el PUBACK");
        Ok(())
    }*/
    
    pub fn handle_connect_packet(&mut self, connect_packet: Connect, client_handler_id: u32) -> Result<Connack, Box<dyn std::error::Error>> {
        println!("Se recibió el connect packet");
        
        let client_id = connect_packet.connect_payload.client_id.to_owned();
        let clean_session = connect_packet.clean_session;
        let exists_previous_session = self.clients.contains_key(&client_id);
    
        // Si hay un cliente con mismo client_id conectado, lo desconectamos
        if let Some(previous_session) = self.clients.get_mut(&client_id){
            previous_session.disconnect();            
        }
    
        // Si no se quiere conexión persistente o no había una sesión con mismo client_id, creamos una nueva
        // Si se quiere una conexión persistente y ya había una sesión, la retomamos
        if clean_session || ! exists_previous_session {
            let new_session = Session::new(client_handler_id, connect_packet)?;
            self.clients.insert(new_session.get_client_id().to_string(), new_session);
        }
        let current_session = self.clients.get_mut(&client_id).unwrap(); //TODO: sacar unwrap
        current_session.connect(client_handler_id);
    
        // Enviamos el connack con 0 return code y el correspondiente flag de session_present:
        // si hay clean_session, session_present debe ser false. Sino, depende de si ya teníamos sesión
        let session_present;
        if clean_session { session_present = false; }
        else { session_present = exists_previous_session; } // TODO: revisar esto, línea 683 pdf
        
        let connack_packet = Connack::new(session_present, 0);
        self.logger.log_msg(LogMessage::new("Connack packet send it to:".to_string(),client_handler_id.to_string()));
        Ok(connack_packet)
    }
}
