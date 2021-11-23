use crate::packet::{Packet, ReadPacket, WritePacket};
use std::io::{Read, Write};
use crate::parser::{decode_remaining_length, encode_remaining_length};
use std::io::{Error, ErrorKind::Other};

const PINGRESP_REMAINING_LENGTH: u32 = 0;
pub const PINGRESP_PACKET_TYPE: u8 = 0xd0;

pub struct Pingresp;

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
        if remaining_length != 0 {
            return Err(Box::new(Error::new(Other, "Incorrect Remaining Length")));
        }

        Ok(Packet::Pingresp(Pingresp{}))
    }
}

fn verify_disconnect_byte(byte: &u8) -> Result<(), String>{
    match *byte {
        PINGRESP_PACKET_TYPE => return Ok(()),
        _ => return Err("Wrong First Byte".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_first_byte() {
        let to_test = verify_disconnect_byte(&PINGRESP_PACKET_TYPE);
        assert_eq!(to_test, Ok(()));
    }
}