use crate::packet::{Packet, ReadPacket, WritePacket};
use crate::parser::decode_remaining_length;
use crate::parser::encode_remaining_length;
use crate::parser::decode_utf8;
use crate::parser::encode_utf8;

use std::io::{Read, Write};
use std::io::Cursor;

#[derive(Debug)]
pub struct Connect {
    connect_payload: ConnectPayload,
    connect_flags: ConnectFlags,
    // keep alive?
}

impl Connect {
    pub fn new(
        connect_payload: ConnectPayload,
        connect_flags: ConnectFlags,
    ) -> Connect {

        Connect {
            connect_payload,
            connect_flags,
        }
    }

    fn get_remaining_length(&self) -> usize {
        10 + self.connect_payload.length()
    }
}

impl WritePacket for Connect {
    fn write_to(&self, stream: &mut dyn Write) -> Result<(), Box<dyn std::error::Error>> {
        // FIXED HEADER
        // Escribimos el packet type + los flags del packet type
        let packet_type_and_flags = 0x10_u8;
        stream.write(&[packet_type_and_flags])?;

        // Escribimos el remaining length
        let remaining_length = self.get_remaining_length();
        let remaining_length_encoded = encode_remaining_length(remaining_length);
        for byte in remaining_length_encoded {

            stream.write(&[byte])?;
        }

        // VARIABLE HEADER
        // Escribimos los bytes 1-6 correspondientes a la string "MQTT"
        let mqtt_string_bytes: [u8; 6] = [0x00, 0x04, 0x4D, 0x51, 0x54, 0x54];
        for byte in &mqtt_string_bytes {
            stream.write(&[*byte])?;
        }

        // Escribimos el protocol level 4
        let protocol_level = 0x04;
        stream.write(&[protocol_level])?;

        // Escribimos los flags
        self.connect_flags.write_to(stream)?;

        let keep_alive_bytes = [0_u8, 0];
        stream.write(&keep_alive_bytes)?;

        self.connect_payload.write_to(stream)?;
        Ok(())
    }
}

impl ReadPacket for Connect {
    fn read_from(stream: &mut dyn Read) -> Result<Packet, Box<dyn std::error::Error>> {
        let remaining_length = decode_remaining_length(stream)?;

        let mut remaining = vec![0u8; remaining_length as usize];
        stream.read_exact(&mut remaining)?;
        let mut remaining_bytes = Cursor::new(remaining);

        let mut mqtt_string_bytes = [0u8; 6];
        remaining_bytes.read_exact(&mut mqtt_string_bytes)?;
        verify_mqtt_string_bytes(&mqtt_string_bytes)?;

        let mut protocol_level_byte = [0u8; 1];
        remaining_bytes.read_exact(&mut protocol_level_byte)?;
        verify_protocol_level_byte(&protocol_level_byte)?;

        let connect_flags = ConnectFlags::read_from(&mut remaining_bytes)?;
        verify_connect_flags(&connect_flags)?;
        
        let mut keep_alive_bytes = [0u8; 2];
        remaining_bytes.read_exact(&mut keep_alive_bytes)?;

        //Payload: order Client Identifier, Will Topic, Will Message, User Name, Password
        let payload = ConnectPayload::read_from(&mut remaining_bytes, &connect_flags)?;
        verify_payload(&connect_flags, &payload)?;
        
        Ok(Packet::Connect(Connect::new(payload, connect_flags)))
    }
}

fn verify_mqtt_string_bytes(bytes: &[u8; 6]) -> Result<(), String> {
    let mqtt_string_bytes: [u8; 6] = [0x00, 0x04, 0x4D, 0x51, 0x54, 0x54];
    if mqtt_string_bytes != *bytes {
        return Err("No es MQTT".into());
    }

    Ok(())
}

fn verify_protocol_level_byte(byte: &[u8; 1]) -> Result<(), String> {
    if byte[0] != 0x4 {
        return Err("Protocol level byte inválido".into());
    }
    Ok(())
}

fn verify_connect_flags(flags: &ConnectFlags) -> Result<(), String> {
    if flags.last_will_flag == false && (flags.last_will_retain == true || flags.last_will_qos == true) {
        return Err("Last will flags invalidos".into());
    } 
    if flags.last_will_qos == false && flags.last_will_flag == true {
        return Err("Last will flags invalidos".into());
    }
    if flags.username == false && flags.password == true {
        return Err("Username y password flags invalidos".into());
    }

    Ok(())
}

fn verify_payload(flags: &ConnectFlags, payload: &ConnectPayload) -> Result<(), String> {
    if (payload.username.is_some() && flags.username == false) ||
    (payload.username.is_none() && flags.username == true) ||
    (payload.password.is_some() && flags.password == false) ||
    (payload.password.is_none() && flags.password == true) ||
    (payload.last_will_message.is_some() && flags.last_will_flag == false) ||
    (payload.last_will_message.is_none() && flags.last_will_flag == true) ||
    (payload.last_will_topic.is_some() && flags.last_will_flag == false) ||
    (payload.last_will_topic.is_none() && flags.last_will_flag == true) {
        return Err("Payload invalido".into());
    }

    Ok(())
}

