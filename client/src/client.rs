use common::all_packets::connack::{CONNACK_CONNECTION_ACCEPTED};
use common::all_packets::connect::{Connect, ConnectPayload};
use common::all_packets::pingreq::Pingreq;
use common::all_packets::publish::{Publish, PublishFlags};
use std::time::Duration;
use crate::handlers::{EventHandlers, HandleDisconnect, HandleInternPacketId, HandleInternPuback, HandlePublish, HandleSubscribe, HandleUnsubscribe};
use crate::response::{PubackResponse, PublishResponse, ResponseHandlers};
use crate::HandleConection;
use common::all_packets::puback::Puback;
use common::all_packets::subscribe::Subscribe;
use common::packet::WritePacket;
use common::packet::SOCKET_CLOSED_ERROR_MSG;
use common::packet::{Packet, Qos, Subscription};
use std::collections::HashMap;
use std::net::{Shutdown, TcpStream};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, mpsc, Mutex};
use std::{io, thread};
use crate::client_puback_processor::PubackProcessor;

const MAX_KEEP_ALIVE: u16 = 65000;
const MAX_WAIT_TIME_FOR_CONNACK_IF_NO_KEEP_ALIVE: u64 = 10;
// KeepAlive muy grande para el caso que keep_alive es 0 => en este caso el server no espera ningun tiempo para que el client envíe paquetes.
const PACKETS_ID: u16 = 100;

pub type PacketResult = Result<Packet, Box<dyn std::error::Error + Send>>;

#[derive(Debug)]
pub struct Client {
    server_stream: Option<TcpStream>,
    packets_id: HashMap<u16, bool>,
    tx_to_puback_processor: Sender<PacketResult>,
    rx_from_packet_processor:
    Option<Receiver<PacketResult>>,
}

impl Client {
    /// Devuelve un client con un socket no conectado.
    pub fn new() -> Client {
        let mut packets: HashMap<u16, bool> = HashMap::new();
        for i in 0..PACKETS_ID {
            packets.insert(i, false);
        }
        let (tx_to_puback_processor, rx_from_puback_processor) = mpsc::channel::<PacketResult>();
        Client {
            server_stream: None,
            packets_id: packets,
            tx_to_puback_processor,
            rx_from_packet_processor: Some(rx_from_puback_processor),
        }
    }

    pub fn set_server_stream(&mut self, stream: TcpStream) {
        self.server_stream = Some(stream);
    }

    pub fn start_client(
        mut self,
        recv_connection: Receiver<EventHandlers>,
        sender_to_window: Sender<ResponseHandlers>,
        sender_intern: Sender<EventHandlers>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let recv_connection = Arc::new(Mutex::new(recv_connection));
        let rx_from_client = self.rx_from_packet_processor.take().unwrap();
        let sender_intern_puback = sender_intern.clone();
        thread::spawn(move || {
            let puback_processor =
                PubackProcessor::new(rx_from_client, sender_intern_puback);
            puback_processor.run();
        });
        loop {
            self.run_gui_processor(
                recv_connection.clone(),
                sender_to_window.clone(),
                sender_intern.clone(),
            )?;
        }
    }

    pub fn run_gui_processor(
        &mut self,
        recv_connection: Arc<Mutex<std::sync::mpsc::Receiver<EventHandlers>>>,
        sender_to_window: Sender<ResponseHandlers>,
        sender_intern: Sender<EventHandlers>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let recv_connection = recv_connection.lock().unwrap();
        let mut keep_alive_sec: u16 = 0;

        loop {
            if let Ok(conection) = recv_connection.recv() {
                match conection {
                    EventHandlers::Conection(conec) => {
                        self.handle_conection(
                            conec,
                            sender_to_window.clone(),
                            &mut keep_alive_sec,
                            sender_intern.clone(),
                        ).unwrap();
                        println!("CLIENT: CONNECTED");
                        break;
                    }
                    _ => println!("Must first connect"),
                };
            }
        }

        loop {
            if keep_alive_sec == 0 {
                keep_alive_sec = MAX_KEEP_ALIVE;
            }

            match recv_connection.recv_timeout(Duration::new(keep_alive_sec as u64, 0)) {
                Ok(EventHandlers::Conection(conec)) => {
                    self.handle_conection(
                        conec,
                        sender_to_window.clone(),
                        &mut keep_alive_sec,
                        sender_intern.clone(),
                    )?;
                }

                Ok(EventHandlers::Publish(publish)) => {
                    self.handle_publish(publish)?;
                }
                Ok(EventHandlers::Subscribe(subscribe)) => {
                    self.handle_subscribe(subscribe)?;
                }
                Ok(EventHandlers::Unsubscribe(unsubs)) => {
                    self.handle_unsubscribe(unsubs)?;
                }
                Ok(EventHandlers::Disconnect(disconnect)) => {
                    self.handle_disconnect(disconnect)?;
                    return Ok(());
                },
                Ok(EventHandlers::InternPuback(intern)) => {
                    let packet_id = intern.puback_packet.packet_id;
                    self.packets_id.insert(packet_id, false); //packet id puesto en false, que no se está usando
                    self.tx_to_puback_processor.send(Ok(Packet::Puback(Puback::new(packet_id)))).unwrap();
                },
                Ok(EventHandlers::InternPublish(publish)) => {
                    self.handle_publish_from_puback_processor(publish.publish_packet).unwrap();
                },
                Ok(EventHandlers::InternPacketId(intern)) => {
                    let packet_id = intern.packet_id;
                    self.packets_id.insert(packet_id, false); //packet id puesto en false, que no se está usando
                },


                Err(mpsc::RecvTimeoutError::Timeout) => {
                    self.handle_pingreq()?;
                }

                _ => (),
            };
        }
    }

