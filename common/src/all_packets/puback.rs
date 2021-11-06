use std::error::Error;
use std::io::{Read, Write};
use crate::packet::{Packet, ReadPacket, WritePacket};
use crate::parser::encode_remaining_length;

pub const PUBACK_PACKET_TYPE : u8 = 0x40;
const PUBACK_VARIABLE_HEADER_BYTES: u32 = 2;
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
        stream.write(&[PUBACK_PACKET_TYPE])?;

        //Escribimos el remaining length
        stream.write(&[PUBACK_REMAINING_LENGTH]);

        //VARIABLE HEADER
        let packet_id_from_publish = self.packet_id as u8;
        stream.write(&[packet_id_from_publish])?;

        Ok(())
    }
}

impl ReadPacket for Puback {
    fn read_from(stream: &mut dyn Read, initial_byte: u8) -> Result<Packet, Box<dyn Error>> {

        let mut remaining_length_byte = [0u8; 1];
        stream.read_exact(&mut remaining_length_byte)?;
        verify_remaining_length_byte(&remaining_length_byte)?;

        let mut packet_id = [0u8; 2];
        stream.read_exact(&mut packet_id)?;
        let packet_id = packet_id[0] as u16;
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

    #[test]
    fn correct_remaining_length_byte() {
        let byte: [u8; 1] = [0x2];
        let to_test = verify_remaining_length_byte(&byte);
        assert_eq!(to_test, Ok(()));
    }
}