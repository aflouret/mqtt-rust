use crate::packet::{Packet, ReadPacket, WritePacket, Qos};
use std::io::Cursor;
use std::io::{Read, Write};
use crate::parser::decode_remaining_length;
use crate::parser::decode_mqtt_string;
use crate::parser::encode_remaining_length;
use crate::parser::encode_mqtt_string;

pub const PUBLISH_PACKET_TYPE: u8 = 0x30;

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
        stream.write_all(&[first_byte])?;
        // Escribimos el remaining length
        let remaining_length = self.get_remaining_length();
        let remaining_length_encoded = encode_remaining_length(remaining_length?);
        for byte in remaining_length_encoded {
            stream.write_all(&[byte])?;
        }

        // VARIABLE HEADER
        // Escribimos el topic name
        let encoded_topic_name = encode_mqtt_string(&self.topic_name)?;
        for byte in &encoded_topic_name {
            stream.write_all(&[*byte])?;
        }

        // Escribimos el packet id (si tiene)
        if let Some(packet_identifier) = self.packet_id {
            let packet_id_bytes = packet_identifier.to_be_bytes();
            stream.write_all(&packet_id_bytes)?;
        }

        //PAYLOAD
        // Escribimos el mensaje
        let encoded_message = encode_mqtt_string(&self.application_message)?;
        for byte in &encoded_message {
            stream.write_all(&[*byte])?;
        }
        println!("Escribo bien el publish");
        Ok(())
    }
}

impl ReadPacket for Publish {
    fn read_from(stream: &mut dyn Read, initial_byte: u8) -> Result<Packet, Box<dyn std::error::Error>> {
        let publish_flags = PublishFlags::new(initial_byte);
        verify_publish_flags(&publish_flags)?;

        let remaining_length = decode_remaining_length(stream)?;
        let mut remaining = vec![0u8; remaining_length as usize];
        stream.read_exact(&mut remaining)?;
        let mut remaining_bytes = Cursor::new(remaining);

        let topic_name = decode_mqtt_string(&mut remaining_bytes)?;

        let packet_id = match publish_flags.qos_level {
            Qos::AtLeastOnce => {
                let mut bytes = [0u8; 2];
                remaining_bytes.read_exact(&mut bytes)?;
                Some(u16::from_be_bytes(bytes))
            }
            
            _ => None
        };

        let application_message = decode_mqtt_string(&mut remaining_bytes)?;
        Ok(Packet::Publish(Publish::new(
            publish_flags,
            topic_name,
            packet_id,
            application_message
        )))
    }
}

fn verify_publish_flags(flags: &PublishFlags) -> Result<(), String> {
    if flags.qos_level == Qos::AtMostOnce && flags.duplicate {
        return Err("The DUP flag MUST be set to 0 for all QoS 0 messages".into());
    }

    Ok(())
}

#[derive(Debug)]
pub struct PublishFlags {
    duplicate: bool,
    qos_level: Qos,
    retain: bool,
}

//Falta checkear: If a Server or Client receives a PUBLISH Packet which has both QoS bits set to 1 it MUST close the Network Connection
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

    #[test]
    fn correct_new_publishflag_all_true() {
        
        let to_test = PublishFlags::new(0b0100_1011);

        assert_eq!(to_test.duplicate, true);
        assert_eq!(to_test.qos_level, Qos::AtLeastOnce);
        assert_eq!(to_test.retain, true);
    }

    #[test]
    fn correct_new_publishflag_all_false() {
        let to_test = PublishFlags::new(0b0100_0000);

        assert_eq!(to_test.duplicate, false);
        assert_eq!(to_test.qos_level, Qos::AtMostOnce);
        assert_eq!(to_test.retain, false);
    }
    
    #[test]
    fn correct_packet() {
        let publish_packet = Publish::new(
            PublishFlags::new(0b0100_1011),
            "Topic".to_string(),
            Some(15),
            "Message".to_string(),
        );

        let mut buff = Cursor::new(Vec::new());
        publish_packet.write_to(&mut buff).unwrap();
        buff.set_position(1);
        let to_test = Publish::read_from(&mut buff, 0x4b).unwrap();
        if let Packet::Publish(to_test) = to_test {
            assert!(
                    to_test.topic_name == publish_packet.topic_name
                    && to_test.packet_id == publish_packet.packet_id
                    && to_test.application_message == publish_packet.application_message
            )
        }
    }

    #[test]
    fn error_packet() {
        let publish_packet = Publish::new(
            PublishFlags::new(0b0100_1011),
            "Topic".to_string(),
            Some(15),
            "Message".to_string(),
        );

        let mut buff = Cursor::new(Vec::new());
        publish_packet.write_to(&mut buff).unwrap();
        buff.set_position(1);
        let to_test = Publish::read_from(&mut buff, 0x10);
        assert!(to_test.is_err());
    }
}
