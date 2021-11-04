use crate::packet::{Packet, ReadPacket, WritePacket};
use std::io::{Read, Write};
use crate::parser::decode_remaining_length;
use crate::parser::decode_mqtt_string;
use crate::parser::encode_remaining_length;
use crate::parser::encode_mqtt_string;

pub const PUBLISH_PACKET_TYPE: u8 = 0x30;

#[repr(u8)]
pub enum Qos {
    AtMostOnce = 0,
    AtLeastOnce = 1,
    ExactlyOnce = 2,
}

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
    fn read_from(stream: &mut dyn Read) -> Result<Packet, Box<dyn std::error::Error>> {
        /*
        PROBLEMA CON RESERVED BYTES: EL BYTE QUE LEEMOS PARA HACER EL MATCH INICIAL CONTIENE
        LOS FLAGS, Y ADEMAS ES EL ÚNICO PAQUETE QUE HACE ESO :/
        */
        Ok(Packet::Publish(Publish::new(
            PublishFlags::new(true, true, true),
            "hola".to_owned(),
            Some(1),
            "chau".to_owned(),
        )))
        
    }
}

pub struct PublishFlags {
    duplicate: bool,
    qos_level: Qos,
    retain: bool,
}

impl PublishFlags {
    pub fn new(duplicate: bool, qos_level: Qos, retain: bool,) -> PublishFlags {
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
            PublishFlags::new(true, true, true),
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
            PublishFlags::new(true, true, true),
            "Topic".to_string(),
            Some(15),
            "Message".to_string(),
        );

        let to_test = publish.get_remaining_length().unwrap();
        assert_eq!(to_test, 18);

    }
}
