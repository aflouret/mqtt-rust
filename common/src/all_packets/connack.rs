use crate::packet::{Packet, ReadPacket, WritePacket};
use std::io::{Read, Write, Cursor};
use crate::parser::{decode_remaining_length, encode_remaining_length};

pub const CONNACK_PACKET_TYPE: u8 = 0x20;
const CONNACK_REMAINING_LENGTH: u32 = 2;

pub struct Connack {
    pub session_present: bool,
    pub connect_return_code: u8,
}

impl Connack {
    pub fn new(session_present: bool, connect_return_code: u8) -> Connack {
        Connack {
            session_present,
            connect_return_code,
        }
    }
}

impl WritePacket for Connack {
    fn write_to(&self, stream: &mut dyn Write) -> Result<(), Box<dyn std::error::Error>> {
        // FIXED HEADER
        // Escribimos el packet type + los flags del packet type
        stream.write_all(&[CONNACK_PACKET_TYPE])?;

        //Escribimos el remaining length
        let remaining_length_encoded = encode_remaining_length(CONNACK_REMAINING_LENGTH);
        for byte in remaining_length_encoded {
            stream.write_all(&[byte])?;
        }

        // VARIABLE HEADER
        // Escribimos el session present flag
        let session_present_flag = self.session_present as u8;
        stream.write_all(&[session_present_flag])?;

        // Escribimos el connect return code
        stream.write_all(&[self.connect_return_code])?;

        println!("Connack packet escrito correctamente");

        Ok(())
    }
}

impl ReadPacket for Connack {
    fn read_from(stream: &mut dyn Read, initial_byte: u8) -> Result<Packet, Box<dyn std::error::Error>> {
        verify_connack_byte(&initial_byte)?;
        let remaining_length = decode_remaining_length(stream)?;
        verify_remaining_length_byte(&remaining_length)?;

        let mut remaining = vec![0u8; remaining_length as usize];
        stream.read_exact(&mut remaining)?;
        let mut remaining_bytes = Cursor::new(remaining);

        let mut flags_byte = [0u8; 1];
        remaining_bytes.read_exact(&mut flags_byte)?;
        verify_flags_byte(&flags_byte)?;

        let session_present = flags_byte[0] == 0x1;

        let mut connect_return_byte = [0u8; 1];
        remaining_bytes.read_exact(&mut connect_return_byte)?;
        let connect_return_code = connect_return_byte[0];

        verify_packet(session_present, connect_return_code)?;

        println!("Connack packet leido correctamente");

        Ok(Packet::Connack(Connack {
            session_present,
            connect_return_code,
        }))
    }
}

fn verify_connack_byte(byte: &u8) -> Result<(), String>{
    match *byte {
        CONNACK_PACKET_TYPE => return Ok(()),
        _ => return Err("Wrong First Byte".to_string()),
    }
}

fn verify_flags_byte(byte: &[u8; 1]) -> Result<(), String> {
    //Byte 1 is the "Connect Acknowledge Flags". Bits 7-1 are reserved and MUST be set to 0. 
    if byte[0] & !0x1 != 0x0 {
        return Err("Flags invalidos".into());
    }
    Ok(())
}

fn verify_remaining_length_byte(byte: &u32) -> Result<(), String> {
    if *byte != CONNACK_REMAINING_LENGTH {
        return Err("Remaining length byte inválido".into());
    }
    Ok(())
}

fn verify_packet(session_present_flag: bool, connect_return_code: u8) -> Result<(), String> {
    //If a server sends a CONNACK packet containing a non-zero return code it MUST set Session Present to 0
    if connect_return_code != 0 && session_present_flag {
        return Err("Session present debe valer 0".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use super::*;

    #[test]
    fn correct_flag_byte_0() {
        let byte: [u8; 1] = [0x0];
        let to_test = verify_flags_byte(&byte);
        assert_eq!(to_test, Ok(()));
    }

    #[test]
    fn correct_flag_byte_1() {
        let byte: [u8; 1] = [0x1];
        let to_test = verify_flags_byte(&byte);
        assert_eq!(to_test, Ok(()));
    }

    #[test]
    fn error_flag_byte() {
        let byte: [u8; 1] = [0x2];
        let to_test = verify_flags_byte(&byte);
        assert_eq!(to_test, Err("Flags invalidos".to_owned()));
    }

    #[test]
    fn correct_remaining_length_byte() {
        let byte: u32 = 0x2;
        let to_test = verify_remaining_length_byte(&byte);
        assert_eq!(to_test, Ok(()));
    }

    #[test]
    fn error_remaining_length_byte() {
        let byte: u32 = 0x5;
        let to_test = verify_remaining_length_byte(&byte);
        assert_eq!(to_test, Err("Remaining length byte inválido".to_owned()));
    }

    #[test]
    fn correct_packet() {
        let connack_packet = Connack::new(true, 0);

        let mut buff = Cursor::new(Vec::new());
        connack_packet.write_to(&mut buff).unwrap();
        buff.set_position(1);
        let to_test = Connack::read_from(&mut buff, 0x20).unwrap();
        if let Packet::Connack(to_test) = to_test {
            assert_eq!(to_test.session_present, connack_packet.session_present);
            assert_eq!(
                to_test.connect_return_code,
                connack_packet.connect_return_code
            )
        }
    }

    #[test]
    fn error_packet() {
        let connack_packet = Connack::new(true, 1);

        let mut buff = Cursor::new(Vec::new());
        connack_packet.write_to(&mut buff).unwrap();
        buff.set_position(1);
        let to_test = Connack::read_from(&mut buff, 0x20);
        assert!(to_test.is_err());
    }
}
