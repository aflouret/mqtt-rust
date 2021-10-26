use crate::packet::{Packet, ReadPacket, WritePacket};
use std::io::{Read, Write};

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
        let packet_type_and_flags = 0x20_u8;
        stream.write(&[packet_type_and_flags])?;

        // Escribimos el remaining length
        let remaining_length = 0x02_u8;
        stream.write(&[remaining_length])?;

        // VARIABLE HEADER
        // Escribimos el session present flag
        let session_present_flag = if self.session_present == true { 0x1_u8 } else { 0x0_u8 };
        stream.write(&[session_present_flag])?;

        // Escribimos el connect return code
        stream.write(&[self.connect_return_code])?;

        Ok(())
    }
}

impl ReadPacket for Connack {
    fn read_from(stream: &mut dyn Read) -> Result<Packet, Box<dyn std::error::Error>> {
        println!("Entro a connack");

        let mut remaining_length_byte = [0u8; 1];
        stream.read_exact(&mut remaining_length_byte)?;
        verify_remaining_length_byte(&remaining_length_byte)?;
        println!("Remaining length leido: 2");

        let mut flags_byte = [0u8; 1];
        stream.read_exact(&mut flags_byte)?;
        verify_flags_byte(&flags_byte)?;
        let session_present = if flags_byte[0] & 0x1 == 1 { true } else { false };
        println!("Session present leido: {}", session_present);

        let mut connect_return_byte = [0u8; 1];
        stream.read_exact(&mut connect_return_byte)?;
        let connect_return_code = connect_return_byte[0];
        println!("Connect return code leido: {}", connect_return_code);

        Ok(Packet::Connack(Connack{ session_present, connect_return_code }))
    }
}

fn verify_flags_byte(byte: &[u8; 1]) -> Result<(), String> {
    if byte[0] & !0x1 != 0x0 {
        return Err("Flags invalidos".into());
    }
    Ok(())
}

fn verify_remaining_length_byte(byte: &[u8; 1]) -> Result<(), String> {
    if byte[0] != 0x2 {
        return Err("Remaining length byte inválido".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_flag_byte_0(){
        let byte: [u8; 1] = [0x0];
        let to_test = verify_flags_byte(&byte);
        assert_eq!(to_test, Ok(()));
    }

    #[test]
    fn correct_flag_byte_1(){
        let byte: [u8; 1] = [0x1];
        let to_test = verify_flags_byte(&byte);
        assert_eq!(to_test, Ok(()));
    }

    #[test]
    fn error_flag_byte(){
        let byte: [u8; 1] = [0x2];
        let to_test = verify_flags_byte(&byte);
        assert_eq!(to_test, Err("Flags invalidos".to_owned()));
    }

    #[test]
    fn correct_remaining_length_byte(){
        let byte: [u8; 1] = [0x2];
        let to_test = verify_remaining_length_byte(&byte);
        assert_eq!(to_test, Ok(()));
    }

    #[test]
    fn error_remaining_length_byte(){
        let byte: [u8; 1] = [0x5];
        let to_test = verify_remaining_length_byte(&byte);
        assert_eq!(to_test, Err("Remaining length byte inválido".to_owned()));
    }
}
