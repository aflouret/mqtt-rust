use std::io::Read;
use crate::all_packets::connect::{Connect};
use crate::packet::{Packet, ReadPacket};

pub fn build_packet(indetifier_byte: u8, stream: &mut dyn Read) -> Result<Packet, String> {
    match indetifier_byte {
        0x10 => {  
            println!("Es un Connect!");
            Connect::read_from(stream) }
        // 0x20 => { ConnackBuilder... } etc etc

        _ => {Err("Ningún packet tiene ese código".to_owned())}
    }
    
}

// Cambié de "read_from_server" a "read_packet" porque la vamos a usar tamb desde el cliente
pub fn read_packet(stream: &mut dyn Read) -> Result<Packet,Box<dyn std::error::Error>> {
    let mut indetifier_byte = [0u8; 1];
    stream.read_exact(&mut indetifier_byte)?;
    let packet = build_packet(u8::from_be_bytes(indetifier_byte), stream)?;
    
    Ok(packet)
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
