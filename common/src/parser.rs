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
    let mut multiplier: u32 = 1;    
    let mut value: u32 = 0;
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

fn encode_remaining_length(packet_length: u32) -> Vec<u8> {
    let mut vec: Vec<u8> = Vec::new();
    let mut encoded_byte;
    let mut x = packet_length;
    loop {
        encoded_byte = x % 128;
        x = x / 128;
        if x > 0 {
            encoded_byte = encoded_byte | 128;
        }
        vec.push(encoded_byte as u8);
        if x <= 0 {
            break;
        }
    }

    vec
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

#[test]
fn encode_length_1_byte_min(){
    let to_test = encode_remaining_length(1);
    assert_eq!(to_test, [1]);
}

#[test]
fn encode_length_1_byte_max(){
    let to_test = encode_remaining_length(127);
    assert_eq!(to_test, [127]);
}

#[test]
fn encode_length_2_bytes_min(){
    let to_test = encode_remaining_length(128);
    assert_eq!(to_test, [128, 1]);
}

#[test]
fn encode_length_2_bytes_max(){
    let to_test = encode_remaining_length(16383);
    assert_eq!(to_test, [255, 127]);
}

#[test]
fn encode_length_3_bytes_min(){
    let to_test = encode_remaining_length(16384);
    assert_eq!(to_test, [128, 128, 1]);
}

#[test]
fn encode_length_3_bytes_max(){
    let to_test = encode_remaining_length(2097151);
    assert_eq!(to_test, [255, 255, 127]);
}

#[test]
fn encode_length_4_bytes_min(){
    let to_test = encode_remaining_length(2097152);
    assert_eq!(to_test, [128, 128, 128, 1]);
}

#[test]
fn encode_length_4_bytes_max(){
    let to_test = encode_remaining_length(268435455);
    assert_eq!(to_test, [255, 255, 255, 127]);
}