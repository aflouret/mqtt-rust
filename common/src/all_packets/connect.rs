use crate::packet_flags::ConnectFlags;
use std::io::Read;
use crate::packet::{Packet, ReadPacket};
use crate::parser::get_remaining_length;

pub struct Connect {
    client_id: String,
    username: String,
    password: String,
    connect_flags: ConnectFlags,
    last_will_message: String,
    last_will_topic: String,
    // keep alive?
}

impl Connect {
    pub fn new(client_id: String,
        username: String,
        password: String,
        connect_flags: ConnectFlags,
        last_will_message: String,
        last_will_topic: String) -> Connect {
            Connect {
                client_id, username, password, connect_flags, last_will_message, last_will_topic
            }
        }

/*    fn set_flags(&self, flags_byte: u8) -> Result<(),String>{
        // TODO --> hacer todas las verificaciones
        if (flags_byte & 0x1) == 0x1 { return Err("El Reserved bit está en 1".to_string())}
        let clean_session = (flags_byte & 0x2) == 0x2;
        self.connect_flags.set_clean_session(clean_session);

        let last_will_flag = (flags_byte & 0x4) == 0x4;
        self.connect_flags.set_last_will_flag(last_will_flag);

        let last_will_qos = (flags_byte & 0x8) == 0x8;
        self.connect_flags.set_last_will_qos(last_will_qos);

        let last_will_retain = (flags_byte & 0x20) == 0x20;
        self.connect_flags.set_last_will_retain(last_will_qos);

        let with_password = (flags_byte & 0x40) == 0x40;
        self.connect_flags.set_password(with_password);

        let with_username = (flags_byte & 0x80) == 0x80;
        self.connect_flags.set_username(with_username);
    }*/

/*    pub fn read_from(&self, stream: &mut dyn Read) -> Result<(),String>{
        let mut length_bytes = [0u8; 1]; // TODO!!!!! Hacer que pueda leer hasta 4 bytes, como dice la documentacion
        stream.read_exact(&mut length_bytes)?;
        let remaining_length = get_remaining_length(length_bytes)?;

        let mut mqtt_bytes = [0u8; 4];
        stream.read_exact(&mut mqtt_bytes)?;
        verify_mqtt_bytes(&mqtt_bytes)?;

        let mut mqtt_bytes = [0u8; 1];
        stream.read_exact(&mut mqtt_bytes)?;
        verify_protocol_level_byte()?;

        let mut flags_byte = [0u8; 1];
        stream.read_exact(&mut flags_byte)?;
        self.set_flags(flags_byte)?;
        //Ok()
    }*/


}

impl ReadPacket for Connect {
    fn read_from(stream: &mut dyn Read) -> Result<Packet, String> {
        println!("Entro a connect");
        Ok(Packet::Connect(Connect::new("123".to_string(),
                     "asd".to_string(),
                     "awd".to_string(),
                     ConnectFlags::new(false, false, false, false, false, false),
                     "last".to_string(), "sdt".to_string())))
        /*        let mut length_bytes = [0u8; 1]; // TODO!!!!! Hacer que pueda leer hasta 4 bytes, como dice la documentacion
        stream.read_exact(&mut length_bytes)?;
        let remaining_length = get_remaining_length(length_bytes)?;

        let mut mqtt_bytes = [0u8; 4];
        stream.read_exact(&mut mqtt_bytes)?;
        verify_mqtt_bytes(&mqtt_bytes)?;

        let mut mqtt_bytes = [0u8; 1];
        stream.read_exact(&mut mqtt_bytes)?;
        verify_protocol_level_byte()?;

        let mut flags_byte = [0u8; 1];
        stream.read_exact(&mut flags_byte)?;
        self.set_flags(flags_byte)?;*/
        //Ok()
    }
}

fn verify_mqtt_bytes(bytes: &[u8; 4]) -> Result<(),String>{
    if bytes[0] != 0x4d || bytes[1] != 0x51 || bytes[2] != 0x54 || bytes[3] != 0x54 {
        return Err("No es MQTT".to_string());
    }
    Ok(())
}

fn verify_protocol_level_byte(byte: &[u8; 1]) -> Result<(),String>{
    if byte[0] != 0x4 {
       return Err("Protocol level byte inválido".to_string());
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

    pub fn build_packet(&self, stream: &mut dyn Read) -> Result<(),String>{
        let mut length_bytes = [0u8; 1]; // TODO!!!!! Hacer que pueda leer hasta 4 bytes, como dice la documentacion
        stream.read_exact(&mut length_bytes)?;
        let remaining_length = get_remaining_length(length_bytes)?;

        let mut mqtt_bytes = [0u8; 4];
        stream.read_exact(&mut mqtt_bytes)?;
        verify_mqtt_bytes(&mqtt_bytes)?;

        let mut mqtt_bytes = [0u8; 1];
        stream.read_exact(&mut mqtt_bytes)?;
        verify_protocol_level_byte()?;

        let mut flags_byte = [0u8; 1];
        stream.read_exact(&mut flags_byte)?;
        self.set_flags(flags_byte)?;
        //Ok()
    }
    
    fn set_flags(&self, flags_byte: u8) -> Result<(),String>{
        // TODO --> hacer todas las verificaciones
        if (flags_byte & 0x1) == 0x1 { Err("El Reserved bit está en 1")}
        let clean_session = (flags_byte & 0x2) == 0x2;
        self.connect_flags.set_clean_session(clean_session);

        let last_will_flag = (flags_byte & 0x4) == 0x4;
        self.connect_flags.last_will_flag(last_will_flag);

        let last_will_qos = (flags_byte & 0x8) == 0x8;
        self.connect_flags.set_last_will_qos(last_will_qos);

        let last_will_retain = (flags_byte & 0x20) == 0x20;
        self.connect_flags.set_last_will_retain(last_will_qos);

        let with_password = (flags_byte & 0x40) == 0x40;
        self.connect_flags.set_password(with_password);

        let with_username = (flags_byte & 0x80) == 0x80;
        self.connect_flags.set_username(with_username);
    }
}*/
