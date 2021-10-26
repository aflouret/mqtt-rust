use crate::packet::{Packet, ReadPacket, WritePacket};
use crate::packet_flags::ConnectFlags;
use crate::parser::decode_remaining_length;
use crate::parser::encode_remaining_length;
use std::io::{Read, Write};
use std::vec;

pub struct Connect {
    client_id: String,
    username: String,
    password: String,
    //connect_flags: ConnectFlags,
    connect_flags: String,
    last_will_message: String,
    last_will_topic: String,
    // keep alive?
}

impl Connect {
    pub fn new(
        client_id: String,
        username: String,
        password: String,
        // connect_flags: ConnectFlags,
        connect_flags: String,
        last_will_message: String,
        last_will_topic: String,
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

    fn get_remaining_length(&self) -> u32 {
        //TODO: obtener el r.l. de esta instancia del packet
        //10 
        2097151
        //268435455 // --> está dando error en el decode
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

        Ok(())
    }
}

impl ReadPacket for Connect {
    fn read_from(stream: &mut dyn Read) -> Result<Packet, Box<dyn std::error::Error>> {
        println!("Entro a connect");

        let remaining_length = decode_remaining_length(stream)?;
        println!("Remaining length decodificado: {}", remaining_length);

        let mut mqtt_string_bytes = [0u8; 6];
        stream.read_exact(&mut mqtt_string_bytes)?;
        verify_mqtt_string_bytes(&mqtt_string_bytes)?;
        println!("MQTT string bytes leidos");

        let mut protocol_level_byte = [0u8; 1];
        stream.read_exact(&mut protocol_level_byte)?;
        verify_protocol_level_byte(&protocol_level_byte)?;
        println!("Protocol level byte leido");

        // let mut flags_byte = [0u8; 1];
        // stream.read_exact(&mut flags_byte)?;
        //TODO: self.set_flags(flags_byte)?; en adelante

        Ok(Packet::Connect(Connect::new(
            "123".to_string(),
            "asd".to_string(),
            "awd".to_string(),
            //ConnectFlags::new(false, false, false, false, false, false),
            "connect_flags".to_string(),
            "last".to_string(),
            "sdt".to_string(),
        )))
    }
}

fn verify_mqtt_string_bytes(bytes: &[u8; 6]) -> Result<(), String> {
    let mqtt_string_bytes: [u8; 6] = [0x00, 0x04, 0x4D, 0x51, 0x54, 0x54];
    for (i, byte) in bytes.iter().enumerate() {
        if *byte != mqtt_string_bytes[i] {
            return Err("No es MQTT".into());
        }
    }
    // if bytes[0] != 0x4d || bytes[1] != 0x51 || bytes[2] != 0x54 || bytes[3] != 0x54
    Ok(())
}

fn verify_protocol_level_byte(byte: &[u8; 1]) -> Result<(), String> {
    if byte[0] != 0x4 {
        return Err("Protocol level byte inválido".into());
    }
    Ok(())
}

//-------------------------------------------------------------------------
/*
pub struct ConnectBuilder {
    client_id: String,
    username: String,
    password: String,
    connect_flags: ConnectFlags,
    last_will_message: String,
    last_will_topic: String,
    // keep alive?
}

impl ConnectBuilder {
    pub fn new() -> ConnectBuilder{
        let empty_flags = ConnectFlags::new(false,false,false,false,false,false);

        ConnectBuilder {
            client_id: "".to_string(),
            username: "".to_string(),
            password: "".to_string(),
            connect_flags: empty_flags,
            last_will_message: "".to_string(),
            last_will_topic: "".to_string()
        }
    }

}*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_protocol_level_byte(){
        let byte: [u8; 1] = [0x4];
        let to_test = verify_protocol_level_byte(&byte);
        assert_eq!(to_test, Ok(()));
    }

    #[test]
    fn error_protocol_level_byte(){
        let byte: [u8; 1] = [0x5];
        let to_test = verify_protocol_level_byte(&byte);

        assert_eq!(to_test, Err("Protocol level byte inválido".to_owned()));
    }

    #[test]
    fn correct_mqtt_string_byte(){
        let bytes: [u8; 6] = [0x00, 0x04, 0x4D, 0x51, 0x54, 0x54];
        let to_test = verify_mqtt_string_bytes(&bytes);
        assert_eq!(to_test, Ok(()));
    }

    #[test]
    fn error_mqtt_string_byte(){
        let bytes: [u8; 6] = [0x00, 0x05, 0x4D, 0x51, 0x54, 0x54];
        let to_test = verify_mqtt_string_bytes(&bytes);
        assert_eq!(to_test, Err("No es MQTT".to_owned()));
    }
}
