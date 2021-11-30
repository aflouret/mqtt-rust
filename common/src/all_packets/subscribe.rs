use crate::packet::{Packet, ReadPacket, WritePacket, Subscription};
use std::io::{Cursor, Read, Write, Error, ErrorKind::UnexpectedEof, ErrorKind::Other};
use crate::parser::{encode_mqtt_string, decode_remaining_length, encode_remaining_length};

pub const SUBSCRIBE_PACKET_TYPE: u8 = 0x80;
const SUBSCRIBE_FIRST_BYTE: u8 = 0x82;
const VARIABLE_HEADER_REMAINING_LENGTH: u8 = 2;

#[derive(Debug)]
pub struct Subscribe {
    pub subscriptions: Vec<Subscription>,
    pub packet_id: u16,
}

impl Subscribe {
    pub fn new(packet_id: u16) -> Subscribe {
        Subscribe {
            subscriptions: vec![],
            packet_id,
        }
    }

    pub fn add_subscription(&mut self, subscription: Subscription){
        self.subscriptions.push(subscription);
    }

    fn get_remaining_length(&self) -> Result<u32, String> {  
        //VARIABLE HEADER
        let mut length = VARIABLE_HEADER_REMAINING_LENGTH;

        //PAYLOAD
        for subscription in &self.subscriptions {
            length += (encode_mqtt_string(&subscription.topic_filter)?.len() + (subscription.max_qos as u8).to_be_bytes().len()) as u8;
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
        // Escribimos el packet id
        let packet_id_bytes = self.packet_id.to_be_bytes();
        stream.write_all(&packet_id_bytes)?;
        
        //PAYLOAD
        for subscription in &self.subscriptions {
            let encoded_name = encode_mqtt_string(&subscription.topic_filter)?;
            for byte in &encoded_name {
                stream.write_all(&[*byte])?;
            }

            let qos_bytes = (subscription.max_qos as u8).to_be_bytes();
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
    
        let mut packet_subscribe = Subscribe::new(packet_identifier);
        loop {
            match Subscription::read_from(&mut remaining_bytes){
                Err(e) => match e.kind(){
                    UnexpectedEof => break,
                    _ => return Err(Box::new(e)),
                },
                Ok(subscription) => packet_subscribe.add_subscription(subscription),
            }
        }

        if packet_subscribe.subscriptions.len() == 0 {
            return Err(Box::new(Error::new(Other, "Subscribe can't have an empty topic list")));
        } else {
            return Ok(Packet::Subscribe(packet_subscribe));
        }
    }
}

fn verify_subscribe_byte(byte: &u8) -> Result<(), String>{
    match *byte {
        SUBSCRIBE_FIRST_BYTE => return Ok(()),
        _ => return Err("Wrong First Byte".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packet::Qos;

    #[test]
    fn correct_remaining_length() {
        let mut subscribe = Subscribe::new(10,);

        subscribe.add_subscription(Subscription{topic_filter: String::from("test"), max_qos: Qos::AtMostOnce});
        subscribe.add_subscription(Subscription{topic_filter: String::from("otro topic"), max_qos: Qos::AtLeastOnce});
        let to_test = subscribe.get_remaining_length().unwrap();
        assert_eq!(to_test, 22);
    }

    #[test]
    fn correct_subscribe_packet() {
        let mut subscribe_packet = Subscribe::new(73);
        subscribe_packet.add_subscription(Subscription{topic_filter: String::from("otro test"), max_qos: Qos::AtLeastOnce});
        let mut buff = Cursor::new(Vec::new());
        subscribe_packet.write_to(&mut buff).unwrap();
        buff.set_position(1);
        let to_test = Subscribe::read_from(&mut buff, 0x82).unwrap();
        if let Packet::Subscribe(to_test) = to_test {
            assert_eq!(to_test.packet_id, subscribe_packet.packet_id);
            if let Some(subscription) = subscribe_packet.subscriptions.pop(){
                assert_eq!(subscription.topic_filter, "otro test".to_string());
                assert_eq!(subscription.max_qos, Qos::AtLeastOnce);
            }
        }
    }

    #[test]
    fn error_wrong_first_byte() {
        let mut subscribe_packet = Subscribe::new(73);
        subscribe_packet.add_subscription(Subscription{topic_filter: String::from("otro test"), max_qos: Qos::AtLeastOnce});
        let mut buff = Cursor::new(Vec::new());
        subscribe_packet.write_to(&mut buff).unwrap();
        buff.set_position(1);
        let to_test = Subscribe::read_from(&mut buff, 0x81);
        assert!(to_test.is_err());
    }

    #[test]
    fn error_empty_topic_list() {
        let subscribe_packet = Subscribe::new(73);
        let mut buff = Cursor::new(Vec::new());
        subscribe_packet.write_to(&mut buff).unwrap();
        buff.set_position(1);
        let to_test = Subscribe::read_from(&mut buff, 0x82);
        assert!(to_test.is_err());
    }
}