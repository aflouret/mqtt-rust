use crate::packet::{Packet, ReadPacket, WritePacket};
use crate::parser::{
    decode_mqtt_string, decode_remaining_length, encode_mqtt_string, encode_remaining_length,
};
use std::io::{Cursor, Error, ErrorKind::Other, ErrorKind::UnexpectedEof, Read, Write};

pub const UNSUBSCRIBE_PACKET_TYPE: u8 = 0xa0;
const UNSUBSCRIBE_FIRST_BYTE: u8 = 0xa2;
const VARIABLE_HEADER_REMAINING_LENGTH: u8 = 2;

#[derive(Debug)]
pub struct Unsubscribe {
    pub topics: Vec<String>,
    pub packet_id: u16,
}

impl Unsubscribe {
    pub fn new(packet_id: u16) -> Unsubscribe {
        Unsubscribe {
            topics: vec![],
            packet_id,
        }
    }

    pub fn add_topic(&mut self, topic: String) {
        self.topics.push(topic);
    }

    fn get_remaining_length(&self) -> Result<u32, String> {
        //VARIABLE HEADER
        let mut length = VARIABLE_HEADER_REMAINING_LENGTH;

        //PAYLOAD
        for topic in &self.topics {
            length += encode_mqtt_string(&topic)?.len() as u8;
        }

        Ok(length as u32)
    }
}

impl WritePacket for Unsubscribe {
    fn write_to(&self, stream: &mut dyn Write) -> Result<(), Box<dyn std::error::Error>> {
        //FIXED HEADER
        //Escribimos el primer byte (0xa0 | 0x02)
        stream.write_all(&[UNSUBSCRIBE_FIRST_BYTE])?;
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
        for topic in &self.topics {
            let encoded_name = encode_mqtt_string(&topic)?;
            for byte in &encoded_name {
                stream.write_all(&[*byte])?;
            }
        }

        Ok(())
    }
}

impl ReadPacket for Unsubscribe {
    fn read_from(stream: &mut dyn Read, initial_byte: u8) -> Result<Packet, Box<dyn std::error::Error>> {
        verify_unsubscribe_byte(&initial_byte)?;
        let remaining_length = decode_remaining_length(stream)?;

        let mut remaining = vec![0u8; remaining_length as usize];
        stream.read_exact(&mut remaining)?;
        let mut remaining_bytes = Cursor::new(remaining);

        let mut packet_identifier_bytes = [0u8; 2];
        remaining_bytes.read_exact(&mut packet_identifier_bytes)?;
        let packet_identifier = u16::from_be_bytes(packet_identifier_bytes);

        let mut packet_unsubscribe = Unsubscribe::new(packet_identifier);
        loop {
            match decode_mqtt_string(&mut remaining_bytes) {
                Err(e) => {
                    match e.kind() {
                        UnexpectedEof => break,
                        _ => return Err(Box::new(e)),
                    }
                }
                Ok(topic) => packet_unsubscribe.add_topic(topic),
            }
        }

        if packet_unsubscribe.topics.len() == 0 {
            return Err(Box::new(Error::new(Other, "Unsubscribe can't have an empty topic list")));
        } else {
            return Ok(Packet::Unsubscribe(packet_unsubscribe));
        }
    }
}

fn verify_unsubscribe_byte(byte: &u8) -> Result<(), String> {
    match *byte {
        UNSUBSCRIBE_FIRST_BYTE => return Ok(()),
        _ => return Err("Wrong First Byte".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_remaining_length() {
        let mut unsubscribe = Unsubscribe::new(10);
        unsubscribe.add_topic(String::from("test"));
        unsubscribe.add_topic(String::from("otro/topic"));
        let to_test = unsubscribe.get_remaining_length().unwrap();
        assert_eq!(to_test, 20);
    }

    #[test]
    fn correct_unsubscribe_packet() {
        let mut unsubscribe_packet = Unsubscribe::new(73);
        unsubscribe_packet.add_topic(String::from("otro test"));
        let mut buff = Cursor::new(Vec::new());
        unsubscribe_packet.write_to(&mut buff).unwrap();
        buff.set_position(1);
        let to_test = Unsubscribe::read_from(&mut buff, 0xa2).unwrap();
        if let Packet::Unsubscribe(to_test) = to_test {
            assert_eq!(to_test.packet_id, unsubscribe_packet.packet_id);
            if let Some(topic) = unsubscribe_packet.topics.pop() {
                assert_eq!(topic, "otro test".to_string());
            }
        }
    }

    #[test]
    fn error_wrong_first_byte() {
        let mut unsubscribe_packet = Unsubscribe::new(73);
        unsubscribe_packet.add_topic(String::from("otro test"));
        let mut buff = Cursor::new(Vec::new());
        unsubscribe_packet.write_to(&mut buff).unwrap();
        buff.set_position(1);
        let to_test = Unsubscribe::read_from(&mut buff, 0xe1);
        assert_eq!(to_test.unwrap_err().to_string(), "Wrong First Byte");
    }

    #[test]
    fn error_empty_topic_list() {
        let unsubscribe_packet = Unsubscribe::new(73);
        let mut buff = Cursor::new(Vec::new());
        unsubscribe_packet.write_to(&mut buff).unwrap();
        buff.set_position(1);
        let to_test = Unsubscribe::read_from(&mut buff, 0xa2);
        assert_eq!(to_test.unwrap_err().to_string(), "Unsubscribe can't have an empty topic list");
    }
}