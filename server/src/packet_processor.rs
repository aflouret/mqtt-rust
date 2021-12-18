use crate::authenticator::Authenticator;
use crate::puback_processor::PubackProcessor;
use crate::session::Session;
use crate::topic_filters;
use common::all_packets::connack::{
    Connack, CONNACK_BAD_USERNAME_OR_PASSWORD, CONNACK_CONNECTION_ACCEPTED,
};
use common::all_packets::connect::Connect;
use common::all_packets::pingreq::Pingreq;
use common::all_packets::pingresp::Pingresp;
use common::all_packets::puback::Puback;
use common::all_packets::publish::{Publish, PublishFlags};
use common::all_packets::suback::{Suback, SubackReturnCode};
use common::all_packets::subscribe::Subscribe;
use common::all_packets::unsuback::Unsuback;
use common::all_packets::unsubscribe::Unsubscribe;
use common::logging::logger::{LogMessage, Logger};
use common::packet::{Packet, Qos};
use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::sync::mpsc::{Receiver, SendError, Sender};
use std::sync::{mpsc, Mutex};
use std::sync::{Arc, RwLock};
use std::thread::{self, JoinHandle};

const PACKETS_ID: u16 = 100;

pub struct Message {
    pub message: String,
    pub qos: Qos,
}

pub struct PacketProcessor {
    sessions: HashMap<String, Session>,
    rx: Receiver<(u32, Result<Packet, Box<dyn std::error::Error + Send>>)>,
    tx_to_puback_processor: Sender<(u32, Result<Packet, Box<dyn std::error::Error + Send>>)>,
    rx_from_packet_processor:
        Option<Receiver<(u32, Result<Packet, Box<dyn std::error::Error + Send>>)>>,
    senders_to_c_h_writers: Arc<
        RwLock<HashMap<u32, Arc<Mutex<Sender<Result<Packet, Box<dyn std::error::Error + Send>>>>>>>,
    >,
    logger: Arc<Logger>,
    retained_messages: HashMap<String, Message>,
    packets_id: HashMap<u16, bool>,
    authenticator: Authenticator, //qos_1_senders: HashMap<u16, Sender<()>>,
}

impl PacketProcessor {
    pub fn new(
        rx: Receiver<(u32, Result<Packet, Box<dyn std::error::Error + Send>>)>,
        senders_to_c_h_writers: Arc<
            RwLock<
                HashMap<u32, Arc<Mutex<Sender<Result<Packet, Box<dyn std::error::Error + Send>>>>>>,
            >,
        >,
        logger: Arc<Logger>,
    ) -> PacketProcessor {
        let (tx_to_puback_processor, rx_from_packet_processor) = mpsc::channel();
        let mut packets: HashMap<u16, bool> = HashMap::new();
        for i in 0..PACKETS_ID {
            packets.insert(i, false);
        }
        let authenticator = Authenticator::from("accounts.txt".to_string()).unwrap();
        PacketProcessor {
            sessions: HashMap::<String, Session>::new(),
            rx,
            tx_to_puback_processor,
            rx_from_packet_processor: Some(rx_from_packet_processor),
            senders_to_c_h_writers,
            logger,
            retained_messages: HashMap::<String, Message>::new(),
            packets_id: packets,
            //qos_1_senders: HashMap::<u16, Sender<()>>::new(),
            authenticator: authenticator,
        }
    }

    pub fn run(mut self) -> JoinHandle<()> {
        let join_handle = thread::spawn(move || {
            let senders_to_c_h_writers = self.senders_to_c_h_writers.clone();
            let rx_from_packet_processor = self.rx_from_packet_processor.take().unwrap();
            let puback_proc_handle = thread::spawn(move || {
                let puback_processor =
                    PubackProcessor::new(senders_to_c_h_writers, rx_from_packet_processor);
                puback_processor.run();
            });

            loop {
                if let Ok((c_h_id, packet)) = self.rx.recv() {
                    match packet {
                        Ok(packet) => {
                            if let Err(_) = self.process_packet(packet, c_h_id) {
                                self.handle_disconnect_error(c_h_id);
                            }
                        }
                        Err(_e) => {
                            self.handle_disconnect_error(c_h_id);
                        }
                    }
                } else {
                    break;
                }
            }

            puback_proc_handle.join().unwrap();
        });
        join_handle
    }

