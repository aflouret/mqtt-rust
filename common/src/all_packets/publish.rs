use crate::packet::{Packet, ReadPacket, WritePacket};
use std::io::Cursor;
use std::io::{Read, Write};
use crate::parser::decode_remaining_length;
use crate::parser::decode_mqtt_string;
use crate::parser::encode_remaining_length;
use crate::parser::encode_mqtt_string;

pub const PUBLISH_PACKET_TYPE: u8 = 0x30;

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Qos {
    AtMostOnce = 0,
    AtLeastOnce = 1,
    //ExactlyOnce = 2,
}
#[derive(Debug)]
pub struct Publish {
    flags: PublishFlags,
    topic_name: String,
    packet_id: Option<u16>,
    application_message: String,
}

impl Publish {
    pub fn new(flags: PublishFlags, topic_name: String, packet_id: Option<u16>, application_message: String) -> Publish {
        Publish {
            flags,
            topic_name,
            packet_id,
            application_message,
        }
    }

    fn get_remaining_length(&self) -> Result<u32, String> {  
        //VARIABLE HEADER
        let mut length = encode_mqtt_string(&self.topic_name)?.len();
        if let Some(packet_identifier) = self.packet_id{
            length += packet_identifier.to_be_bytes().len();
        }

        //PAYLOAD
        length += encode_mqtt_string(&self.application_message)?.len();

        Ok(length as u32)
    }
}

impl WritePacket for Publish {
    fn write_to(&self, stream: &mut dyn Write) -> Result<(), Box<dyn std::error::Error>> {
        // FIXED HEADER
        // Escribimos el packet type + los flags del publish packet
        let first_byte = PUBLISH_PACKET_TYPE | 
        (self.flags.duplicate as u8) << 3 | 
        (self.flags.qos_level as u8) << 1 |
        (self.flags.retain as u8);
        stream.write(&[first_byte])?;

        // Escribimos el remaining length
        let remaining_length = self.get_remaining_length();
        let remaining_length_encoded = encode_remaining_length(remaining_length?);
        for byte in remaining_length_encoded {
            stream.write(&[byte])?;
        }

        // VARIABLE HEADER
        // Escribimos el topic name
        let encoded_topic_name = encode_mqtt_string(&self.topic_name)?;
        for byte in &encoded_topic_name {
            stream.write(&[*byte])?;
        }

        // Escribimos el packet id (si tiene)
        if let Some(packet_identifier) = self.packet_id {
            let packet_id_bytes = packet_identifier.to_be_bytes();
            stream.write(&packet_id_bytes)?;
        }

        //PAYLOAD
        // Escribimos el mensaje
        let encoded_message = encode_mqtt_string(&self.application_message)?;
        for byte in &encoded_message {
            stream.write(&[*byte])?;
        }

        Ok(())
    }
}

impl ReadPacket for Publish {
    fn read_from(stream: &mut dyn Read, initial_byte: u8) -> Result<Packet, Box<dyn std::error::Error>> {
        
        let publish_flags = PublishFlags::new(initial_byte);
        let remaining_length = decode_remaining_length(stream)?;

        let mut remaining = vec![0u8; remaining_length as usize];
        stream.read_exact(&mut remaining)?;
        let mut remaining_bytes = Cursor::new(remaining);

        let mut topic_name = decode_mqtt_string(&mut remaining_bytes)?;

        let packet_id = match publish_flags.qos_level {
            Qos::AtLeastOnce => {
                let mut bytes = [0u8; 2];
                stream.read_exact(&mut bytes)?;
                Some(((bytes[0] as u16) << 8) | bytes[1] as u16)
            }
            
            _ => None
        };

        let mut application_message = decode_mqtt_string(&mut remaining_bytes)?;

        Ok(Packet::Publish(Publish::new(
            publish_flags,
            topic_name,
            packet_id,
            application_message
        )))
        
    }
}
#[derive(Debug)]
pub struct PublishFlags {
    duplicate: bool,
    qos_level: Qos,
    retain: bool,
}

impl PublishFlags {
    pub fn new(initial_byte: u8) -> PublishFlags {
        let retain = (initial_byte & 0x01) != 0;
        let duplicate = (initial_byte & 0x08) != 0;
        let qos_level = match initial_byte & 0x02 >> 1  {
            0 => Qos::AtMostOnce,
            _ => Qos::AtLeastOnce,
        };

        PublishFlags {
            duplicate,
            qos_level,
            retain,
        }
    }
}

/* ------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_remaining_length_qos0() {
        let publish = Publish::new(
            PublishFlags::new(0b0100_1011),
            "Topic".to_string(),
            None,
            "Message".to_string(),
        );

        let to_test = publish.get_remaining_length().unwrap();
        assert_eq!(to_test, 16);

    }

    #[test]
    fn correct_remaining_length_qos1() {
        let publish = Publish::new(
            PublishFlags::new(0b0100_1011),
            "Topic".to_string(),
            Some(15),
            "Message".to_string(),
        );
        println!("{:?}",publish);
        let to_test = publish.get_remaining_length().unwrap();
        assert_eq!(to_test, 18);

    }
}
