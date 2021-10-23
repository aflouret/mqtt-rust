use std::io::Read;
use crate::all_packets::connect::{Connect};
use crate::packet::{Packet, ReadPacket};

pub fn build_packet(byte: u8, stream: &mut dyn Read) -> Result<Packet, String> {
    match byte {
        0x10 => {  Connect::read_from(stream) }
        // 0x20 => { ConnackBuilder... } etc etc

        _ => {Err("Ningún packet tiene ese código".to_owned())}
    }
    
}

pub fn read_from_server(stream: &mut dyn Read) -> Result<Packet,String> {
    let mut indetifier_byte = [0u8; 1];
    stream.read_exact(&mut indetifier_byte);
    let packet = build_packet(indetifier_byte[0],stream);
    //connect
    //t p = packet.build_packet(packet);
    packet
}

// Algoritmo para desencodear el número que representa el RemainingLength 
// en el fixed header de cualquier packet
pub fn get_remaining_length(encoded_bytes: Vec<u8>) -> Result<u32,()> {
    let mut multiplier:u32 = 1;    
    let mut value:u32 = 0;
    for encoded_byte in encoded_bytes {
        value += (encoded_byte & 127) as u32 * multiplier;
        multiplier *= 128;
        if multiplier > 128*128*128 {
           return Err(()); }
        if (encoded_byte & 128) == 0 {break};
    }
    Ok(value)
}


/*#[test]
fn identify_connect_package(){
    assert_eq!(ConnectBuilder::new(), identify_package(0x10));
}*/