    pub fn handle_disconnect(&mut self, c_h_id: u32) {
        // La session que tenía dicho c_h_id y era clean, debe eliminarse
        self.sessions.retain(|_, session| {
            !(session.get_client_handler_id() == Some(c_h_id) && session.is_clean_session)
        });

        // Si es que no era clean, la desconectamos del c_h_id para que la próxima vez que se conecte
        // el mismo cliente, se use el c_h_id del nuevo c_h
        self.sessions
            .iter_mut()
            .filter(|(_, session)| session.get_client_handler_id() == Some(c_h_id))
            .for_each(|(_, session)| session.disconnect());

        // Eliminamos el sender al c_h del hash ya que se va a dropear ese c_h
        let mut senders_hash = self.senders_to_c_h_writers.write().unwrap();
        if let Some(sender) = senders_hash.remove(&c_h_id) {
            // Le mandamos al c_h_w que se cierre
            sender
                .lock()
                .unwrap()
                .send(Err(Box::new(SendError("Socket Disconnect"))))
                .unwrap();
            println!("se mando el error este pp, linea 114");
        }
    }

    pub fn handle_disconnect_error(&mut self, c_h_id: u32) {
        // Obtenemos la session del c_h_id. Si no existe, es porque ya se desconectó el client con Disconnect
        let session = match self
            .sessions
            .iter()
            .find(|(_id, session)| session.get_client_handler_id() == Some(c_h_id))
        {
            Some((_client_id, session)) => session,
            None => return,
        };

        println!("Session: {:?}", session);

        // Si hay last will
        if let Some(_) = session.last_will_msg {
            // Mandamos el publish con el last will msg al last will topic
            let mut p = None;
            if let Some(level) = session.last_will_qos {
                if level == Qos::AtLeastOnce {
                    if let Some(packet_id) =
                        PacketProcessor::find_key_for_value(self.packets_id.clone(), false)
                    {
                        p = Some(packet_id);
                        self.packets_id.insert(packet_id, true);
                    }
                }
            }
            /*
            if let Some(packet_id) = PacketProcessor::find_key_for_value(self.packets_id.clone(), false) {
                p = Some(packet_id);
                self.packets_id.insert(packet_id, true);
            }
            */

            // Mandamos el publish a los suscriptores
            //TODO: solo hacer si es que hay last will
            let sess = session.clone();
            let publish_packet = Publish::new(
                PublishFlags {
                    duplicate: false,
                    qos_level: sess.last_will_qos.unwrap(),
                    retain: sess.last_will_retain,
                },
                sess.last_will_topic.as_ref().unwrap().clone(),
                p,
                sess.last_will_msg.as_ref().unwrap().clone(),
            );

            print!("Voy a mandar el publish last will {:?}", publish_packet);

            self.handle_publish_packet(publish_packet).unwrap();
        }

        self.handle_disconnect(c_h_id);
    }

    pub fn process_packet(
        &mut self,
        packet: Packet,
        c_h_id: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut client_id = "No client id yet".to_string();
        if let Some(c_id) = self.get_client_id_from_handler_id(c_h_id) {
            client_id = c_id;
        }

        let response_packet = match packet {
            Packet::Connect(connect_packet) => {
                self.logger.log_msg(LogMessage::new(
                    "Connect Packet received from:".to_string(),
                    client_id.to_string(),
                ))?;
                println!("Recibi el Connect (en process_pracket)");
                let connack_packet = self.handle_connect_packet(connect_packet, c_h_id)?;
                Some(Ok(Packet::Connack(connack_packet)))
            }

            Packet::Publish(publish_packet) => {
                self.logger.log_msg(LogMessage::new(
                    "Publish Packet received from:".to_string(),
                    client_id,
                ))?;
                let puback_packet = self.handle_publish_packet(publish_packet)?;
                if let Some(puback_packet) = puback_packet {
                    Some(Ok(Packet::Puback(puback_packet)))
                } else {
                    None
                }
            }

            Packet::Puback(puback_packet) => {
                self.logger.log_msg(LogMessage::new(
                    "Puback Packet received from:".to_string(),
                    client_id.to_string(),
                ))?;
                self.handle_puback_packet(puback_packet)?;
                None
            }

            Packet::Subscribe(subscribe_packet) => {
                self.logger.log_msg(LogMessage::new(
                    "Subscribe Packet received from:".to_string(),
                    client_id.to_string(),
                ))?;
                let suback_packet = self.handle_subscribe_packet(subscribe_packet, c_h_id)?;
                Some(Ok(Packet::Suback(suback_packet)))
            }

            Packet::Unsubscribe(unsubscribe_packet) => {
                self.logger.log_msg(LogMessage::new(
                    "Unsubscribe Packet received from:".to_string(),
                    client_id.to_string(),
                ))?;
                let unsuback_packet = self.handle_unsubscribe_packet(unsubscribe_packet, c_h_id)?;
                Some(Ok(Packet::Unsuback(unsuback_packet)))
            }

            Packet::Pingreq(pingreq_packet) => {
                self.logger.log_msg(LogMessage::new(
                    "Pingreq Packet received from:".to_string(),
                    client_id.to_string(),
                ))?;
                let pingresp_packet = self.handle_pingreq_packet(pingreq_packet, c_h_id)?;
                Some(Ok(Packet::Pingresp(pingresp_packet)))
            }

            Packet::Disconnect(_disconnect_packet) => {
                self.logger.log_msg(LogMessage::new(
                    "Disconnect Packet received from:".to_string(),
                    client_id.to_string(),
                ))?;
                self.handle_disconnect(c_h_id);
                None
            }

            _ => {
                return Err("Invalid packet".into());
            }
        };

        if let Some(response_packet) = response_packet {
            //Si es un Ok(Packet::Connack(connack)) con return code != 0, se envia el connack y se procede a desconectar al cliente
            match &response_packet {
                Ok(Packet::Connack(connack)) => {
                    let conn = connack.clone();
                    self.send_packet_to_client_handler(c_h_id, response_packet)?;
                    if conn.connect_return_code != CONNACK_CONNECTION_ACCEPTED {
                        self.handle_disconnect(c_h_id);
                    } else {
                        self.send_unacknowledged_messages(c_h_id);
                    }
                }
                _ => self.send_packet_to_client_handler(c_h_id, response_packet)?,
            }
            //self.send_packet_to_client_handler(c_h_id, response_packet);
        }

        Ok(())
    }