/* ------------------------------------------- */
#[derive(PartialEq)]
#[derive(Debug)]
pub struct ConnectFlags {
    username: bool,
    password: bool,
    last_will_retain: bool,
    last_will_qos: bool,
    last_will_flag: bool,
    clean_session: bool,
}

impl ConnectFlags {
    pub fn new(
        username: bool,
        password: bool,
        last_will_retain: bool,
        last_will_qos: bool,
        last_will_flag: bool,
        clean_session: bool,
    ) -> ConnectFlags {
        ConnectFlags {
            username,
            password,
            last_will_retain,
            last_will_qos,
            last_will_flag,
            clean_session,
        }
    }

    fn write_to(&self, stream: &mut dyn Write) -> Result<(), Box<dyn std::error::Error>> {
        let mut result_byte: u8 = 0b0000_0000;
        if self.username {
            result_byte |= 0b1000_0000;
        }
        if self.password {
            result_byte |= 0b0100_0000;
        }
        if self.last_will_retain {
            result_byte |= 0b0010_0000;
        }
        if self.last_will_qos {
            result_byte |= 0b0000_1000;
        }
        if self.last_will_flag {
            result_byte |= 0b0000_0100;
        }
        if self.clean_session {
            result_byte |= 0b0000_0010;
        }
        // The LSB (Reserved) must be 0, so we set it to 0. 
        // As there is no QoS 2, the 4th bit is also set to 0.
        stream.write(&[result_byte])?;
        Ok(())
    }

    fn read_from(stream: &mut dyn Read) -> Result<ConnectFlags, Box<dyn std::error::Error>> {
        let mut flags_byte = [0u8; 1];
        stream.read_exact(&mut flags_byte)?;
        let flags_byte = flags_byte[0];
        let mut flags = [false; 8];

        if flags_byte & 0b1000_0000 == 0b1000_0000 {
            flags[0] = true; // Username flag
        }
        if flags_byte & 0b0100_0000 == 0b0100_0000 {
            flags[1] = true; // Password flag
        }
        if flags_byte & 0b0010_0000 == 0b0010_0000 {
            flags[2] = true; // Last will retain flag
        }
        if flags_byte & 0b0001_0000 == 0b0001_0000 {
            return Err("4th msb of Connect flags is 1, and should be 0 (Quality of Service can be 1 o 0 only)".into())
        }
        if flags_byte & 0b0000_1000 == 0b0000_1000 {
            flags[4] = true; // Last will qos flag
        }
        if flags_byte & 0b0000_0100 == 0b0000_0100 {
            flags[5] = true; // Last will flag
        }
        if flags_byte & 0b0000_0010 == 0b0000_0010 {
            flags[6] = true; // Clean session flag
        }
        if flags_byte & 0b0000_0001 == 0b0000_0001 {
            return Err("Connect flags: Reserved bit should be 0".into())
        }

        Ok(ConnectFlags::new(flags[0], flags[1], flags[2], flags[4], flags[5], flags[6]))
    }


}
#[derive(PartialEq)]
#[derive(Debug)]
pub struct ConnectPayload {
    client_id: String,
    last_will_topic: Option<String>,
    last_will_message: Option<String>,
    username: Option<String>,
    password: Option<String>,
}

impl ConnectPayload {
    pub fn new(
        client_id: String,
        last_will_topic: Option<String>,
        last_will_message: Option<String>,
        username: Option<String>,
        password: Option<String>,
    ) -> ConnectPayload {
        ConnectPayload {
            client_id,
            last_will_topic,
            last_will_message,
            username,
            password,
        }
    }

    fn length(&self) -> usize {
        let mut length = self.client_id.as_bytes().len() + 2;
        if let Some(string) = &self.username {
            length += string.as_bytes().len() + 2;
        }
        if let Some(string) = &self.password {
            length += string.as_bytes().len() + 2;
        }
        if let Some(string) = &self.last_will_topic {
            length += string.as_bytes().len() + 2;
        }
        if let Some(string) = &self.last_will_message {
            length += string.as_bytes().len() + 2;
        }

        length
    }

    fn read_from(stream: &mut dyn Read, flags: &ConnectFlags) -> Result<ConnectPayload, Box<dyn std::error::Error>> {
        let client_id = decode_utf8(stream)?;

        let mut last_will_topic = None;
        let mut last_will_message = None;
        if flags.last_will_flag == true {
            last_will_topic = Some(decode_utf8(stream)?);
            last_will_message = Some(decode_utf8(stream)?);
        }

        let mut username = None;
        let mut password = None;
        if flags.username == true {
            username = Some(decode_utf8(stream)?);
            password = Some(decode_utf8(stream)?);
        }

        Ok(ConnectPayload::new(client_id, last_will_topic, last_will_message, username, password))
    }


