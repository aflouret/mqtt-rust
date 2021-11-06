use crate::packet::{Packet, ReadPacket, WritePacket};
use std::io::{Read, Write};

pub const CONNACK_PACKET_TYPE: u8 = 0x20;
const CONNACK_REMAINING_LENGTH: u8 = 2;

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
        stream.write(&[CONNACK_PACKET_TYPE])?;

        // Escribimos el remaining length
        stream.write(&[CONNACK_REMAINING_LENGTH])?;

        // VARIABLE HEADER
        // Escribimos el session present flag
        let session_present_flag = self.session_present as u8;
        stream.write(&[session_present_flag])?;

        // Escribimos el connect return code
        stream.write(&[self.connect_return_code])?;

        Ok(())
    }
}

impl ReadPacket for Connack {
    fn read_from(stream: &mut dyn Read, initial_byte: u8) -> Result<Packet, Box<dyn std::error::Error>> {
        println!("Entro a connack");

        let mut remaining_length_byte = [0u8; 1];
        stream.read_exact(&mut remaining_length_byte)?;
        verify_remaining_length_byte(&remaining_length_byte)?;
        println!("Remaining length leido: 2");

        let mut flags_byte = [0u8; 1];
        stream.read_exact(&mut flags_byte)?;
        verify_flags_byte(&flags_byte)?;

        let session_present = flags_byte[0] == 0x1;
        println!("Session present leido: {}", session_present);

        let mut connect_return_byte = [0u8; 1];
        stream.read_exact(&mut connect_return_byte)?;
        let connect_return_code = connect_return_byte[0];
        println!("Connect return code leido: {}", connect_return_code);

        verify_packet(session_present, connect_return_code)?;
        Ok(Packet::Connack(Connack {
            session_present,
            connect_return_code,
        }))
    }
}

fn verify_flags_byte(byte: &[u8; 1]) -> Result<(), String> {
    if byte[0] & !0x1 != 0x0 {
        return Err("Flags invalidos".into());
    }
    Ok(())
}

fn verify_remaining_length_byte(byte: &[u8; 1]) -> Result<(), String> {
    if byte[0] != CONNACK_REMAINING_LENGTH {
        return Err("Remaining length byte inválido".into());
    }
    Ok(())
}

fn verify_packet(session_present_flag: bool, connect_return_code: u8) -> Result<(), String> {
    if connect_return_code != 0 && session_present_flag == true {
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
        let byte: [u8; 1] = [0x2];
        let to_test = verify_remaining_length_byte(&byte);
        assert_eq!(to_test, Ok(()));
    }

    #[test]
    fn error_remaining_length_byte() {
        let byte: [u8; 1] = [0x5];
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
