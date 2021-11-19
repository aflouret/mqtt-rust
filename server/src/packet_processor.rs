use crate::session::Session;
use common::all_packets::connack::Connack;
use common::all_packets::connect::Connect;
use common::packet::{Packet, WritePacket};
use std::net::{TcpStream};
use std::collections::HashMap;
use std::error::Error;
use common::all_packets::publish::Publish;
use common::all_packets::puback::Puback;
use std::sync::{Mutex, MutexGuard, mpsc};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::sync::{RwLock, Arc};



pub struct PacketProcessor {
    clients: HashMap<String, Session>,
    rx: Receiver<(u32, Result<Packet,Box<dyn std::error::Error + Send>>)>,
    senders_to_c_h_writers: Arc<RwLock<HashMap<u32, Arc<Mutex<Sender<Result<Packet,Box<dyn std::error::Error + Send>>>>>>>>,
}

impl PacketProcessor {

    pub fn new(rx: Receiver<(u32, Result<Packet,Box<dyn std::error::Error + Send>>)>, senders_to_c_h_writers: Arc<RwLock<HashMap<u32, Arc<Mutex<Sender<Result<Packet,Box<dyn std::error::Error + Send>>>>>>>>,) -> PacketProcessor {
        PacketProcessor {
            clients: HashMap::<String, Session>::new(),
            rx,
            senders_to_c_h_writers,
        }
    }

    pub fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        thread::spawn(move || {
            loop {
                // TODO: sacar unwraps del thread
                let (id, packet) = self.rx.recv().unwrap();
                let senders_hash = self.senders_to_c_h_writers.read().unwrap();
                let sender = senders_hash.get(&id).unwrap();
                let sender_mutex_guard = sender.lock().unwrap();
                self.process_packet(packet.unwrap(), sender_mutex_guard).unwrap();
            }
        });
        Ok(())
    } 

    pub fn process_packet(&self, packet: Packet, sender_to_c_h_writer: MutexGuard<Sender<Result<Packet, Box<dyn Error + Send>>>>) -> Result<(), Box<dyn std::error::Error>> {
        let response_packet = match packet {
                Packet::Connect(connect_packet) => {
                    println!("Recibi el Connect (en process_pracket)");
                    //TODO: self.handle_connect_packet(connect_packet)?; //la respuesta la manda el handle_connect?
                    sender_to_c_h_writer.send(Ok(common::packet::Packet::Connack(Connack::new(false,1))));
                }
                Packet::Publish(publish_packet) => {
                    //TODO: self.handle_publish_packet(publish_packet)?;
                    sender_to_c_h_writer.send(Ok(common::packet::Packet::Puback(Puback::new(1))));
                },
                _ => { return Err("Invalid packet".into()) },
            };
        Ok(())
    }
    
    pub fn handle_publish_packet(&mut self, publish_packet: Publish) -> Result<(), Box<dyn std::error::Error>> {
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
    }
    
    pub fn handle_connect_packet(&mut self, connect_packet: Connect, client_stream: TcpStream) -> Result<&Session, Box<dyn std::error::Error>> {
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
            let new_session = Session::new(client_stream, connect_packet)?;
            self.clients.insert(new_session.get_client_id().to_string(), new_session);
        }
        let current_session = self.clients.get_mut(&client_id).unwrap(); //TODO: sacar unwrap
        current_session.connect();
    
        // Enviamos el connack con 0 return code y el correspondiente flag de session_present:
        // si hay clean_session, session_present debe ser false. Sino, depende de si ya teníamos sesión
        let session_present;
        if clean_session { session_present = false; }
        else { session_present = exists_previous_session; } // TODO: revisar esto, línea 683 pdf
        
        let connack_packet = Connack::new(session_present, 0);
        connack_packet.write_to(current_session.get_socket())?;
        println!("Se envió el connack packet");
    
        Ok(current_session)
    }
}
