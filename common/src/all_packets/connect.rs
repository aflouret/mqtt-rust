use crate::packet::{Packet, ReadPacket, WritePacket};
use crate::parser::decode_remaining_length;
use crate::parser::decode_mqtt_string;
use crate::parser::encode_remaining_length;
use crate::parser::encode_mqtt_string;

use std::io::Cursor;
use std::io::{Read, Write};

const PROTOCOL_NAME: &str = "MQTT";
pub const CONNECT_PACKET_TYPE: u8 = 0x10;
const CONNECT_VARIABLE_HEADER_BYTES: u32 = 10;
const CONNECT_PROTOCOL_LEVEL: u8 = 0x04;
const USERNAME_FLAG: u8 = 0b1000_0000;
const PASSWORD_FLAG: u8 = 0b0100_0000;
const LAST_WILL_RETAIN_FLAG: u8 = 0b0010_0000;
const LAST_WILL_QOS_MSB_FLAG: u8 = 0b0001_0000;
const LAST_WILL_QOS_LSB_FLAG: u8 = 0b0000_1000;
const LAST_WILL_FLAG: u8 = 0b0000_0100;
const CLEAN_SESSION_FLAG: u8 = 0b0000_0010;
const RESERVED_BIT: u8 = 0b0000_0001;

pub const INCORRECT_PROTOCOL_NAME_ERROR_MSG: &str = "Disconnect: Incorrect Protocol Name";
pub const INCORRECT_PROTOCOL_LEVEL_ERROR_MSG: &str = "SendConnackAndDisconnect: Incorrect Protocol Level";

#[derive(Debug)]
pub struct Connect {
    pub connect_payload: ConnectPayload,
    pub keep_alive_seconds: u16,
    pub clean_session: bool,
    pub last_will_retain: bool,
    pub last_will_qos: bool,
}

impl Connect {
    pub fn new(
        connect_payload: ConnectPayload,
        keep_alive_seconds: u16,
        clean_session: bool,
        last_will_retain: bool,
        last_will_qos: bool
    ) -> Connect {
        Connect {
            connect_payload,
            keep_alive_seconds,
            clean_session,
            last_will_retain,
            last_will_qos,
        }
    }

    fn get_remaining_length(&self) -> Result<u32, String> {  
        //Variable header bytes + Payload bytes
        Ok(CONNECT_VARIABLE_HEADER_BYTES + self.connect_payload.length()?)
    }

    fn write_flags_to(&self, stream: &mut dyn Write) -> Result <(), Box<dyn std::error::Error>>{
        let mut result_byte: u8 = 0b0000_0000;
        if self.connect_payload.username.is_some() {
            result_byte |= USERNAME_FLAG;
        }
        if self.connect_payload.password.is_some() {
            result_byte |= PASSWORD_FLAG;
        }
        if self.last_will_retain {
            result_byte |= LAST_WILL_RETAIN_FLAG;
        }
        if self.last_will_qos {
            result_byte |= LAST_WILL_QOS_LSB_FLAG;
        }
        if self.connect_payload.last_will_message.is_some() {
            result_byte |= LAST_WILL_FLAG;
        }
        if self.clean_session {
            result_byte |= CLEAN_SESSION_FLAG;
        }
        // The LSB (Reserved) must be 0, so we set it to 0.
        // As there is no QoS 2, the 4th bit is also set to 0.
        stream.write_all(&[result_byte])?;
        Ok(())
    }
}

impl WritePacket for Connect {
    fn write_to(&self, stream: &mut dyn Write) -> Result<(), Box<dyn std::error::Error>> {
        // FIXED HEADER
        // Escribimos el packet type + los flags del packet type
        stream.write_all(&[CONNECT_PACKET_TYPE])?;

        // Escribimos el remaining length
        let remaining_length = self.get_remaining_length();
        let remaining_length_encoded = encode_remaining_length(remaining_length?);
        for byte in remaining_length_encoded {
            stream.write_all(&[byte])?;
        }

        // VARIABLE HEADER
        let encoded_protocol_name = encode_mqtt_string(PROTOCOL_NAME)?;

        for byte in &encoded_protocol_name {
            stream.write_all(&[*byte])?;
        }

        // Escribimos el protocol level 4
        stream.write_all(&[CONNECT_PROTOCOL_LEVEL])?;

        // Escribimos los flags
        self.write_flags_to(stream)?;

        let keep_alive_bytes = self.keep_alive_seconds.to_be_bytes();
        stream.write_all(&keep_alive_bytes)?;

        self.connect_payload.write_to(stream)?;
        
        Ok(())
    }
}

