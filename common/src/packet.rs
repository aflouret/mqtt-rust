use crate::all_packets::connack::Connack;
use crate::all_packets::connect::Connect;
use std::io::{Read, Write};

pub enum Packet {
    Connect(Connect),
    Connack(Connack),
    Publish,
    Puback,
    Subscribe,
    Unsubscribe,
    Suback,
    Unsuback,
    Disconnect,
}

pub trait ReadPacket {
    fn read_from(stream: &mut dyn Read) -> Result<Packet, Box<dyn std::error::Error>>;
}

pub trait WritePacket {
    fn write_to(&self, stream: &mut dyn Write) -> Result<(), Box<dyn std::error::Error>>;
}