use crate::packet::{Packet, ReadPacket, WritePacket};
use std::io::{Cursor, Read, Write};
use crate::parser::{decode_remaining_length, encode_remaining_length};
use std::io::{Error, ErrorKind::UnexpectedEof, ErrorKind::Other};

const DISCONNECT_REMAINING_LENGTH: u32 = 0;
pub const DISCONNECT_PACKET_TYPE: u8 = 0xe0;

pub struct Disconnect;

impl WritePacket for Disconnect {
    fn write_to(&self, stream: &mut dyn Write) -> Result<(), Box<dyn std::error::Error>> {
        // FIXED HEADER
        // Escribimos el Packet Type
        stream.write_all(&[DISCONNECT_PACKET_TYPE])?;

        //Escribimos el remaining length 
        let remaining_length_encoded = encode_remaining_length(DISCONNECT_REMAINING_LENGTH);
        for byte in remaining_length_encoded {
            stream.write_all(&[byte])?;
        }

        Ok(())
    }
}

impl ReadPacket for Disconnect {
    fn read_from(stream: &mut dyn Read, initial_byte: u8) -> Result<Packet, Box<dyn std::error::Error>> {
        verify_disconnect_byte(&initial_byte)?;
        let remaining_length = decode_remaining_length(stream)?;
        if remaining_length != 0 {
            return Err(Box::new(Error::new(Other, "Incorrect Remaining Length")));
        }

        Ok(Packet::Disconnect(Disconnect{}))
    }
}

fn verify_disconnect_byte(byte: &u8) -> Result<(), String>{
    match *byte {
        DISCONNECT_PACKET_TYPE => return Ok(()),
        _ => return Err("Wrong Packet Type".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_first_byte() {
        let to_test = verify_disconnect_byte(&DISCONNECT_PACKET_TYPE);
        assert_eq!(to_test, Ok(()));
    }
}