impl ReadPacket for Connect {
    fn read_from(stream: &mut dyn Read, _initial_byte: u8) -> Result<Packet, Box<dyn std::error::Error>> {
        let remaining_length = decode_remaining_length(stream)?;

        let mut remaining = vec![0u8; remaining_length as usize];
        stream.read_exact(&mut remaining)?;
        let mut remaining_bytes = Cursor::new(remaining);

        let mut mqtt_string_bytes = [0u8; 6];
        remaining_bytes.read_exact(&mut mqtt_string_bytes)?;
        verify_protocol_name_bytes(&mqtt_string_bytes)?; 

        let mut protocol_level_byte = [0u8; 1];
        remaining_bytes.read_exact(&mut protocol_level_byte)?;
        verify_protocol_level_byte(&protocol_level_byte)?;

        let connect_flags = ConnectFlags::read_from(&mut remaining_bytes)?;
        verify_connect_flags(&connect_flags)?;

        let mut keep_alive_bytes = [0u8; 2];
        remaining_bytes.read_exact(&mut keep_alive_bytes)?;
        let keep_alive_seconds = u16::from_be_bytes(keep_alive_bytes);

        //Payload: order Client Identifier, Will Topic, Will Message, User Name, Password
        let payload = ConnectPayload::read_from(&mut remaining_bytes, &connect_flags)?;
        verify_payload(&connect_flags, &payload)?;

        println!("Connect packet leido correctamente");

        Ok(Packet::Connect(Connect::new(
            payload,
            keep_alive_seconds,
            connect_flags.clean_session,
            connect_flags.last_will_retain,
            connect_flags.last_will_qos
        )))
    }
}

fn verify_protocol_name_bytes(bytes: &[u8; 6]) -> Result<(), String> {
    let mqtt_string_bytes = encode_mqtt_string(PROTOCOL_NAME)?;
    if mqtt_string_bytes != *bytes {
        // [MQTT-3.1.2-1]. 
        return Err(INCORRECT_PROTOCOL_NAME_ERROR_MSG.into());
    }

    Ok(())
}

fn verify_protocol_level_byte(byte: &[u8; 1]) -> Result<(), String> {
    //TODO: The Server MUST respond to the 401 CONNECT Packet with a CONNACK return code 0x01 
    // (unacceptable protocol level) and then disconnect 402 the Client if the Protocol Level is not supported by the Server
    if byte[0] != CONNECT_PROTOCOL_LEVEL {
        return Err(INCORRECT_PROTOCOL_LEVEL_ERROR_MSG.into());
    }
    Ok(())
}

fn verify_connect_flags(flags: &ConnectFlags) -> Result<(), String> {
    if !flags.last_will_flag && (flags.last_will_retain || flags.last_will_qos ) {
        return Err("Invalid last will flags".into());
    }
    if !flags.last_will_qos && flags.last_will_flag {
        return Err("Invalid last will flags".into());
    }
    if !flags.username && flags.password {
        return Err("Invalid Username and Password flags".into());
    }

    Ok(())
}

fn verify_payload(flags: &ConnectFlags, payload: &ConnectPayload) -> Result<(), String> {
    if (payload.username.is_some() && !flags.username)
        || (payload.username.is_none() && flags.username)
        || (payload.password.is_some() && !flags.password)
        || (payload.password.is_none() && flags.password)
        || (payload.last_will_message.is_some() && !flags.last_will_flag)
        || (payload.last_will_message.is_none() && flags.last_will_flag)
        || (payload.last_will_topic.is_some() && !flags.last_will_flag)
        || (payload.last_will_topic.is_none() && flags.last_will_flag)
    {
        return Err("Invalid Payload".into());
    }

    Ok(())
}

