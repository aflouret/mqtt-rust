use crate::packet::{Packet, ReadPacket, WritePacket, Qos};
use std::io::{Cursor, Read, Write};
use crate::parser::{encode_mqtt_string, decode_remaining_length, decode_mqtt_string, encode_remaining_length};
use std::io::ErrorKind::UnexpectedEof;

pub const SUBSCRIBE_PACKET_TYPE: u8 = 0x80;
pub const SUBSCRIBE_FIRST_BYTE: u8 = 0x82;
const VARIABLE_HEADER_REMAINING_LENGTH: u8 = 2;

#[derive(Debug)]
pub struct Topic {
    name: String,
    qos: Qos,
}

impl Topic {
    fn read_from(stream: &mut dyn Read) -> Result<Topic, std::io::Error> {
        let topic_name = decode_mqtt_string(stream)?;
        
        let mut qos_level_bytes = [0u8; 1];
        stream.read_exact(&mut qos_level_bytes)?;
        let qos_level = u8::from_be_bytes(qos_level_bytes);

        let qos = match qos_level {
            0 => Qos::AtMostOnce,
            _ => Qos::AtLeastOnce,
        };

        Ok(Topic{name: topic_name, qos})
    }
}

#[derive(Debug)]
pub struct Subscribe {
    topics: Vec<Topic>,
    packet_id: u16,
}

impl Subscribe {
    pub fn new(packet_id: u16, initial_topic: Topic) -> Subscribe {
        Subscribe {
            topics: vec![initial_topic],
            packet_id,
        }
    }

    pub fn add_topic(&mut self, topic: Topic){
        self.topics.push(topic);
    }

    fn get_remaining_length(&self) -> Result<u32, String> {  
        //VARIABLE HEADER
        let mut length = VARIABLE_HEADER_REMAINING_LENGTH;

        //PAYLOAD
        for topic in &self.topics {
            length += (encode_mqtt_string(&topic.name)?.len() + (topic.qos as u8).to_be_bytes().len()) as u8;
        }

        Ok(length as u32)
    }
}

impl WritePacket for Subscribe {
    fn write_to(&self, stream: &mut dyn Write) -> Result<(), Box<dyn std::error::Error>> {
        // FIXED HEADER
        //Escribimos el primer byte (0x80 | 0x02)
        stream.write_all(&[SUBSCRIBE_FIRST_BYTE])?;
        // Escribimos el remaining length
        let remaining_length = self.get_remaining_length();
        let remaining_length_encoded = encode_remaining_length(remaining_length?);
        for byte in remaining_length_encoded {
            stream.write_all(&[byte])?;
        }

        //VARIABLE HEADER
        // Escribimos el packet id (si tiene)
        let packet_id_bytes = self.packet_id.to_be_bytes();
        stream.write_all(&packet_id_bytes)?;
        
        //PAYLOAD
        for topic in &self.topics {
            let encoded_name = encode_mqtt_string(&topic.name)?;
            for byte in &encoded_name {
                stream.write_all(&[*byte])?;
            }

            let qos_bytes = (topic.qos as u8).to_be_bytes();
            stream.write_all(&qos_bytes)?;
        }

        Ok(())
    }
}

impl ReadPacket for Subscribe {
    fn read_from(stream: &mut dyn Read, initial_byte: u8) -> Result<Packet, Box<dyn std::error::Error>> {
        verify_subscribe_byte(&initial_byte)?;
        let remaining_length = decode_remaining_length(stream)?;

        let mut remaining = vec![0u8; remaining_length as usize];
        stream.read_exact(&mut remaining)?;
        let mut remaining_bytes = Cursor::new(remaining);

        let mut packet_identifier_bytes = [0u8; 2];
        remaining_bytes.read_exact(&mut packet_identifier_bytes)?;
        let packet_identifier = u16::from_be_bytes(packet_identifier_bytes);
    
        let initial_topic = Topic::read_from(&mut remaining_bytes)?;
        let mut packet_subscribe = Subscribe::new(packet_identifier, initial_topic);
        loop {
            match Topic::read_from(&mut remaining_bytes){
                Err(e) => match e.kind(){
                    UnexpectedEof => return Ok(Packet::Subscribe(packet_subscribe)),
                    _ => return Err(Box::new(e)),
                }, //read_exact devuelve std::io::ErrorKind::UnexpectedEof
                Ok(topic) => packet_subscribe.add_topic(topic),
            }
        }
    }
}

fn verify_subscribe_byte(byte: &u8) -> Result<(), String>{
    match *byte {
        SUBSCRIBE_FIRST_BYTE => return Ok(()),
        _ => return Err("Wrong Packet Type".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_remaining_length() {
        let mut subscribe = Subscribe::new(
            10,
            Topic{name: String::from("test"), qos: Qos::AtMostOnce}
        );

        subscribe.add_topic(Topic{name: String::from("otro topic"), qos: Qos::AtLeastOnce});
        let to_test = subscribe.get_remaining_length().unwrap();
        assert_eq!(to_test, 22);
    }

    #[test]
    fn correct_subscribe_packet() {
        let mut subscribe_packet = Subscribe::new(
            73, 
            Topic{name: String::from("otro test"), qos: Qos::AtLeastOnce}
        );
        let mut buff = Cursor::new(Vec::new());
        subscribe_packet.write_to(&mut buff).unwrap();
        buff.set_position(1);
        let to_test = Subscribe::read_from(&mut buff, 0x82).unwrap();
        if let Packet::Subscribe(to_test) = to_test {
            assert_eq!(to_test.packet_id, subscribe_packet.packet_id);
            if let Some(topic) = subscribe_packet.topics.pop(){
                assert_eq!(topic.name, "otro test".to_string());
                assert_eq!(topic.qos, Qos::AtLeastOnce);
            }
        }
    }

    #[test]
    fn error_wrong_first_byte() {
        let subscribe_packet = Subscribe::new(
            73,
            Topic{name: String::from("otro test"), qos: Qos::AtLeastOnce}
        );
        let mut buff = Cursor::new(Vec::new());
        subscribe_packet.write_to(&mut buff).unwrap();
        buff.set_position(1);
        let to_test = Subscribe::read_from(&mut buff, 0x81);
        assert!(to_test.is_err());
    }
}