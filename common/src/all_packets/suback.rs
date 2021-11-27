use crate::packet::{Packet, ReadPacket, WritePacket};
use std::io::{Cursor, Read, Write};
use crate::parser::{decode_remaining_length, encode_remaining_length};
use std::io::{Error, ErrorKind::UnexpectedEof, ErrorKind::Other};

pub const VARIABLE_HEADER_REMAINING_LENGTH: u8 = 2;
pub const SUBACK_PACKET_TYPE: u8 = 0x90;
pub const SUCCESS_MAX_QOS_0: u8 = 0x00;
pub const SUCCESS_MAX_QOS_1: u8 = 0x01;
pub const FAILURE: u8 = 0x80;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SubackReturnCode {
    SuccessAtMostOnce = SUCCESS_MAX_QOS_0,
    SuccessAtLeastOnce = SUCCESS_MAX_QOS_1,
    Failure = FAILURE,
}

impl SubackReturnCode {
    fn read_from(stream: &mut dyn Read) -> Result<SubackReturnCode, Error> {
        let mut return_code_byte = [0u8; 1];
        stream.read_exact(&mut return_code_byte)?;
        match return_code_byte[0] {
            SUCCESS_MAX_QOS_0 => Ok(SubackReturnCode::SuccessAtMostOnce),
            SUCCESS_MAX_QOS_1 => Ok(SubackReturnCode::SuccessAtLeastOnce),
            FAILURE => Ok(SubackReturnCode::Failure),
            _ => return Err(Error::new(Other, "Invalid Return Code"))
        }
    }
}

pub struct Suback {
    packet_id: u16,
    return_codes: Vec<SubackReturnCode>,
}

impl Suback {
    pub fn new(packet_id: u16) -> Suback{
        Suback {
            packet_id,
            return_codes: vec![],
        }
    }

    fn get_remaining_length(&self) -> Result<u32, String> {  
        //VARIABLE HEADER
        let mut length = VARIABLE_HEADER_REMAINING_LENGTH;
        length += self.return_codes.len() as u8;
        Ok(length as u32)
    }

    pub fn add_return_code(&mut self, code: SubackReturnCode){
        self.return_codes.push(code);
    }
}

impl WritePacket for Suback {
    fn write_to(&self, stream: &mut dyn Write) -> Result<(), Box<dyn std::error::Error>> {
        // FIXED HEADER
        // Escribimos el Packet Type
        stream.write_all(&[SUBACK_PACKET_TYPE])?;
        // Escribimos el remaining length
        let remaining_length = self.get_remaining_length();
        let remaining_length_encoded = encode_remaining_length(remaining_length?);
        for byte in remaining_length_encoded {
            stream.write_all(&[byte])?;
        }

        //VARIABLE HEADER
        let packet_id_bytes = self.packet_id.to_be_bytes();
        stream.write_all(&packet_id_bytes)?;

        //PAYLOAD
        for code in &self.return_codes {
            let qos_bytes = (*code as u8).to_be_bytes();
            stream.write_all(&qos_bytes)?;
        }

        Ok(())
    }
}

impl ReadPacket for Suback {
    fn read_from(stream: &mut dyn Read, initial_byte: u8) -> Result<Packet, Box<dyn std::error::Error>> {
        verify_suback_byte(&initial_byte)?;
        let remaining_length = decode_remaining_length(stream)?;

        let mut remaining = vec![0u8; remaining_length as usize];
        stream.read_exact(&mut remaining)?;
        let mut remaining_bytes = Cursor::new(remaining);

        let mut packet_identifier_bytes = [0u8; 2];
        remaining_bytes.read_exact(&mut packet_identifier_bytes)?;
        let packet_identifier = u16::from_be_bytes(packet_identifier_bytes);
    
        let mut suback_packet = Suback::new(packet_identifier);
        loop {
            match SubackReturnCode::read_from(&mut remaining_bytes){
                Err(e) => match e.kind(){
                    UnexpectedEof => return Ok(Packet::Suback(suback_packet)),
                    _ => return Err(Box::new(e)),
                },
                Ok(code) => suback_packet.add_return_code(code),
            }
        }
    }
}

fn verify_suback_byte(byte: &u8) -> Result<(), String>{
    match *byte {
        SUBACK_PACKET_TYPE => return Ok(()),
        _ => return Err("Wrong First Byte".to_string()),
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_remaining_length() {
        let mut suback_packet = Suback::new(50);
        suback_packet.add_return_code(SubackReturnCode::SuccessAtMostOnce);
        suback_packet.add_return_code(SubackReturnCode::SuccessAtLeastOnce);
        suback_packet.add_return_code(SubackReturnCode::Failure);

        let to_test = suback_packet.get_remaining_length().unwrap();
        assert_eq!(to_test, 5);
    }

    #[test]
    fn correct_suback_packet() {
        let mut suback_packet = Suback::new(73);
        suback_packet.add_return_code(SubackReturnCode::SuccessAtLeastOnce);
        suback_packet.add_return_code(SubackReturnCode::SuccessAtMostOnce);
        let mut buff = Cursor::new(Vec::new());
        suback_packet.write_to(&mut buff).unwrap();
        buff.set_position(1);
        let to_test = Suback::read_from(&mut buff, 0x90).unwrap();
        if let Packet::Suback(to_test) = to_test {
            assert_eq!(to_test.packet_id, suback_packet.packet_id);
            assert_eq!(to_test.return_codes, vec![SubackReturnCode::SuccessAtLeastOnce, SubackReturnCode::SuccessAtMostOnce]);
        }
    }

    #[test]
    fn error_wrong_first_byte(){
        let mut suback_packet = Suback::new(73);
        suback_packet.add_return_code(SubackReturnCode::SuccessAtLeastOnce);
        let mut buff = Cursor::new(Vec::new());
        suback_packet.write_to(&mut buff).unwrap();
        buff.set_position(1);
        let to_test = Suback::read_from(&mut buff, 0x91);
        assert!(to_test.is_err());
    }
}