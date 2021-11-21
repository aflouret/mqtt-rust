use crate::all_packets::connack::Connack;
use crate::all_packets::connect::Connect;
use crate::all_packets::publish::Publish;
use crate::all_packets::subscribe::Subscribe;
use crate::all_packets::suback::Suback;
use std::io::{Read, Write};
use crate::all_packets::puback::Puback;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Qos {
    AtMostOnce = 0,
    AtLeastOnce = 1,
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
    Disconnect,
}


pub trait ReadPacket {
    fn read_from(stream: &mut dyn Read, initial_byte: u8) -> Result<Packet, Box<dyn std::error::Error>>;
}

pub trait WritePacket {
    fn write_to(&self, stream: &mut dyn Write) -> Result<(), Box<dyn std::error::Error>>;
}
