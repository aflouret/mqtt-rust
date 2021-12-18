use common::all_packets::connect::{Connect, ConnectPayload};
use common::all_packets::connack::{CONNACK_CONNECTION_ACCEPTED};
use common::all_packets::publish::{Publish, PublishFlags};
use common::all_packets::pingreq::Pingreq;
use std::time::Duration;

use common::packet::{Packet, Qos, Subscription};
use common::packet::WritePacket;
use std::net::{TcpStream, Shutdown};
use std::sync::{Arc, mpsc, Mutex};
use std::sync::mpsc::{Receiver, Sender};
use std::{io, thread};
use std::collections::HashMap;
use common::all_packets::puback::Puback;
use common::all_packets::subscribe::Subscribe;
use common::packet::{SOCKET_CLOSED_ERROR_MSG};
use crate::handlers::{EventHandlers, HandleDisconnect,HandleInternPacketId, HandlePublish, HandleSubscribe, HandleUnsubscribe};
use crate::HandleConection;
use crate::response::{PubackResponse, PublishResponse, ResponseHandlers};

const MAX_KEEP_ALIVE: u16 = 65000;
const MAX_WAIT_TIME_FOR_CONNACK_IF_NO_KEEP_ALIVE: u64 = 10;
// KeepAlive muy grande para el caso que keep_alive es 0 => en este caso el server no espera ningun tiempo para que el client envíe paquetes.
const PACKETS_ID: u16 = 100;


#[derive(Debug)]
pub struct Client {
    server_stream: Option<TcpStream>,
    packets_id: HashMap<u16, bool>,
}

impl Client {
    /// Devuelve un client con un socket no conectado.
    pub fn new() -> Client {
        let mut packets: HashMap<u16, bool> = HashMap::new();
        for i in 0..PACKETS_ID {
            packets.insert(i, false);
        }
        Client {
            server_stream: None,
            packets_id: packets,
        }
    }

    pub fn set_server_stream(&mut self, stream: TcpStream) {
        self.server_stream = Some(stream);
    }

    pub fn start_client(mut self, recv_connection: Receiver<EventHandlers>, sender_to_window: Sender<ResponseHandlers>, sender_intern: Sender<EventHandlers>) -> Result<(), Box<dyn std::error::Error>> {
        let recv_connection = Arc::new(Mutex::new(recv_connection));
        loop{
            //probando
            self.run_gui_processor(recv_connection.clone(), sender_to_window.clone(), sender_intern.clone())?;
        }
    }

    pub fn run_gui_processor(&mut self, recv_connection: Arc<Mutex<std::sync::mpsc::Receiver<EventHandlers>>>, sender_to_window: Sender<ResponseHandlers>, sender_intern: Sender<EventHandlers>) -> Result<(), Box<dyn std::error::Error>> {
        let recv_connection = recv_connection.lock().unwrap();
        let mut keep_alive_sec: u16 = 0;

        loop {
            if let Ok(conection) = recv_connection.recv() {
                match conection {
                    EventHandlers::HandleConection(conec) => {
                        self.handle_conection(conec, sender_to_window.clone(), &mut keep_alive_sec, sender_intern.clone()).unwrap();
                        println!("Connected Client");
                        break;
                    }
                    _ => println!("Primero se debe conectar"),
                };
            }
        }

        loop {
            if keep_alive_sec == 0 {
                keep_alive_sec = MAX_KEEP_ALIVE;
            }
            
            match recv_connection.recv_timeout(Duration::new(keep_alive_sec as u64, 0)) {
                Ok(EventHandlers::HandleConection(conec)) => {
                    self.handle_conection(conec, sender_to_window.clone(), &mut keep_alive_sec, sender_intern.clone() ).unwrap();
                }

                Ok(EventHandlers::HandlePublish(publish)) => {
                    //escuchar el pbuack processor para reenviar publish
                    self.handle_publish(publish).unwrap();
                }
                Ok(EventHandlers::HandleSubscribe(subscribe)) => {
                    self.handle_subscribe(subscribe).unwrap();
                }
                Ok(EventHandlers::HandleUnsubscribe(unsubs)) => {
                    self.handle_unsubscribe(unsubs).unwrap();
                }
                Ok(EventHandlers::HandleDisconnect(disconnect)) => {
                    self.handle_disconnect(disconnect).unwrap();
                    return Ok(())
                }

                Err(mpsc::RecvTimeoutError::Timeout) => {
                    self.handle_pingreq().unwrap();
                }

                _ => ()
            };
        }

        Ok(())
    }