    pub fn handle_connect_packet(
        &mut self,
        connect_packet: Connect,
        client_handler_id: u32,
    ) -> Result<Connack, Box<dyn std::error::Error>> {
        //Authentication
        if let Some(password) = &connect_packet.connect_payload.password {
            match &connect_packet.connect_payload.username {
                None => {
                    return Err(Box::new(Error::new(
                        ErrorKind::Other,
                        "Invalid Packet: Contains password but no username",
                    )))
                }
                Some(username) => {
                    if !self.authenticator.account_is_valid(&username, password) {
                        println!("Invalid Acount: sending Connack packet with error code");
                        return Ok(Connack::new(false, CONNACK_BAD_USERNAME_OR_PASSWORD));
                    }
                }
            }
        }
        println!("Valid Account");

        let client_id = connect_packet.connect_payload.client_id.to_owned();
        let clean_session = connect_packet.clean_session;
        let exists_previous_session = self.sessions.contains_key(&client_id);

        // Si hay un cliente con mismo client_id conectado, desconectamos la sesión del client anterior
        if let Some(existing_session) = self.sessions.get(&client_id) {
            println!("\n Session existente ------> {:?} \n ", existing_session);
            if existing_session.is_active() {
                let existing_handler_id = existing_session.get_client_handler_id().unwrap();
                self.handle_disconnect_error(existing_handler_id);
                self.logger.log_msg(LogMessage::new(
                    "El cliente ya estaba conectado. Se remplazó la sesión por la nueva"
                        .to_string(),
                    client_id.clone(),
                ))?;
            }
        }

        // Si no se quiere conexión persistente, o no había una sesión activa con mismo client_id, creamos una nueva
        // Si se quiere una conexión persistente y ya había una sesión, la retomamos
        if clean_session || !exists_previous_session {
            let new_session = Session::new(client_handler_id, connect_packet)?;
            self.sessions
                .insert(new_session.get_client_id().to_string(), new_session);
        }
        let current_session = self.sessions.get_mut(&client_id).unwrap();
        current_session.connect(client_handler_id);

        // Enviamos el connack con 0 return code y el correspondiente flag de session_present:
        // si hay clean_session, session_present debe ser false. Sino, depende de si ya teníamos sesión
        let session_present;
        if clean_session {
            session_present = false;
        } else {
            session_present = exists_previous_session;
        } // TODO: revisar esto, línea 683 pdf

        let connack_packet = Connack::new(session_present, 0);
        self.logger.log_msg(LogMessage::new(
            "Connack packet send it to:".to_string(),
            client_handler_id.to_string(),
        ))?;
        Ok(connack_packet)
    }

    pub fn handle_pingreq_packet(
        &mut self,
        _pingreq_packet: Pingreq,
        _c_h_id: u32,
    ) -> Result<Pingresp, Box<dyn std::error::Error>> {
        println!("Se recibió el pingreq packet");

        Ok(Pingresp::new())
    }

