use crate::all_packets::connack::{Connack, CONNACK_PACKET_TYPE};
use crate::all_packets::connect::{Connect, CONNECT_PACKET_TYPE};
use crate::all_packets::publish::{Publish, PUBLISH_PACKET_TYPE};
use crate::all_packets::puback::{Puback, PUBACK_PACKET_TYPE};
use crate::all_packets::subscribe::{Subscribe, SUBSCRIBE_PACKET_TYPE};
use crate::all_packets::suback::{Suback, SUBACK_PACKET_TYPE};
use crate::all_packets::disconnect::{Disconnect, DISCONNECT_PACKET_TYPE};
use crate::all_packets::unsubscribe::{Unsubscribe, UNSUBSCRIBE_PACKET_TYPE};
use crate::all_packets::unsuback::{Unsuback, UNSUBACK_PACKET_TYPE};
use crate::all_packets::pingreq::{Pingreq, PINGREQ_PACKET_TYPE};
use crate::all_packets::pingresp::{Pingresp, PINGRESP_PACKET_TYPE};
use crate::parser::decode_mqtt_string;
use std::io::{Read, Write};

const PACKET_TYPE_BYTE: u8 = 0xF0;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Qos {
    AtMostOnce = 0,
    AtLeastOnce = 1,
}

#[derive(Debug)]
pub struct Topic {
    pub name: String,
    pub qos: Qos,
}

impl Topic {
    pub fn read_from(stream: &mut dyn Read) -> Result<Topic, std::io::Error> {
        let topic_name = decode_mqtt_string(stream)?;
        
        let mut qos_level_bytes = [0u8; 1];
        stream.read_exact(&mut qos_level_bytes)?;
        let qos_level = u8::from_be_bytes(qos_level_bytes);

        let qos = match qos_level {
            0 => Qos::AtMostOnce,
            _ => Qos::AtLeastOnce,
        };

        Ok(Topic{name: topic_name, qos})
    }
}

pub trait ReadPacket {
    fn read_from(stream: &mut dyn Read, initial_byte: u8) -> Result<Packet, Box<dyn std::error::Error>>;
}

pub trait WritePacket {
    fn write_to(&self, stream: &mut dyn Write) -> Result<(), Box<dyn std::error::Error>>;
}

pub enum Packet {
    Connect(Connect),
    Connack(Connack),
    Publish(Publish),
    Puback(Puback),
    Subscribe(Subscribe),
    Suback(Suback),
    Unsubscribe(Unsubscribe),
    Unsuback(Unsuback),
    Pingreq(Pingreq),
    Pingresp(Pingresp),
    Disconnect(Disconnect),
}

impl Packet {
    pub fn read_from(stream: &mut dyn Read) -> Result<Packet, Box<dyn std::error::Error>>{
        let mut indetifier_byte = [0u8; 1];
        let read_bytes = stream.read(&mut indetifier_byte)?;
        if read_bytes == 0 {
            println!("ENTRO a socket disconeect");
            return Err("Socket desconectado".into());
        }
        match indetifier_byte[0] & PACKET_TYPE_BYTE {
            CONNECT_PACKET_TYPE => Ok(Connect::read_from(stream, indetifier_byte[0])?),
            CONNACK_PACKET_TYPE => Ok(Connack::read_from(stream, indetifier_byte[0])?),
            PUBLISH_PACKET_TYPE => Ok(Publish::read_from(stream, indetifier_byte[0])?),
            PUBACK_PACKET_TYPE =>  Ok(Puback::read_from(stream, indetifier_byte[0])?),
            SUBSCRIBE_PACKET_TYPE => Ok(Subscribe::read_from(stream, indetifier_byte[0])?),
            SUBACK_PACKET_TYPE => Ok(Suback::read_from(stream, indetifier_byte[0])?),
            UNSUBSCRIBE_PACKET_TYPE => Ok(Unsubscribe::read_from(stream, indetifier_byte[0])?),
            UNSUBACK_PACKET_TYPE => Ok(Unsuback::read_from(stream, indetifier_byte[0])?),
            PINGREQ_PACKET_TYPE => Ok(Pingreq::read_from(stream, indetifier_byte[0])?),
            PINGRESP_PACKET_TYPE => Ok(Pingresp::read_from(stream, indetifier_byte[0])?),
            DISCONNECT_PACKET_TYPE => Ok(Disconnect::read_from(stream, indetifier_byte[0])?),
            _ => Err("Ningún packet tiene ese código".into()),
        }
    }

    pub fn write_to(&self, stream: &mut dyn Write) -> Result<(), Box<dyn std::error::Error>>{
        match self {
            Packet::Connect(connect) => {
                println!("Se manda el connect...");
                connect.write_to(stream)
            }

            Packet::Connack(connack) => {
                println!("Se manda el connack...");
                connack.write_to(stream)
            }

            Packet::Publish(publish) => {
                println!("Se manda el publish...");
                publish.write_to(stream)
            }

            Packet::Puback(puback) => {
                println!("Se manda el puback...");
                puback.write_to(stream)
            }

            Packet::Subscribe(subscribe) => {
                println!("Se manda el subscribe...");
                subscribe.write_to(stream)
            }

            Packet::Suback(suback) => {
                println!("Se manda el suback...");
                suback.write_to(stream)
            }

            Packet::Unsubscribe(unsubscribe) => {
                println!("Se manda el unsubscribe...");
                unsubscribe.write_to(stream)
            }

            Packet::Unsuback(unsuback) => {
                println!("Se manda el unsuback...");
                unsuback.write_to(stream)
            }
            
            Packet::Pingreq(pingreq) => {
                println!("Se manda el pingreq...");
                pingreq.write_to(stream)
            }

            Packet::Pingresp(pingresp) => {
                println!("Se manda el pingresp...");
                pingresp.write_to(stream)
            }

            Packet::Disconnect(disconnect) => {
                println!("Se manda el disconnect...");
                disconnect.write_to(stream)
            }
        }
    }
}
