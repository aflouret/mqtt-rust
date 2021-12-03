use std::error::Error;
use std::io::{Read, Write, Cursor};
use crate::packet::{Packet, ReadPacket, WritePacket};
use crate::parser::{decode_remaining_length, encode_remaining_length};

pub const PUBACK_PACKET_TYPE : u8 = 0x40;
const PUBACK_REMAINING_LENGTH: u32 = 2;

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
        let remaining_length_encoded = encode_remaining_length(PUBACK_REMAINING_LENGTH);
        for byte in remaining_length_encoded {
            stream.write_all(&[byte])?;
        }

        //VARIABLE HEADER
        let packet_id_from_publish = self.packet_id.to_be_bytes();
        stream.write_all(&packet_id_from_publish)?;

        println!("Puback packet escrito correctamente");

        Ok(())
    }
}

impl ReadPacket for Puback {
    fn read_from(stream: &mut dyn Read, initial_byte: u8) -> Result<Packet, Box<dyn Error>> {
        verify_puback_byte(&initial_byte)?;
        let remaining_length = decode_remaining_length(stream)?;
        verify_remaining_length_byte(&remaining_length)?;

        let mut remaining = vec![0u8; remaining_length as usize];
        stream.read_exact(&mut remaining)?;
        let mut remaining_bytes = Cursor::new(remaining);

        let mut packet_id = [0u8; 2];
        remaining_bytes.read_exact(&mut packet_id)?;
        let packet_id = u16::from_be_bytes(packet_id);

        println!("Puback packet leido correctamente");

        Ok(Packet::Puback(Puback {
            packet_id,
        }))
    }
}

fn verify_puback_byte(byte: &u8) -> Result<(), String>{
    match *byte {
        PUBACK_PACKET_TYPE => return Ok(()),
        _ => return Err("Wrong First Byte".to_string()),
    }
}

fn verify_remaining_length_byte(byte: &u32) -> Result<(), String> {
    if *byte != PUBACK_REMAINING_LENGTH {
        return Err("Incorrect Remaining Length".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn correct_remaining_length_byte() {
        let byte: u32 = 0x2;
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

    #[test]
    fn error_remaining_length_byte() {
        let byte: u32 = 0xF;
        let to_test = verify_remaining_length_byte(&byte);

        assert_eq!(to_test, Err("Incorrect Remaining Length".to_owned()));
    }

    #[test]
    fn error_first_byte() {
        let byte: u8 = 0xaa;
        let to_test = verify_puback_byte(&byte);
        assert_eq!(to_test.unwrap_err().to_string(), "Wrong First Byte");
    }
}