/* ------------------------------------------- */
#[derive(PartialEq, Debug)]
struct ConnectFlags {
    pub username: bool,
    pub password: bool,
    last_will_retain: bool,
    last_will_qos: bool,
    last_will_flag: bool,
    pub clean_session: bool,
}

impl ConnectFlags {
    fn new(
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

    fn read_from(stream: &mut dyn Read) -> Result<ConnectFlags, Box<dyn std::error::Error>> {
        let mut flags_byte = [0u8; 1];
        stream.read_exact(&mut flags_byte)?;
        let flags_byte = flags_byte[0];
        let mut flags = [false; 8];

        if flags_byte & USERNAME_FLAG == USERNAME_FLAG {
            flags[0] = true; // Username flag
        }
        if flags_byte & PASSWORD_FLAG == PASSWORD_FLAG {
            flags[1] = true; // Password flag
        }
        if flags_byte & LAST_WILL_RETAIN_FLAG == LAST_WILL_RETAIN_FLAG {
            flags[2] = true; // Last will retain flag
        }
        if flags_byte & LAST_WILL_QOS_MSB_FLAG == LAST_WILL_QOS_MSB_FLAG {
            return Err("4th msb of Connect flags is 1, and should be 0 (Quality of Service can be 1 o 0 only)".into());
        }
        if flags_byte & LAST_WILL_QOS_LSB_FLAG == LAST_WILL_QOS_LSB_FLAG {
            flags[4] = true; // Last will qos flag
        }
        if flags_byte & LAST_WILL_FLAG == LAST_WILL_FLAG {
            flags[5] = true; // Last will flag
        }
        if flags_byte & CLEAN_SESSION_FLAG == CLEAN_SESSION_FLAG {
            flags[6] = true; // Clean session flag
        }
        if flags_byte & RESERVED_BIT == RESERVED_BIT {
            return Err("Connect flags: Reserved bit should be 0".into());
        }

        Ok(ConnectFlags::new(
            flags[0], flags[1], flags[2], flags[4], flags[5], flags[6],
        ))
    }
}
#[derive(PartialEq, Debug)]
pub struct ConnectPayload {
    pub client_id: String,
    pub last_will_topic: Option<String>,
    pub last_will_message: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
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

    fn length(&self) -> Result<u32, String> {
        let mut length = encode_mqtt_string(&self.client_id)?.len();
        if let Some(string) = &self.username {
            length += encode_mqtt_string(string)?.len();
        }
        if let Some(string) = &self.password {
            length += encode_mqtt_string(string)?.len();
        }
        if let Some(string) = &self.last_will_topic {
            length += encode_mqtt_string(string)?.len();
        }
        if let Some(string) = &self.last_will_message {
            length += encode_mqtt_string(string)?.len();
        }

        Ok(length as u32)
    }

    fn read_from(
        stream: &mut dyn Read,
        flags: &ConnectFlags,
    ) -> Result<ConnectPayload, Box<dyn std::error::Error>> {
        let client_id = decode_mqtt_string(stream)?;

        let mut last_will_topic = None;
        let mut last_will_message = None;
        if flags.last_will_flag {
            last_will_topic = Some(decode_mqtt_string(stream)?);
            last_will_message = Some(decode_mqtt_string(stream)?);
        }

        let mut username = None;
        let mut password = None;
        if flags.username {
            username = Some(decode_mqtt_string(stream)?);
            password = Some(decode_mqtt_string(stream)?);
        }

        Ok(ConnectPayload::new(
            client_id,
            last_will_topic,
            last_will_message,
            username,
            password,
        ))
    }