    pub fn handle_response(mut s: TcpStream, sender: Sender<ResponseHandlers>, sender_ev_handlers: Sender<EventHandlers>) {
        let mut subscriptions_msg: Vec<String> = Vec::new();
        thread::spawn(move || {
            loop {
                let receiver_packet = Packet::read_from(&mut s);
                
                match receiver_packet {
                    Ok(Packet::Connack(connack)) => {
                        println!("CLIENT: CONNACK packet successful received");
                        if connack.connect_return_code != CONNACK_CONNECTION_ACCEPTED {
                            s.shutdown(Shutdown::Both).unwrap();
                        }
                        // Como llegó el Connack, listo, "deshacemos" el read_timeout del socket
                        s.set_read_timeout(Some(Duration::new(u64::from(MAX_KEEP_ALIVE) * 2, 0))).unwrap();
                    }
                    Ok(Packet::Puback(puback)) => {
                        println!("CLIENT: PUBACK packet successful received");
                        let puback_response = ResponseHandlers::PubackResponse(PubackResponse::new("PubackResponse".to_string()));
                        sender.send(puback_response);
                        thread::sleep(Duration::new(2,0));
                        sender.send(ResponseHandlers::PubackResponse(PubackResponse::new("".to_string())));
                        let intern = EventHandlers::HandleInternPacketId(HandleInternPacketId::new(puback.packet_id));
                        sender_ev_handlers.send(intern);
                        println!("CLIENT: Packet id enviado internamente para liberar");
                        //sender.send("Topic Successfully published".to_string());
                        //mandar via channel el puback al puback processor,
                    }
                    Ok(Packet::Suback(suback)) => {
                        println!("CLIENT: SUBACK packet successful received");
                        let intern = EventHandlers::HandleInternPacketId(HandleInternPacketId::new(suback.packet_id));
                        sender_ev_handlers.send(intern);
                        //liberar el packet id que nos mandan
                    }
                    Ok(Packet::Unsuback(_unsuback)) => {
                        println!("CLIENT: UNSUBACK packet successful received");
                    }
                    Ok(Packet::Publish(publish)) => {
                        println!("CLIENT: Recibi publish: msg: {:?}, qos: {}", &publish.application_message, publish.flags.qos_level as u8);
                        if let Some(id) = publish.packet_id {
                            let puback = Puback::new(id);
                            puback.write_to(&mut s);
                        }
                        //let packet_id_pub = publish.packet_id.unwrap();
                        //subscriptions_msg.push(publish.application_message.to_string() + " - topic:" + &*publish.topic_name.to_string() + " \n");
                        subscriptions_msg.push(
                            //publish.application_message.to_string() + " - topic:" + &*publish.topic_name.to_string() + " \n"
                            "Topic: ".to_string() + &publish.topic_name.to_string() + &" - ".to_string() + &publish.application_message.to_string() +
                            &" - Qos: ".to_string() + &(publish.flags.qos_level as u8).to_string() + " \n"
                        );
                        let response = ResponseHandlers::PublishResponse(PublishResponse::new(publish, subscriptions_msg.clone(), "Published Succesfull".to_string()));
                        sender.send(response);
                        /*
                        let puback = Puback::new(packet_id_pub);
                        puback.write_to(&mut s);
                        */
                    }
                    Ok(Packet::Pingresp(_pingresp)) => {
                        println!("CLIENT: Pingresp successful received");
                    }
                    Err(e) => { 
                        match e.to_string().as_str() {
                            SOCKET_CLOSED_ERROR_MSG => { // Causado por el Disconnect
                                println!("Se desconecta por socket cerrado");
                            },
                            _ => { // Causado por el read_timeout
                                println!("Se cierra el cliente por no recibir el Connack a tiempo");
                                s.shutdown(Shutdown::Both).unwrap();
                                std::process::exit(1);
                            }
                        }  
                        break;
                    }        
                    _ => ()
                };
            }
        });
    }


    pub fn handle_pingreq(&mut self) -> io::Result<()> {
        if let Some(socket) = &mut self.server_stream {
            let mut s = socket.try_clone()?;
            let pingreq_packet = Pingreq::new(); //Usar new una vez mergeado
            println!("CLIENT: Send pinreq packet");
            pingreq_packet.write_to(&mut s);
        }

        Ok(())
    }

    pub fn handle_disconnect(&mut self, disconnect: HandleDisconnect) -> io::Result<()> {
        if let Some(socket) = &mut self.server_stream {
            let mut s = socket.try_clone()?;
            let disconnect_packet = disconnect.disconnect_packet;
            println!("CLIENT: Send disconnect packet: {:?}", &disconnect_packet);
            disconnect_packet.write_to(&mut s).unwrap();
            //TODO: cerrar la conexion
            socket.shutdown(Shutdown::Both).unwrap();
        }

        Ok(())
    }

