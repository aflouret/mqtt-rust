use crate::packet::{Packet, ReadPacket, WritePacket};
use crate::parser::decode_remaining_length;
use crate::parser::encode_remaining_length;
use crate::parser::decode_utf8;
use std::io::{Read, Write};
use std::io::Cursor;

/*
pub struct Connect {
    client_id: String,
    username: Option<String>,
    password: Option<String>,
    connect_flags: ConnectFlags,
    last_will_message: Option<String>,
    last_will_topic: Option<String>,
    // keep alive?
}
*/

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

    /*
    pub fn new(
        client_id: String,
        username: Option<String>,
        password: Option<String>,
        connect_flags: ConnectFlags,
        last_will_message: Option<String>,
        last_will_topic: Option<String>,
    ) -> Connect {
        Connect {
            client_id,
            username,
            password,
            connect_flags,
            last_will_message,
            last_will_topic,
        }
    }
    */

    fn get_remaining_length(&self) -> u32 {
        //TODO: obtener el r.l. de esta instancia del packet
        268435455 
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
        for byte in mqtt_string_bytes {
            stream.write(&[byte])?;
        }

        // Escribimos el protocol level 4
        let protocol_level = 0x04;
        stream.write(&[protocol_level])?;

        // Escribimos los flags
        self.connect_flags.write_to(stream)?;

        // TODO: keep alive

        // TODO: payload (username, etc etc de acuerdo al remaining length)

        //TODO: calcular el remaining length

        // TODO: validar las cominaciones de flags...
        Ok(())
    }
}

impl ReadPacket for Connect {
    fn read_from(stream: &mut dyn Read) -> Result<Packet, Box<dyn std::error::Error>> {
        let remaining_length = decode_remaining_length(stream)?;
        //println!("Remaining length decodificado: {}", remaining_length);

        //nuevo
        let mut remaining = vec![0u8; remaining_length as usize];
        stream.read_exact(&mut remaining)?;
        let mut remaining_bytes = Cursor::new(remaining);
        //fin de nuevo

        /*
        let mut mqtt_string_bytes = [0u8; 6];
        stream.read_exact(&mut mqtt_string_bytes)?;
        verify_mqtt_string_bytes(&mqtt_string_bytes)?;
        */
        let mut mqtt_string_bytes = [0u8; 6];
        remaining_bytes.read_exact(&mut mqtt_string_bytes)?;
        verify_mqtt_string_bytes(&mqtt_string_bytes)?;

        /*
        let mut protocol_level_byte = [0u8; 1];
        stream.read_exact(&mut protocol_level_byte)?;
        verify_protocol_level_byte(&protocol_level_byte)?;
        */
        let mut protocol_level_byte = [0u8; 1];
        remaining_bytes.read_exact(&mut protocol_level_byte)?;
        verify_protocol_level_byte(&protocol_level_byte)?;

        //let connect_flags = ConnectFlags::read_from(stream)?;
        let connect_flags = ConnectFlags::read_from(&mut remaining_bytes)?;

        //Payload: order Client Identifier, Will Topic, Will Message, User Name, Password
        let payload = ConnectPayload::read_from(&mut remaining_bytes, &connect_flags)?;

        /*
        Ok(Packet::Connect(Connect::new(
            "123".to_string(),
            Some("asd".to_string()),
            Some("asd".to_string()),
            connect_flags,
            Some("asd".to_string()),
            Some("asd".to_string()),
        )))
        */
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

/* ------------------------------------------- */
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
        if self.last_will_flag {
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

pub struct ConnectPayload {
    client_id: String,
    username: Option<String>,
    password: Option<String>,
    last_will_topic: Option<String>,
    last_will_message: Option<String>,
}

impl ConnectPayload {
    pub fn new(
        client_id: String,
        username: Option<String>,
        password: Option<String>,
        last_will_topic: Option<String>,
        last_will_message: Option<String>,
    ) -> ConnectPayload {
        ConnectPayload {
            client_id,
            username,
            password,
            last_will_topic,
            last_will_message,
        }
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
}