    pub fn handle_unsubscribe_packet(
        &mut self,
        unsubscribe_packet: Unsubscribe,
        c_h_id: u32,
    ) -> Result<Unsuback, Box<dyn std::error::Error>> {
        println!("Se recibió el unsubscribe packet");

        let client_id = self.get_client_id_from_handler_id(c_h_id);
        let session;
        if let Some(client_id) = client_id {
            session = self.sessions.get_mut(&client_id).unwrap();
        } else {
            return Err("Client not found".into());
        }

        let unsuback_packet = Unsuback::new(unsubscribe_packet.packet_id);
        for subscription in unsubscribe_packet.topics {
            session.remove_subscription(subscription);
        }

        Ok(unsuback_packet)
    }

    pub fn handle_subscribe_packet(
        &mut self,
        subscribe_packet: Subscribe,
        c_h_id: u32,
    ) -> Result<Suback, Box<dyn std::error::Error>> {
        println!("Se recibió el subscribe packet");

        let client_id = self.get_client_id_from_handler_id(c_h_id);
        let session;
        if let Some(client_id) = client_id {
            session = self.sessions.get_mut(&client_id).unwrap();
        } else {
            return Err("Client not found".into());
        }

        let mut suback_packet = Suback::new(subscribe_packet.packet_id);
        for subscription in subscribe_packet.subscriptions {
            if topic_filters::topic_filter_is_valid(&subscription.topic_filter) == false {
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

            //Retain Logic Subscribe
            if let Some(topic) = self.retained_messages.keys().find(|topic| {
                topic_filters::filter_matches_topic(&subscription.topic_filter, topic)
            }) {
                let mut flags = 0b0011_0001;
                let mut id = None;
                if subscription.max_qos == Qos::AtLeastOnce
                    && self.retained_messages.get(topic).unwrap().qos == Qos::AtLeastOnce
                {
                    if let Some(packet_id) =
                        PacketProcessor::find_key_for_value(self.packets_id.clone(), false)
                    {
                        self.packets_id.insert(packet_id, true);
                        id = Some(packet_id);
                    }

                    flags = 0b0011_0011;
                }

                //Send publish al cliente con el mensaje en el retained_messages
                let publish_packet = Publish::new(
                    PublishFlags::new(flags),
                    subscription.topic_filter,
                    id,
                    self.retained_messages
                        .get(topic)
                        .unwrap()
                        .message
                        .to_string(),
                );
                println!("Publish a mandar: {:?}", &publish_packet);

                let senders_hash = self.senders_to_c_h_writers.read().unwrap();
                let sender = senders_hash.get(&c_h_id).unwrap();
                let sender_mutex_guard = sender.lock().unwrap();
                sender_mutex_guard
                    .send(Ok(Packet::Publish(publish_packet)))
                    .unwrap();
            }
        }
        //suback_packet.add_return_code(SuccessAtMostOnce);///HARDCODED
        Ok(suback_packet)
    }

    pub fn handle_publish_packet(
        &mut self,
        publish_packet: Publish,
    ) -> Result<Option<Puback>, Box<dyn std::error::Error>> {
        println!("Se recibió el publish packet");
        //Sacamos el packet_id del pubblish
        //Sacar info del publish
        //Mandamos el puback al client.

        //let current_session = self.sessions.get_mut("a").unwrap(); //TODO: sacar unwrap
        let topic_name = &publish_packet.topic_name;

        //Retain Logic Publish
        //A PUBLISH Packet with a RETAIN flag set to 1 and a payload containing zero bytes will be processed as normal by the Server
        //and sent to Clients with a subscription matching the topic name.
        //Additionally any existing retained message with the same topic name MUST be removed and any future subscribers
        //for the topic will not receive a retained message
        if publish_packet.flags.retain {
            self.retained_messages.insert(
                topic_name.clone(),
                Message {
                    message: publish_packet.application_message.clone(),
                    qos: publish_packet.flags.qos_level,
                },
            );
        }

        match publish_packet.flags.qos_level {
            Qos::AtMostOnce => self.handle_publish_packet_qos0(publish_packet),
            _ => self.handle_publish_packet_qos1(publish_packet),
        }
    }

    fn handle_publish_packet_qos0(
        &mut self,
        publish_packet: Publish,
    ) -> Result<Option<Puback>, Box<dyn std::error::Error>> {
        let mut publish_send = publish_packet.clone();
        publish_send.flags.duplicate = false;
        publish_send.flags.retain = false;

        for (_, session) in &self.sessions {
            if let Some(_) = session.is_subscribed_to(&publish_packet.topic_name) {
                if let Some(client_handler_id) = session.get_client_handler_id() {
                    self.send_packet_to_client_handler(
                        client_handler_id,
                        Ok(Packet::Publish(publish_send.clone())),
                    )?;
                }
            }
        }
        return Ok(None);
    }

    fn handle_publish_packet_qos1(
        &mut self,
        publish_packet: Publish,
    ) -> Result<Option<Puback>, Box<dyn std::error::Error>> {
        let packet_id = publish_packet.packet_id;

        let mut publish_send = publish_packet.clone();
        publish_send.flags.duplicate = false;
        publish_send.flags.retain = false;

        for (_, session) in &mut self.sessions {
            if session.is_subscribed_to(&publish_packet.topic_name) == Some(Qos::AtLeastOnce) {
                session.store_publish_packet(publish_send.clone());
            }
        }

        for (_, session) in &self.sessions {
            match session.is_subscribed_to(&publish_packet.topic_name) {
                Some(Qos::AtLeastOnce) => {
                    if let Some(client_handler_id) = session.get_client_handler_id() {
                        let mut publish_send_2 = publish_send.clone();
                        publish_send_2.packet_id = packet_id;
                        self.packets_id.insert(packet_id.unwrap(), true);

                        self.send_packet_to_client_handler(
                            client_handler_id,
                            Ok(Packet::Publish(publish_send_2.clone())),
                        )?;
                        self.tx_to_puback_processor
                            .send((
                                client_handler_id,
                                Ok(Packet::Publish(publish_send_2.clone())),
                            ))
                            .unwrap();
                    }
                }

                Some(Qos::AtMostOnce) => {
                    if let Some(client_handler_id) = session.get_client_handler_id() {
                        let mut publish_send_2 = publish_send.clone();
                        publish_send_2.packet_id = None;
                        publish_send_2.flags.qos_level = Qos::AtMostOnce;
                        self.send_packet_to_client_handler(
                            client_handler_id,
                            Ok(Packet::Publish(publish_send_2.clone())),
                        )?;
                    }
                }

                _ => (),
            }
        }

        println!("Se envio correctamente el PUBACK");
        return Ok(Some(Puback::new(packet_id.unwrap())));
    }

    pub fn handle_puback_packet(
        &mut self,
        puback_packet: Puback,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let puback_packet_id = puback_packet.packet_id;

        self.tx_to_puback_processor
            .send((0, Ok(Packet::Puback(puback_packet))))?;

        for (_, session) in &mut self.sessions {
            if session.is_active() {
                session.unacknowledged_messages.retain(|publish_packet| {
                    println!(
                        "packetid_publish: {:?} || packetid_puback: {:?}",
                        publish_packet.packet_id.unwrap(),
                        puback_packet_id
                    );
                    publish_packet.packet_id.unwrap() != puback_packet_id
                })
            }
        }

        Ok(())
    }

    fn get_client_id_from_handler_id(&self, c_h_id: u32) -> Option<String> {
        for (client_id, session) in &self.sessions {
            if session.is_active() && session.get_client_handler_id().unwrap() == c_h_id {
                return Some(client_id.to_string());
            }
        }
        None
    }

    fn send_packet_to_client_handler(
        &self,
        c_h_id: u32,
        packet: Result<Packet, Box<dyn std::error::Error + Send>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let senders_hash = self.senders_to_c_h_writers.read().unwrap();
        let sender = senders_hash.get(&c_h_id).unwrap();
        let sender_mutex_guard = sender.lock().unwrap();
        sender_mutex_guard.send(packet)?;
        Ok(())
    }

    fn find_key_for_value(map: HashMap<u16, bool>, _value: bool) -> Option<u16> {
        for (key, value) in map {
            if value == false {
                return Some(key);
            }
        }
        None
    }

    fn send_unacknowledged_messages(&mut self, c_h_id: u32) {
        if let Some(client_id) = self.get_client_id_from_handler_id(c_h_id) {
            //thread::sleep(Duration::from_millis(100));

            let current_session = self.sessions.get_mut(&client_id).unwrap();
            let mut unacknowledged_messages_copy = current_session.unacknowledged_messages.clone();
            unacknowledged_messages_copy.retain(|publish| {
                println!("Envio el publish: {:?}", publish.application_message);
                self.send_packet_to_client_handler(c_h_id, Ok(Packet::Publish(publish.clone())))
                    .is_err()
            });
            self.sessions
                .get_mut(&client_id)
                .unwrap()
                .unacknowledged_messages = unacknowledged_messages_copy;
        }
    }
}