    pub fn handle_response(
        mut s: TcpStream,
        sender: Sender<ResponseHandlers>,
        sender_intern: Sender<EventHandlers>,
    ) {
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
                        s.set_read_timeout(Some(Duration::new(u64::from(MAX_KEEP_ALIVE) * 2, 0)))
                            .unwrap();
                    }
                    Ok(Packet::Puback(puback)) => {
                        let intern = EventHandlers::InternPuback(HandleInternPuback::new(
                            puback
                        ));
                        sender_intern.send(intern).unwrap();
                        println!("CLIENT: PUBACK packet successful received");
                        let puback_response = ResponseHandlers::PubackResponse(
                            PubackResponse::new("Published correctly".to_string())
                        );
                        sender.send(puback_response).unwrap();
                       // thread::sleep(Duration::new(2, 0));
/*                        sender.
                            send(ResponseHandlers::PubackResponse(
                                PubackResponse::new("".to_string())))
                            .unwrap();*/
                        println!("CLIENT: Packet id enviado internamente para liberar");

                        //mandar via channel el puback al puback processor,
                    }
                    Ok(Packet::Suback(suback)) => {
                        println!("CLIENT: SUBACK packet successful received");
                        let intern = EventHandlers::InternPacketId(HandleInternPacketId::new(suback.packet_id));
                        sender_intern.send(intern).unwrap();
                        //liberar el packet id que nos mandan
                    }
                    Ok(Packet::Unsuback(_unsuback)) => {
                        println!("CLIENT: UNSUBACK packet successful received");
                    }
                    Ok(Packet::Publish(publish)) => {
                        println!("CLIENT: Recibi publish: msg: {:?}, qos: {}", &publish.application_message, publish.flags.qos_level as u8);
                        if let Some(id) = publish.packet_id {
                            let puback = Puback::new(id);
                            puback.write_to(&mut s).unwrap();
                        }
                        subscriptions_msg.push(
                            "Topic: ".to_string()
                                + &publish.topic_name.to_string()
                                + &" - ".to_string()
                                + &publish.application_message.to_string()
                                + &" - Qos: ".to_string()
                                + &(publish.flags.qos_level as u8).to_string()
                                + " \n",
                        );
                        let response = ResponseHandlers::PublishResponse(PublishResponse::new(
                            publish,
                            subscriptions_msg.clone(),
                            "Published Succesfull".to_string(),
                        ));
                        sender.send(response).unwrap();
                    }
                    Ok(Packet::Pingresp(_pingresp)) => {
                        println!("CLIENT: Pingresp successful received");
                    }
                    Err(e) => {
                        match e.to_string().as_str() {
                            SOCKET_CLOSED_ERROR_MSG => {
                                // Causado por el Disconnect
                                println!("Se desconecta por socket cerrado");
                            }
                            _ => {
                                // Causado por el read_timeout
                                println!("Se cierra el cliente por no recibir el Connack a tiempo");
                                s.shutdown(Shutdown::Both).unwrap();
                                std::process::exit(1);
                            }
                        }
                        break;
                    }
                    _ => (),
                };
            }
        });
    }


    pub fn handle_pingreq(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(socket) = &mut self.server_stream {
            let mut s = socket.try_clone()?;
            let pingreq_packet = Pingreq::new();
            println!("CLIENT: Send pinreq packet");
            pingreq_packet.write_to(&mut s)?;
        }

        Ok(())
    }

    pub fn handle_publish_from_puback_processor(&mut self, publish: Publish) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(socket) = &mut self.server_stream {
            let mut s = socket.try_clone()?;
            publish.write_to(&mut s)?;
        }

        Ok(())
    }

    pub fn handle_disconnect(
        &mut self,
        disconnect: HandleDisconnect)
        -> Result<(), Box<dyn std::error::Error>> {
        if let Some(socket) = &mut self.server_stream {
            let mut s = socket.try_clone()?;
            let disconnect_packet = disconnect.disconnect_packet;
            println!("CLIENT: Send disconnect packet: {:?}", &disconnect_packet);
            disconnect_packet.write_to(&mut s)?;
            //TODO: cerrar la conexion
            socket.shutdown(Shutdown::Both)?;
        }

        Ok(())
    }

    pub fn handle_unsubscribe(
        &mut self,
        unsubs: HandleUnsubscribe)
        -> Result<(), Box<dyn std::error::Error>> {
        if let Some(socket) = &mut self.server_stream {
            let mut s = socket.try_clone()?;
            let unsubscribe_packet = unsubs.unsubscribe_packet;
            println!("CLIENT: Send unsubscribe packet: {:?}", &unsubscribe_packet);
            unsubscribe_packet.write_to(&mut s)?;
        }

        Ok(())
    }

    pub fn handle_subscribe(
        &mut self,
        subscribe: HandleSubscribe)
        -> Result<(), Box<dyn std::error::Error>> {
        if let Some(socket) = &mut self.server_stream {
            let mut s = socket.try_clone()?;
            let subscribe_packet = self.create_subscribe_packet(subscribe)?;
            println!("CLIENT: Send subscribe packet: {:?}", &subscribe_packet);
            subscribe_packet.write_to(&mut s)?;
        }

        Ok(())
    }

    pub fn create_subscribe_packet(
        &mut self,
        subscribe: HandleSubscribe)
        -> io::Result<Subscribe> {
        let packet_id = self.get_packet_id();
        let mut subscribe_packet = Subscribe::new(packet_id);
        subscribe_packet.add_subscription(Subscription { topic_filter: subscribe.topic, max_qos: subscribe.qos });

        Ok(subscribe_packet)
    }

    pub fn handle_publish(&mut self, publish: HandlePublish) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(socket) = &mut self.server_stream {
            let mut s = socket.try_clone()?;
            let publish_packet = self.create_publish_packet(publish)?;
            if publish_packet.flags.qos_level == Qos::AtLeastOnce && !publish_packet.flags.duplicate {
                self.tx_to_puback_processor.send(Ok(Packet::Publish(publish_packet.clone()))).unwrap();
            }
            println!("CLIENT: Send publish packet: {:?}", &publish_packet);
            publish_packet.write_to(&mut s)?;
        }
        Ok(())
    }

    pub fn get_packet_id(&mut self) -> u16 {
        let packet_id: u16;
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
        packet_id
    }

    pub fn create_publish_packet(
        &mut self,
        publish: HandlePublish,
    ) -> io::Result<Publish> {
        let mut packet_id: u16 = 0;
        if publish.qos1_level {
            packet_id = self.get_packet_id();
        }

        let qos_lvl: Qos;
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
            PublishFlags {
                duplicate: false,
                qos_level: qos_lvl,
                retain: publish.retain,
            },
            publish.topic,
            packet_id_send,
            publish.app_msg,
        );

        Ok(publish_packet)
    }

    pub fn handle_conection(
        &mut self,
        mut conec: HandleConection,
        sender_to_window: Sender<ResponseHandlers>,
        keep_alive_sec: &mut u16,
        sender_intern: Sender<EventHandlers>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let address = conec.get_address();
        let mut socket = TcpStream::connect(address.clone())?;
        let keep_alive_time = conec.keep_alive_second.parse()?;
        println!("Connecting to: {:?}", address);
        println!("Client id: {:?}", conec.client_id);

        let connect_packet = Connect::new(
            ConnectPayload::new(
                conec.client_id,
                conec.last_will.last_will_topic,
                conec.last_will.last_will_msg,
                conec.username, conec.password,
            ),
            keep_alive_time,
            conec.clean_session,
            conec.last_will.last_will_retain,
            conec.last_will.last_will_qos,
        );

        let mut max_wait_time_for_connack = MAX_WAIT_TIME_FOR_CONNACK_IF_NO_KEEP_ALIVE;
        if keep_alive_time != 0 {
            max_wait_time_for_connack = u64::from(keep_alive_time) * 2;
        }

        socket
            .set_read_timeout(Some(Duration::new(max_wait_time_for_connack, 0)))
            .unwrap();

        Client::handle_response(socket.try_clone().unwrap(), sender_to_window, sender_intern);

        *keep_alive_sec = keep_alive_time;
        println!("CLIENT: Send connect packet: {:?}", &connect_packet);
        connect_packet.write_to(&mut socket)?;
        self.set_server_stream(socket);
        Ok(())
    }

    fn find_key_for_value(map: HashMap<u16, bool>, value_to_look: bool) -> Option<u16> {
        for (key, value) in map {
            if value == value_to_look {
                return Some(key);
            }
        }
        None
    }
}
