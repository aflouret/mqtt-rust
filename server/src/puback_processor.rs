use common::all_packets::puback::Puback;
use common::all_packets::publish::Publish;
use common::packet::Packet;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use crate::server::{PacketResult, ArcSenderPacket};

pub struct PubackProcessor {
    publish_packets: Vec<(SystemTime, u32, Publish)>,
    senders_to_c_h_writers: Arc<
        RwLock<HashMap<u32, ArcSenderPacket>>,
    >,
    rx_from_packet_processor: Receiver<(u32, PacketResult)>,
}

impl PubackProcessor {
    pub fn new(
        senders_to_c_h_writers: Arc<
            RwLock<
                HashMap<u32, ArcSenderPacket>,
            >,
        >,
        rx_from_packet_processor: Receiver<(
            u32,
            PacketResult,
        )>,
    ) -> PubackProcessor {
        PubackProcessor {
            senders_to_c_h_writers,
            rx_from_packet_processor,
            publish_packets: vec![],
        }
    }

    pub fn run(mut self) {
        loop {
            if let Ok(received) = self
                .rx_from_packet_processor
                .recv_timeout(Duration::from_millis(1000))
            {
                if let (id, Ok(packet)) = received {
                    match packet {
                        Packet::Publish(publish_packet) => {
                            let time = SystemTime::now();
                            self.publish_packets.push((time, id, publish_packet));
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
        self.publish_packets.retain(|(_, _, publish_packet)| {
            publish_packet.packet_id.unwrap() != puback_packet.packet_id
        });
    }

    fn resend_packets(&mut self) {
        let current_time = SystemTime::now();
        let to_send: Vec<(u32, Publish)> = self
            .publish_packets
            .clone()
            .into_iter()
            .filter(|(time, _, _)| {
                current_time.duration_since(*time).unwrap() >= Duration::from_millis(1000)
            })
            .map(|(_, id, publish_packet)| (id, publish_packet))
            .collect();

        for (id, mut packet) in to_send {
            packet.flags.duplicate = true;
            if self.send_packet(id, packet).is_err() {
                self.publish_packets.retain(|(_, c_h_id, _)| *c_h_id != id);
                break;
            };
        }
    }

    fn send_packet(
        &mut self,
        id: u32,
        publish_packet: Publish,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let senders_hash = self.senders_to_c_h_writers.read().unwrap();
        let sender = senders_hash.get(&id).ok_or("Sender not found")?;
        let sender_mutex_guard = sender.lock().unwrap();
        sender_mutex_guard
            .send(Ok(Packet::Publish(publish_packet)))
            .unwrap();
        Ok(())
    }
}