    fn write_to(&self, stream: &mut dyn Write) -> Result<(), Box<dyn std::error::Error>> {
        let client_id_utf8 = encode_utf8(&self.client_id)?;
        stream.write(&client_id_utf8)?;

        if let Some(string) = &self.last_will_topic {
            let last_will_topic_utf8 = encode_utf8(string)?;
            stream.write(&last_will_topic_utf8)?; 
        }
        if let Some(string) = &self.last_will_message {
            let last_will_message_utf8 = encode_utf8(string)?;
            stream.write(&last_will_message_utf8)?; 
        }
        if let Some(string) = &self.username {
            let username_utf8 = encode_utf8(string)?;
            stream.write(&username_utf8)?; 
        }
        if let Some(string) = &self.password {
            let password_utf8 = encode_utf8(string)?;
            stream.write(&password_utf8)?; 
        }

        Ok(())
    }
}
/* ------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_protocol_level_byte() {
        let byte: [u8; 1] = [0x4];
        let to_test = verify_protocol_level_byte(&byte);
        assert_eq!(to_test, Ok(()));
    }

    #[test]
    fn error_protocol_level_byte() {
        let byte: [u8; 1] = [0x5];
        let to_test = verify_protocol_level_byte(&byte);

        assert_eq!(to_test, Err("Protocol level byte inválido".to_owned()));
    }

    #[test]
    fn correct_mqtt_string_byte() {
        let bytes: [u8; 6] = [0x00, 0x04, 0x4D, 0x51, 0x54, 0x54];
        let to_test = verify_mqtt_string_bytes(&bytes);
        assert_eq!(to_test, Ok(()));
    }

    #[test]
    fn error_mqtt_string_byte() {
        let bytes: [u8; 6] = [0x00, 0x05, 0x4D, 0x51, 0x54, 0x54];
        let to_test = verify_mqtt_string_bytes(&bytes);
        assert_eq!(to_test, Err("No es MQTT".to_owned()));
    }

    #[test]
    fn correct_connect_flags() {
        let flags = ConnectFlags::new(true, true, true, true, true, true);
        let to_test = verify_connect_flags(&flags);
        assert_eq!(to_test, Ok(()));
    }

    #[test]
    fn error_username_password_flags() {
        let flags = ConnectFlags::new(false, true, true, true, true, true);
        let to_test = verify_connect_flags(&flags);
        assert_eq!(to_test, Err("Username y password flags invalidos".into()));
    }

    #[test]
    fn error_last_will_flags() {
        let flags = ConnectFlags::new(true, true, true, true, false, true);
        let to_test = verify_connect_flags(&flags);
        assert_eq!(to_test, Err("Last will flags invalidos".into()));
    }

    #[test]
    fn correct_payload() {
        let flags = ConnectFlags::new(true, true, true, true, true, true);
        let payload = ConnectPayload::new("u".to_owned(), Some("u".to_owned()), Some("u".to_owned()), Some("u".to_owned()), Some("u".to_owned()));

        let to_test = verify_payload(&flags, &payload);
        assert_eq!(to_test, Ok(()));
    }

    #[test]
    fn error_payload() {
        let flags = ConnectFlags::new(true, true, true, true, true, true);
        let payload = ConnectPayload::new("u".to_owned(), None, Some("u".to_owned()), Some("u".to_owned()), Some("u".to_owned()));

        let to_test = verify_payload(&flags, &payload);
        assert_eq!(to_test, Err("Payload invalido".into()));
    }
    #[test]
    fn correct_packet() {
        let connect_packet = Connect::new(
            ConnectPayload::new("u".to_owned(), Some("u".to_owned()), Some("u".to_owned()), Some("u".to_owned()), Some("u".to_owned())),
            ConnectFlags::new(true, true, true, true, true, true),
        );

        let mut buff = Cursor::new(Vec::new());
        connect_packet.write_to(&mut buff).unwrap();
        buff.set_position(1);
        let to_test = Connect::read_from(&mut buff).unwrap();
        if let Packet::Connect(to_test) = to_test {
            assert!(to_test.connect_flags == connect_packet.connect_flags && 
            to_test.connect_payload == connect_packet.connect_payload)
        }
    }

    #[test]
    fn error_packet() {
        let connect_packet = Connect::new(
            ConnectPayload::new("u".to_owned(), Some("u".to_owned()), Some("u".to_owned()), Some("u".to_owned()), Some("u".to_owned())),
            ConnectFlags::new(false, true, true, true, true, true),
        );

        let mut buff = Cursor::new(Vec::new());
        connect_packet.write_to(&mut buff).unwrap();
        buff.set_position(1);
        let to_test = Connect::read_from(&mut buff);
        assert!(to_test.is_err());
    }
}
