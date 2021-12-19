use common::all_packets::puback::Puback;
use common::all_packets::publish::Publish;
use common::packet::Packet;

use std::sync::mpsc::{Receiver, Sender};

use std::time::{Duration, SystemTime};
use crate::client::PacketResult;
use crate::handlers::{EventHandlers, HandleInternPublish};

pub struct PubackProcessor {
    publish_packets: Vec<(SystemTime, Publish)>,
    rx_from_client: Receiver<PacketResult>,
    sender_to_event_handler_client: Sender<EventHandlers>,
}

impl PubackProcessor {
    pub fn new(
        rx_from_client: Receiver<PacketResult>,
        sender_to_event_handler_client: Sender<EventHandlers>,
    ) -> PubackProcessor {
        PubackProcessor {
            rx_from_client,
            publish_packets: vec![],
            sender_to_event_handler_client: sender_to_event_handler_client,
        }
    }

    pub fn run(mut self) {
        loop {
            if let Ok(received) = self
                .rx_from_client
                .recv_timeout(Duration::from_millis(1000))
            {
                if let Ok(packet) = received {
                    match packet {
                        Packet::Publish(publish_packet) => {
                            let time = SystemTime::now();
                            self.publish_packets.push((time, publish_packet));
                        }
                        Packet::Puback(puback_packet) => {
                            self.process_puback(puback_packet);
                            self.resend_packets();
                        }
                        _ => {
                            self.resend_packets();
                        }
                    }
                }
            } else {
                self.resend_packets();
            }
        }
    }

    fn process_puback(&mut self, puback_packet: Puback) {
        let mut index_to_delete= 0;
        for (index, packet) in self.publish_packets.iter_mut().enumerate() {
            if packet.1.packet_id.unwrap() == puback_packet.packet_id {
                index_to_delete = index;
            }
        }
        self.publish_packets.remove(index_to_delete);
    }

    fn resend_packets(&mut self) {
        let current_time = SystemTime::now();
        let to_send: Vec<Publish> = self
            .publish_packets
            .clone()
            .into_iter()
            .filter(|(time, _)| {
                current_time.duration_since(*time).unwrap() >= Duration::from_millis(1000)
            })
            .map(|(_, publish_packet)| publish_packet)
            .collect();

        for mut packet in to_send {
            packet.flags.duplicate = true;
            self.send_packet(packet).unwrap();
        }
    }

    fn send_packet(
        &mut self,
        publish_packet: Publish,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.sender_to_event_handler_client.send(EventHandlers::InternPublish(HandleInternPublish::new(publish_packet))).unwrap();
        Ok(())
    }
}
