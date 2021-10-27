use crate::all_packets::connack::Connack;
use crate::all_packets::connect::Connect;
use crate::packet::{Packet, ReadPacket};
use std::io::Read;

// Devuelve el packet correspondiente a lo que leyó del stream.
pub fn read_packet(stream: &mut dyn Read) -> Result<Packet, Box<dyn std::error::Error>> {
    let mut indetifier_byte = [0u8; 1];
    stream.read_exact(&mut indetifier_byte)?;

    match indetifier_byte[0] {
        0x10 => Ok(Connect::read_from(stream)?),
        0x20 => Ok(Connack::read_from(stream)?),
        // 0x3_ => { Ok(Publish::read_from(stream)?) }
        _ => Err("Ningún packet tiene ese código".into()),
    }
}

// Algoritmo para decodificar el número que representa el Remaining Length
// en el fixed header de cualquier packet
pub fn decode_remaining_length(stream: &mut dyn Read) -> Result<u32, Box<dyn std::error::Error>> {
    let mut multiplier: u32 = 1;
    let mut value: u32 = 0;
    for encoded_byte in stream.bytes() {
        println!("encoded byte: {:?}", encoded_byte);
        let encoded_byte: u8 = encoded_byte?;
        value += (encoded_byte & 127) as u32 * multiplier;
        if (encoded_byte & 128) == 0 {
            break;
        };
        multiplier *= 128;
        if multiplier > 128 * 128 * 128 {
            return Err("Incorrect length".into());
        }
    }
    Ok(value)
}

// Algoritmo para codificar el Remaining Length. Devuelve un vector que puede ser de 1, 2, 3 ó 4
// elementos u8
pub fn encode_remaining_length(packet_length: u32) -> Vec<u8> {
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

/* ----------------------------- Unit tests -----------------------------*/
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn encode_length_1_byte_min() {
        let to_test = encode_remaining_length(1);
        assert_eq!(to_test, [1]);
    }

    #[test]
    fn encode_length_1_byte_max() {
        let to_test = encode_remaining_length(127);
        assert_eq!(to_test, [127]);
    }

    #[test]
    fn encode_length_2_bytes_min() {
        let to_test = encode_remaining_length(128);
        assert_eq!(to_test, [128, 1]);
    }

    #[test]
    fn encode_length_2_bytes_max() {
        let to_test = encode_remaining_length(16383);
        assert_eq!(to_test, [255, 127]);
    }

    #[test]
    fn encode_length_3_bytes_min() {
        let to_test = encode_remaining_length(16384);
        assert_eq!(to_test, [128, 128, 1]);
    }

    #[test]
    fn encode_length_3_bytes_max() {
        let to_test = encode_remaining_length(2097151);
        assert_eq!(to_test, [255, 255, 127]);
    }

    #[test]
    fn encode_length_4_bytes_min() {
        let to_test = encode_remaining_length(2097152);
        assert_eq!(to_test, [128, 128, 128, 1]);
    }

    #[test]
    fn encode_length_4_bytes_max() {
        let to_test = encode_remaining_length(268435455);
        assert_eq!(to_test, [255, 255, 255, 127]);
    }

    #[test]
    fn decode_length_1_byte_min() {
        let mut buff = Cursor::new(vec![1]);
        let to_test = decode_remaining_length(&mut buff).unwrap();
        assert_eq!(to_test, 1);
    }

    #[test]
    fn decode_length_1_byte_max() {
        let mut buff = Cursor::new(vec![127]);
        let to_test = decode_remaining_length(&mut buff).unwrap();
        assert_eq!(to_test, 127);
    }

    #[test]
    fn decode_length_2_byte_min() {
        let mut buff = Cursor::new(vec![128, 1]);
        let to_test = decode_remaining_length(&mut buff).unwrap();
        assert_eq!(to_test, 128);
    }

    #[test]
    fn decode_length_2_byte_max() {
        let mut buff = Cursor::new(vec![255, 127]);
        let to_test = decode_remaining_length(&mut buff).unwrap();
        assert_eq!(to_test, 16383);
    }

    #[test]
    fn decode_length_3_byte_min() {
        let mut buff = Cursor::new(vec![128, 128, 1]);
        let to_test = decode_remaining_length(&mut buff).unwrap();
        assert_eq!(to_test, 16384);
    }

    #[test]
    fn decode_length_3_byte_max() {
        let mut buff = Cursor::new(vec![255, 255, 127]);
        let to_test = decode_remaining_length(&mut buff).unwrap();
        assert_eq!(to_test, 2097151);
    }

    #[test]
    fn decode_length_4_byte_min() {
        let mut buff = Cursor::new(vec![128, 128, 128, 1]);
        let to_test = decode_remaining_length(&mut buff).unwrap();
        assert_eq!(to_test, 2097152);
    }

    #[test]
    fn decode_length_4_byte_max() {
        let mut buff = Cursor::new(vec![255, 255, 255, 127]);
        let to_test = decode_remaining_length(&mut buff).unwrap();
        assert_eq!(to_test, 268435455);
    }

    #[test]
    fn error_decode_length() {
        let mut buff = Cursor::new(vec![255, 255, 255, 255, 127]);
        let to_test = decode_remaining_length(&mut buff);

        assert_eq!(to_test.is_err(), true);
    }
}
