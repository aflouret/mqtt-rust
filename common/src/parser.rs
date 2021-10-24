use std::io::{BufReader, Read, stdin};
use crate::all_packets::connect::{Connect};
use crate::packet::{Packet, ReadPacket};
use std::str::from_utf8;

// Devuelve el packet correspondiente a lo que leyó del stream.
pub fn read_packet(stream: &mut dyn Read) -> Result<Packet, Box<dyn std::error::Error>> {
    let mut indetifier_byte = [0u8; 1];
    stream.read_exact(&mut indetifier_byte)?;

    match indetifier_byte[0] {
        0x10 => { Ok(Connect::read_from(stream)?) }
        // 0x20 => { Ok(Connack::read_from(stream)?) }
        // 0x3_ => { Ok(Publish::read_from(stream)?) }
        _ => {Err("Ningún packet tiene ese código".into())}
    }
}

// Algoritmo para desencodear el número que representa el RemainingLength 
// en el fixed header de cualquier packet
pub fn get_remaining_length(stream: &mut dyn Read) -> Result<u32,Box<dyn std::error::Error>> {
    let mut multiplier:u32 = 1;    
    let mut value:u32 = 0;
    for encoded_byte in stream.bytes() {
        let encoded_byte: u8 = from_utf8(&[encoded_byte?])?.parse()?;
        value += (encoded_byte & 127) as u32 * multiplier;
        multiplier *= 128;
        if multiplier > 128*128*128 {
           return Err("Incorrect length".into()); }
        if (encoded_byte & 128) == 0 {break};
    }
    Ok(value)
}


#[test]
// fn identify_connect_package(){
//     assert_eq!(ConnectBuilder::new(), identify_package(0x10));
// }
fn decode_remaining_length(){
    println!("testeo {:?}", "5".as_bytes());
    let to_test = get_remaining_length(&mut stdin()).unwrap();
    assert_eq!(to_test, 5);
}// para que esto sirva hay que usar from_utf8(&[encoded_byte])?, para decodificar el as_bytes