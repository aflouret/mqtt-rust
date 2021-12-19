use crate::packet::{Packet, ReadPacket, WritePacket};
use std::io::{Read, Write};
use crate::parser::{decode_remaining_length, encode_remaining_length};

const PINGRESP_REMAINING_LENGTH: u32 = 0;
pub const PINGRESP_PACKET_TYPE: u8 = 0xd0;

#[derive(Debug)]
pub struct Pingresp;

impl Pingresp {
    pub fn new() -> Pingresp {
        Pingresp {}
    }
}

impl WritePacket for Pingresp {
    fn write_to(&self, stream: &mut dyn Write) -> Result<(), Box<dyn std::error::Error>> {
        // FIXED HEADER
        // Escribimos el Packet Type
        stream.write_all(&[PINGRESP_PACKET_TYPE])?;

        //Escribimos el remaining length 
        let remaining_length_encoded = encode_remaining_length(PINGRESP_REMAINING_LENGTH);
        for byte in remaining_length_encoded {
            stream.write_all(&[byte])?;
        }

        Ok(())
    }
}

impl ReadPacket for Pingresp {
    fn read_from(stream: &mut dyn Read, initial_byte: u8) -> Result<Packet, Box<dyn std::error::Error>> {
        verify_disconnect_byte(&initial_byte)?;
        let remaining_length = decode_remaining_length(stream)?;
        verify_remaining_length_byte(&remaining_length)?;

        Ok(Packet::Pingresp(Pingresp::new()))
    }
}

impl Default for Pingresp {
    fn default() -> Self {
        Self::new()
    }
}

fn verify_disconnect_byte(byte: &u8) -> Result<(), String>{
    match *byte {
        PINGRESP_PACKET_TYPE => Ok(()),
        _ => Err("Wrong First Byte".to_string()),
    }
}

fn verify_remaining_length_byte(byte: &u32) -> Result<(), String> {
    if *byte != PINGRESP_REMAINING_LENGTH {
        return Err("Incorrect Remaining Length".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_first_byte() {
        let to_test = verify_disconnect_byte(&PINGRESP_PACKET_TYPE);
        assert_eq!(to_test, Ok(()));
    }

    #[test]
    fn correct_remaining_length_byte() {
        let byte: u32 = PINGRESP_REMAINING_LENGTH;
        let to_test = verify_remaining_length_byte(&byte);
        assert_eq!(to_test, Ok(()));
    }

    #[test]
    fn error_wrong_first_byte(){
        let byte: u8 = 0xc0;
        let to_test = verify_disconnect_byte(&byte);
        assert_eq!(to_test.unwrap_err().to_string(), "Wrong First Byte");
    }

    #[test]
    fn error_remaining_length_byte() {
        let byte: u32 = 0x5;
        let to_test = verify_remaining_length_byte(&byte);
        assert_eq!(to_test, Err("Incorrect Remaining Length".to_string()));
    }
}