    fn write_to(&self, stream: &mut dyn Write) -> Result<(), Box<dyn std::error::Error>> {
        let client_id_utf8 = encode_mqtt_string(&self.client_id)?;
        stream.write_all(&client_id_utf8)?;

        if let Some(string) = &self.last_will_topic {
            let last_will_topic_utf8 = encode_mqtt_string(string)?;
            stream.write_all(&last_will_topic_utf8)?;
        }
        if let Some(string) = &self.last_will_message {
            let last_will_message_utf8 = encode_mqtt_string(string)?;
            stream.write_all(&last_will_message_utf8)?;
        }
        if let Some(string) = &self.username {
            let username_utf8 = encode_mqtt_string(string)?;
            stream.write_all(&username_utf8)?;
        }
        if let Some(string) = &self.password {
            let password_utf8 = encode_mqtt_string(string)?;
            stream.write_all(&password_utf8)?;
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

        assert_eq!(to_test, Err(INCORRECT_PROTOCOL_LEVEL_ERROR_MSG.to_owned()));
    }

    #[test]
    fn correct_mqtt_string_byte() {
        let bytes: [u8; 6] = [0x00, 0x04, 0x4D, 0x51, 0x54, 0x54];
        let to_test = verify_protocol_name_bytes(&bytes);
        assert_eq!(to_test, Ok(()));
    }

    #[test]
    fn error_mqtt_string_byte() {
        let bytes: [u8; 6] = [0x00, 0x05, 0x4D, 0x51, 0x54, 0x54];
        let to_test = verify_protocol_name_bytes(&bytes);
        assert_eq!(to_test, Err(INCORRECT_PROTOCOL_NAME_ERROR_MSG.to_owned()));
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
        assert_eq!(to_test, Err("Invalid Username and Password flags".into()));
    }

    #[test]
    fn error_last_will_flags() {
        let flags = ConnectFlags::new(true, true, true, true, false, true);
        let to_test = verify_connect_flags(&flags);
        assert_eq!(to_test, Err("Invalid last will flags".into()));
    }

    #[test]
    fn correct_payload() {
        let flags = ConnectFlags::new(true, true, true, true, true, true);
        let payload = ConnectPayload::new(
            "u".to_owned(),
            Some("u".to_owned()),
            Some("u".to_owned()),
            Some("u".to_owned()),
            Some("u".to_owned()),
        );

        let to_test = verify_payload(&flags, &payload);
        assert_eq!(to_test, Ok(()));
    }

    #[test]
    fn error_payload() {
        let flags = ConnectFlags::new(true, true, true, true, true, true);
        let payload = ConnectPayload::new(
            "u".to_owned(),
            None,
            Some("u".to_owned()),
            Some("u".to_owned()),
            Some("u".to_owned()),
        );

        let to_test = verify_payload(&flags, &payload);
        assert_eq!(to_test, Err("Invalid Payload".into()));
    }
    
    #[test]
    fn correct_packet() {
        let connect_packet = Connect::new(
            ConnectPayload::new(
                "u".to_owned(),
                Some("u".to_owned()),
                Some("u".to_owned()),
                Some("u".to_owned()),
                Some("u".to_owned()),
            ),
            60,
            true,
            true,
            true,
        );

        let mut buff = Cursor::new(Vec::new());
        connect_packet.write_to(&mut buff).unwrap();
        buff.set_position(1);
        let to_test = Connect::read_from(&mut buff, 0x10).unwrap();
        if let Packet::Connect(to_test) = to_test {
            assert!(
                    to_test.connect_payload == connect_packet.connect_payload
                    && to_test.keep_alive_seconds == 60
                    && to_test.clean_session == true
                    && to_test.last_will_retain == true
                    && to_test.last_will_qos == true
            )
        }
    }

    #[test]
    fn error_packet() {
        let connect_packet = Connect::new(
            ConnectPayload::new(
                "u".to_owned(),
                Some("u".to_owned()),
                Some("u".to_owned()),
                None,
                Some("u".to_owned()),
            ),
            60,
            true,
            true,
            true,
        );

        let mut buff = Cursor::new(Vec::new());
        connect_packet.write_to(&mut buff).unwrap();
        buff.set_position(1);
        let to_test = Connect::read_from(&mut buff, 0x10);
        assert_eq!(to_test.unwrap_err().to_string(), "Invalid Username and Password flags");
    }
}
