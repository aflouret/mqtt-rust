use crate::packet::{Packet, ReadPacket, WritePacket};
use std::io::{Cursor, Read, Write};
use crate::parser::{decode_remaining_length, encode_remaining_length};

pub const UNSUBACK_PACKET_TYPE: u8 = 0xb0;
const UNSUBACK_REMAINING_LENGTH: u32 = 2;

#[derive(Debug)]
pub struct Unsuback {
    packet_id: u16,
}

impl Unsuback {
    pub fn new(packet_id: u16) -> Unsuback {
        Unsuback {
            packet_id,
        }
    }
}

impl WritePacket for Unsuback {
    fn write_to(&self, stream: &mut dyn Write) -> Result<(), Box<dyn std::error::Error>> {
        // FIXED HEADER
        // Escribimos el Packet Type
        stream.write_all(&[UNSUBACK_PACKET_TYPE])?;

        //Escribimos el remaining length 
        let remaining_length_encoded = encode_remaining_length(UNSUBACK_REMAINING_LENGTH);
        for byte in remaining_length_encoded {
            stream.write_all(&[byte])?;
        }

        //VARIABLE HEADER
        let packet_id_bytes = self.packet_id.to_be_bytes();
        stream.write_all(&packet_id_bytes)?;

        Ok(())
    }
}

impl ReadPacket for Unsuback {
    fn read_from(stream: &mut dyn Read, initial_byte: u8) -> Result<Packet, Box<dyn std::error::Error>> {
        verify_unsuback_byte(&initial_byte)?;
        let remaining_length = decode_remaining_length(stream)?;
        verify_remaining_length_byte(&remaining_length)?;

        let mut remaining = vec![0u8; remaining_length as usize];
        stream.read_exact(&mut remaining)?;
        let mut remaining_bytes = Cursor::new(remaining);

        let mut packet_identifier_bytes = [0u8; 2];
        remaining_bytes.read_exact(&mut packet_identifier_bytes)?;
        let packet_identifier = u16::from_be_bytes(packet_identifier_bytes);

        Ok(Packet::Unsuback(Unsuback::new(packet_identifier)))
    }
}

fn verify_remaining_length_byte(byte: &u32) -> Result<(), String> {
    if *byte != UNSUBACK_REMAINING_LENGTH {
        return Err("Incorrect Remaining Length".to_string());
    }
    Ok(())
}

fn verify_unsuback_byte(byte: &u8) -> Result<(), String>{
    match *byte {
        UNSUBACK_PACKET_TYPE => Ok(()),
        _ => Err("Wrong First Byte".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_first_byte() {
        let to_test = verify_unsuback_byte(&UNSUBACK_PACKET_TYPE);
        assert_eq!(to_test, Ok(()));
    }

    #[test]
    fn error_wrong_first_byte(){
        let byte: u8 = 0xa0;
        let to_test = verify_unsuback_byte(&byte);
        assert_eq!(to_test.unwrap_err().to_string(), "Wrong First Byte");
    }

    #[test]
    fn correct_remaining_length_byte() {
        let byte: u32 = UNSUBACK_REMAINING_LENGTH;
        let to_test = verify_remaining_length_byte(&byte);
        assert_eq!(to_test, Ok(()));
    }

    #[test]
    fn error_remaining_length_byte() {
        let byte: u32 = 0x5;
        let to_test = verify_remaining_length_byte(&byte);
        assert_eq!(to_test, Err("Incorrect Remaining Length".to_string()));
    }
}