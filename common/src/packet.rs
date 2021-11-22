use crate::all_packets::connack::{Connack, CONNACK_PACKET_TYPE};
use crate::all_packets::connect::{Connect, CONNECT_PACKET_TYPE};
use crate::all_packets::publish::{Publish, PUBLISH_PACKET_TYPE};
use crate::all_packets::puback::{Puback, PUBACK_PACKET_TYPE};
use crate::all_packets::subscribe::{Subscribe, SUBSCRIBE_PACKET_TYPE};
use crate::all_packets::suback::{Suback, SUBACK_PACKET_TYPE};
use crate::all_packets::disconnect::{Disconnect, DISCONNECT_PACKET_TYPE};
use std::io::{Read, Write};

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Qos {
    AtMostOnce = 0,
    AtLeastOnce = 1,
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
    Unsubscribe,
    Suback(Suback),
    Unsuback,
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
        match indetifier_byte[0] & 0xF0 {
            CONNECT_PACKET_TYPE => Ok(Connect::read_from(stream, indetifier_byte[0])?),
            CONNACK_PACKET_TYPE => Ok(Connack::read_from(stream, indetifier_byte[0])?),
            PUBLISH_PACKET_TYPE => Ok(Publish::read_from(stream, indetifier_byte[0])?),
            PUBACK_PACKET_TYPE =>  Ok(Puback::read_from(stream, indetifier_byte[0])?),
            SUBSCRIBE_PACKET_TYPE => Ok(Subscribe::read_from(stream, indetifier_byte[0])?),
            SUBACK_PACKET_TYPE => Ok(Suback::read_from(stream, indetifier_byte[0])?),
            // 0xa _ => { Ok(Unsuscribe::read_from(stream, indetifier_byte[0])?) }
            // 0xb_ => { Ok(Unsuback::read_from(stream, indetifier_byte[0])?) }
            // 0xc_ => { Ok(Pingreq::read_from(stream, indetifier_byte[0])?) }
            // 0xd _ => { Ok(Pingresp::read_from(stream, indetifier_byte[0])?) }
            DISCONNECT_PACKET_TYPE => Ok(Disconnect::read_from(stream, indetifier_byte[0])?),
            _ => Err("Ningún packet tiene ese código".into()),
        }
    }
}

impl WritePacket for Packet {
    fn write_to(&self, stream: &mut dyn Write) -> Result<(), Box<dyn std::error::Error>>{
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

            //...

            _ => Err("Packet desconocido".into())
        }
    }
}
