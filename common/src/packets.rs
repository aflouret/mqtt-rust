use crate::all_packets::connect::Connect;
use crate::parser::{identify_package};
use std::io::Read;


struct Connack {
}

struct Publish {
}

struct Puback {
}

struct Subscribe {
}

struct Unsubscribe {
}

struct Suback {
}

struct Unsuback {
}

struct Disconnect {
}

pub enum Packet {
    Connect(Connect),
    Connack(Connack),
    Publish(Publish),
    Puback(Puback),
    Subscribe(Subscribe),
    Unsubscribe(Unsubscribe),
    Suback(Suback),
    Unsuback(Unsuback),
    Disconnect(Disconnect),
}

impl Packet {
    pub fn read_from(stream: &mut dyn Read) -> Result<Packet,String> {
        let mut indetifier_byte = [0u8; 1];
        stream.read_exact(&mut indetifier_byte)?;
        let builder = identify_package(indetifier_byte.0)?;
        let packet = builder.build_packet(stream)?;
        Ok(packet)
    }
}