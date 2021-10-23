use std::io::Read;
use crate::all_packets::connect::Connect;

pub enum Packet {
    Connect(Connect),
    Connack,
    Publish,
    Puback,
    Subscribe,
    Unsubscribe,
    Suback,
    Unsuback,
    Disconnect,
}

pub trait ReadPacket {
    fn read_from(stream: &mut dyn Read) -> Result<Packet, String>;
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