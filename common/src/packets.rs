mod packet_flags;
mod parser;
use connect::{indentify_package};

struct Connack {
    flags: Flags,
}

struct Publish {
    flags: Flags,
}

struct Puback {
    flags: Flags,
}

struct Subscribe {
    flags: Flags,
}

struct Unsubscribe {
    flags: Flags,
}

struct Suback {
    flags: Flags,
}

struct Unsuback {
    flags: Flags,
}

struct Disconnect {
    flags: Flags,
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
    pub fn read_from(stream: &mut dyn Read) -> std::io::Result<Packet> {
        let mut indetifier_byte = [0u8; 1];
        stream.read_exact(&mut indetifier_byte)?;
        let builder = indentify_package(indetifier_byte)?;
        let packet = builder.build_packet(stream)?;
        packet
    }
}