    pub fn handle_unsubscribe(&mut self, unsubs: HandleUnsubscribe) -> io::Result<()> {
        if let Some(socket) = &mut self.server_stream {
            let mut s = socket.try_clone()?;
            let unsubscribe_packet = unsubs.unsubscribe_packet;
            println!("CLIENT: Send unsubscribe packet: {:?}", &unsubscribe_packet);
            unsubscribe_packet.write_to(&mut s);
        }

        Ok(())
    }

    pub fn handle_subscribe(&mut self, subscribe: HandleSubscribe) -> io::Result<()> {
        if let Some(socket) = &mut self.server_stream {
            let mut s = socket.try_clone()?;
            let subscribe_packet = self.create_subscribe_packet(subscribe).unwrap();
            println!("CLIENT: Send subscribe packet: {:?}", &subscribe_packet);
            subscribe_packet.write_to(&mut s);
        }

        Ok(())
    }

    pub fn create_subscribe_packet(&mut self, subscribe: HandleSubscribe) -> io::Result<Subscribe> {
        let mut packet_id: u16 = 0;
        if let Some(id) = Client::find_key_for_value(self.packets_id.clone(), false) {
            packet_id = id;
            self.packets_id.insert(id, true);
        } else {
            let length = self.packets_id.len();
            for i in length..length * 2 {
                self.packets_id.insert(i as u16, false);
            }
            packet_id = length as u16;
            self.packets_id.insert(packet_id, true);
        }
        let mut subscribe_packet = Subscribe::new(packet_id);
        subscribe_packet.add_subscription(Subscription { topic_filter: subscribe.topic, max_qos: subscribe.qos });

        Ok(subscribe_packet)
    }

    pub fn handle_publish(&mut self, publish: HandlePublish) -> io::Result<()> {
        if let Some(socket) = &mut self.server_stream {
            let mut s = socket.try_clone()?;
            let publish_packet = self.create_publish_packet(publish).unwrap();
            println!("CLIENT: Send publish packet: {:?}", &publish_packet);
            publish_packet.write_to(&mut s);
        }
        Ok(())
    }

    pub fn create_publish_packet(&mut self, publish: HandlePublish) -> io::Result<Publish> {
        let mut packet_id: u16 = 0;
        if publish.qos1_level {
            if let Some(id) = Client::find_key_for_value(self.packets_id.clone(), false) {
                packet_id = id;
                self.packets_id.insert(id, true);
            } else {
                let length = self.packets_id.len();
                for i in length..length * 2 {
                    self.packets_id.insert(i as u16, false);
                }
                packet_id = length as u16;
                self.packets_id.insert(packet_id, true);
            }
        }

        let mut qos_lvl : Qos = Qos::AtMostOnce;
        if publish.qos1_level {
            qos_lvl = Qos::AtLeastOnce;
        } else {
            qos_lvl = Qos::AtMostOnce;
        }

        let packet_id_send = match packet_id {
            0 => None,
            _ => Some(packet_id),
        };

        let publish_packet = Publish::new(
            PublishFlags {duplicate: false, qos_level: qos_lvl, retain: publish.retain },
            publish.topic, packet_id_send, publish.app_msg,
        );

        Ok(publish_packet)
    }

    pub fn handle_conection(&mut self, mut conec: HandleConection, sender_to_window: Sender<ResponseHandlers>, keep_alive_sec: &mut u16, sender_intern: Sender<EventHandlers>) -> io::Result<()> {
        let address = conec.get_address();
        let mut socket = TcpStream::connect(address.clone()).unwrap();
        let keep_alive_time = conec.keep_alive_second.parse().unwrap();
        println!("Connecting to: {:?}", address);
        
        let connect_packet = Connect::new(
            ConnectPayload::new(conec.client_id, 
                conec.last_will_topic,
                conec.last_will_msg, 
                conec.username, conec.password),
                keep_alive_time,
            conec.clean_session, 
            conec.last_will_retain, 
            conec.last_will_qos);
        

        let mut max_wait_time_for_connack = MAX_WAIT_TIME_FOR_CONNACK_IF_NO_KEEP_ALIVE; 
        if keep_alive_time != 0 {
            max_wait_time_for_connack = u64::from(keep_alive_time) * 2;
        } 
        socket.set_read_timeout(Some(Duration::new(max_wait_time_for_connack, 0))).unwrap();
        
        Client::handle_response(socket.try_clone().unwrap(), sender_to_window, sender_intern.clone());

        *keep_alive_sec = keep_alive_time.clone();
        println!("CLIENT: Send connect packet: {:?}", &connect_packet);
        connect_packet.write_to(&mut socket);
        self.set_server_stream(socket);
        Ok(())
    }

    fn find_key_for_value(map: HashMap<u16, bool>, value_to_look: bool) -> Option<u16> {
        for (key, value) in map {
            if value == value_to_look {
                return  Some(key);
            }
        }
        None
    }
}
