use crate::all_packets::connect::Connect;
use crate::all_packets::connack::Connack;
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

/*impl Packet {
    pub fn read_from(stream: &mut dyn Read) -> Result<Packet,String> {
        let mut indetifier_byte = [0u8; 1];
        stream.read_exact(&mut indetifier_byte)?;
        //que esta identificar devuelva el paquete que sea, y sino que devuelva error
        let packet = build_packet(indetifier_byte.0,stream)?;
        //connect
        //t p = packet.build_packet(packet);
        Ok(p)
    }


}*/
