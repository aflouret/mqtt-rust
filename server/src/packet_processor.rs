use crate::session::Session;
use common::all_packets::connack::Connack;
use common::all_packets::connect::Connect;
use common::all_packets::unsuback::Unsuback;
use common::all_packets::unsubscribe::Unsubscribe;
use common::packet::{Packet, Qos};
use std::collections::HashMap;
use common::all_packets::publish::{Publish, PublishFlags};
use common::all_packets::puback::Puback;
use std::sync::{Mutex};
use std::sync::mpsc::{Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::sync::{RwLock, Arc};
use common::logging::logger::{Logger, LogMessage};
use common::all_packets::suback::{Suback, SubackReturnCode};
use common::all_packets::subscribe::Subscribe;

pub struct Message {
    pub message: String,
    pub qos: Qos
}

pub struct PacketProcessor {
    clients: HashMap<String, Session>,
    rx: Receiver<(u32, Result<Packet,Box<dyn std::error::Error + Send>>)>,
    senders_to_c_h_writers: Arc<RwLock<HashMap<u32, Arc<Mutex<Sender<Result<Packet,Box<dyn std::error::Error + Send>>>>>>>>,
    logger: Arc<Logger>,
    retained_messages: HashMap<String, Message>,
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
            retained_messages: HashMap::<String, Message>::new(),
        }
    }

    pub fn run(mut self) -> JoinHandle<()> {
        let join_handle = thread::spawn(move || {
            loop {
                // TODO: sacar unwraps del thread
                if let Ok((c_h_id, packet)) = self.rx.recv() {
                    
                    match packet {
                        Ok(packet) => {
                            if let Err(_) = self.process_packet(packet, c_h_id) {
                                self.handle_disconnect_error(c_h_id);
                            }
                        },
                        Err(_) => self.handle_disconnect_error(c_h_id),
                    }

                } else {
                    break;
                }  
            }
        });
        join_handle
    } 

    pub fn handle_disconnect_error(&mut self, c_h_id: u32) {
        // La session que tenía dicho c_h_id y era clean, debe eliminarse
        self.clients.retain(|_, session| 
            ! (session.get_client_handler_id() == Some(c_h_id) && session.is_clean_session));
        
        // Si es que no era clean, la desconectamos del c_h_id para que la próxima vez que se conecte
        // el mismo cliente, se use el c_h_id del nuevo c_h
        self.clients.iter_mut()
            .filter(|(_, session)| session.get_client_handler_id() == Some(c_h_id))
            .for_each(|(_,session)| session.disconnect());
        
        // Eliminamos el sender al c_h del hash ya que se va a dropear ese c_h
        let mut senders_hash = self.senders_to_c_h_writers.write().unwrap();
        senders_hash.remove(&c_h_id);
        // Cuando eliminamos el sender, el receiver del c_h_w devuelve un error     
    }

    pub fn process_packet(&mut self, packet: Packet, c_h_id: u32) -> Result<(), Box<dyn std::error::Error>> {
        let response_packet = match packet {
                Packet::Connect(connect_packet) => {
                    self.logger.log_msg(LogMessage::new("Connect Packet received from:".to_string(),c_h_id.to_string()));
                    println!("Recibi el Connect (en process_pracket)");
                    let connack_packet = self.handle_connect_packet(connect_packet, c_h_id)?;
                    Some(Ok(Packet::Connack(connack_packet)))
                }

                Packet::Publish(publish_packet) => {
                    self.logger.log_msg(LogMessage::new("Publish Packet received from:".to_string(),c_h_id.to_string()));
                    let puback_packet = self.handle_publish_packet(publish_packet, c_h_id)?;
                    if let Some(puback_packet) = puback_packet {
                        Some(Ok(Packet::Puback(puback_packet)))
                    } else {
                        None
                    }
                },

                Packet::Subscribe(subscribe_packet) => {
                    self.logger.log_msg(LogMessage::new("Subscribe Packet received from:".to_string(),c_h_id.to_string()));
                    let suback_packet = self.handle_subscribe_packet(subscribe_packet, c_h_id)?;
                    Some(Ok(Packet::Suback(suback_packet)))
                },

                _ => { return Err("Invalid packet".into()) },
            };
        
        if let Some(response_packet) = response_packet{
            let senders_hash = self.senders_to_c_h_writers.read().unwrap();
            let sender = senders_hash.get(&c_h_id).unwrap();
            let sender_mutex_guard = sender.lock().unwrap();
            sender_mutex_guard.send(response_packet).unwrap();
        }
        
        Ok(())
    }

    pub fn handle_unsubscribe_packet(&mut self, unsubscribe_packet: Unsubscribe, c_h_id: u32) -> Result<Unsuback, Box<dyn std::error::Error>> {
        println!("Se recibió el subscribe packet");
    
        let unsuback_packet = Unsuback::new(unsubscribe_packet.packet_id);
        for subscription in unsubscribe_packet.topics {
            for (_, session) in &mut self.clients {
                if let Some(client_handler_id)  = session.get_client_handler_id() {
                    if client_handler_id == c_h_id {
                        session.remove_subscription(subscription);
                        break;
                    }
                } 
            }
        }
        
        Ok(unsuback_packet)
        
    }

    pub fn handle_subscribe_packet(&mut self, subscribe_packet: Subscribe, c_h_id: u32) -> Result<Suback, Box<dyn std::error::Error>> {
        println!("Se recibió el subscribe packet");
    


        let mut suback_packet = Suback::new(subscribe_packet.packet_id);
        for subscription in subscribe_packet.subscriptions {
            for (_, session) in &mut self.clients {
                if let Some(client_handler_id)  = session.get_client_handler_id() {
                    if client_handler_id == c_h_id {
                        /*if subscription_is_valid() true == false {
                            let return_code = SubackReturnCode::Failure;
                            suback_packet.add_return_code(return_code);
                        } else {
                            let return_code = match subscription.max_qos {
                                Qos::AtMostOnce => SubackReturnCode::SuccessAtMostOnce,
                                _ => SubackReturnCode::SuccessAtLeastOnce,
                            };
                            session.add_subscription(subscription.clone());
                            suback_packet.add_return_code(return_code)
                        }
                        */

                        //Retain Logic Subscribe
                        if self.retained_messages.contains_key(&subscription.topic_filter) {
                            if let Some(message) = self.retained_messages.get(&subscription.topic_filter) {
                                //Send publish al cliente con el mensaje en el retained_messages
                                let publish_packet = Publish::new(
                                    PublishFlags::new(0b0011_0011),
                                    subscription.topic_filter,
                                    None, 
                                    message.message.clone(),
                                );
                                
                                let senders_hash = self.senders_to_c_h_writers.read().unwrap();
                                let sender = senders_hash.get(&client_handler_id).unwrap();
                                let sender_mutex_guard = sender.lock().unwrap();
                                sender_mutex_guard.send(Ok(Packet::Publish(publish_packet))).unwrap();
                            }
                        };

                        break;
                    }
                } 
            }
        }
        
        Ok(suback_packet)
        
    }
    
    pub fn handle_publish_packet(&mut self, publish_packet: Publish, c_h_id: u32) -> Result<Option<Puback>, Box<dyn std::error::Error>> {
        println!("Se recibió el publish packet");
        //Sacamos el packet_id del pubblish
        //Sacar info del publish
        //Mandamos el puback al client.
        
        let packet_id = 1 as u16;
        let puback_packet_response = Puback::new(packet_id);
        let current_session = self.clients.get_mut("u").unwrap(); //TODO: sacar unwrap
        let topic_name = &publish_packet.topic_name;

        //Retain Logic Publish
        if publish_packet.flags.retain {
            self.retained_messages.insert(topic_name.clone(), Message { 
                message: publish_packet.application_message.clone(), 
                qos: publish_packet.flags.qos_level 
            });
        }

        let publish_send = publish_packet.clone();

        for (c_h_id, session) in &self.clients {
            if session.is_subscribed_to(&topic_name) {
                if let Some(client_handler_id) = session.get_client_handler_id() {
                    let senders_hash = self.senders_to_c_h_writers.read().unwrap();
                    let sender = senders_hash.get(&client_handler_id).unwrap();
                    let sender_mutex_guard = sender.lock().unwrap();
                    sender_mutex_guard.send(Ok(Packet::Publish(publish_send.clone()))).unwrap();
                }
            }
        }
        println!("Se envio correctamente el PUBACK");

        if publish_packet.flags.qos_level == Qos::AtMostOnce {
            Ok(None)
        } else {
            Ok(Some(Puback::new(publish_packet.packet_id.unwrap())))
        } 
    }
    
    pub fn handle_connect_packet(&mut self, connect_packet: Connect, client_handler_id: u32) -> Result<Connack, Box<dyn std::error::Error>> {
        let client_id = connect_packet.connect_payload.client_id.to_owned();
        let clean_session = connect_packet.clean_session;
        let exists_previous_session = self.clients.contains_key(&client_id);
    
        // Si hay un cliente con mismo client_id conectado, lo desconectamos
        if let Some(previous_session) = self.clients.get(&client_id){
            if previous_session.is_active() {
                let previous_handler_id = previous_session.get_client_handler_id().unwrap();
                self.handle_disconnect_error(previous_handler_id);
                println!("El cliente ya estaba conectado");
                //return Err("El cliente ya estaba conectado".into())

            }
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
