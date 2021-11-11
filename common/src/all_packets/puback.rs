use std::error::Error;
use std::io::{Read, Write};
use crate::packet::{Packet, ReadPacket, WritePacket};

pub const PUBACK_PACKET_TYPE : u8 = 0x40;
const PUBACK_REMAINING_LENGTH: u8 = 2;

#[derive(Debug)]
pub struct Puback {
    packet_id: u16,

}

impl Puback {
    pub fn new(packet_id: u16) -> Puback {
        Puback {
            packet_id,
        }
    }

}

impl WritePacket for Puback{
    fn write_to(&self, stream: &mut dyn Write) -> Result<(), Box<dyn Error>> {
        //FIXED HEADER
        //Escribimos el packet type
        stream.write_all(&[PUBACK_PACKET_TYPE])?;

        //Escribimos el remaining length
        stream.write_all(&[PUBACK_REMAINING_LENGTH])?;

        //VARIABLE HEADER
        let packet_id_from_publish = self.packet_id.to_be_bytes();
        stream.write_all(&packet_id_from_publish)?;

        Ok(())
    }
}

impl ReadPacket for Puback {
    fn read_from(stream: &mut dyn Read, _initial_byte: u8) -> Result<Packet, Box<dyn Error>> {
        let mut remaining_length_byte = [0u8; 1];
        stream.read_exact(&mut remaining_length_byte)?;
        verify_remaining_length_byte(&remaining_length_byte)?;

        let mut packet_id = [0u8; 2];
        stream.read_exact(&mut packet_id)?;
        let packet_id = u16::from_be_bytes(packet_id);
        println!("Packet_id que devuelve puback es: {}", packet_id);

        Ok(Packet::Puback(Puback {
            packet_id,
        }))

    }
}

//Consultar de meterla en un utils.rs porque tmb se usa en connack.rs
fn verify_remaining_length_byte(byte: &[u8; 1]) -> Result<(), String> {
    if byte[0] != PUBACK_REMAINING_LENGTH {
        return Err("Remaining length byte inválido".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn correct_remaining_length_byte() {
        let byte: [u8; 1] = [0x2];
        let to_test = verify_remaining_length_byte(&byte);
        assert_eq!(to_test, Ok(()));
    }

    #[test]
    fn correct_packet_id() {
        let puback_packet = Puback::new(10);
        let mut buff = Cursor::new(Vec::new());
        puback_packet.write_to(&mut buff).unwrap();
        buff.set_position(1);
        let to_test = Puback::read_from(&mut buff, PUBACK_PACKET_TYPE).unwrap();
        if let Packet::Puback(to_test) = to_test {
            assert_eq!(to_test.packet_id, 10);
        }
    }
}