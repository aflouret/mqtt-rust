use std::{io::{Read, ErrorKind}};

const MAX_MQTT_STRING_BYTES: u16 = 65535;

// Algoritmo para decodificar el número que representa el Remaining Length
// en el fixed header de cualquier packet
pub fn decode_remaining_length(stream: &mut dyn Read) -> Result<u32, Box<dyn std::error::Error>> {
    let mut multiplier: u32 = 1;
    let mut value: u32 = 0;
    for encoded_byte in stream.bytes() {
        let encoded_byte: u8 = encoded_byte?;
        value += (encoded_byte & 0x7F) as u32 * multiplier;
        if (encoded_byte & 0x80) == 0 {
            break;
        };
        multiplier *= 0x80;
        if multiplier > 0x80 * 0x80 * 0x80 {
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
        encoded_byte = x % 0x80;
        x /= 0x80;
        if x > 0 {
            encoded_byte |= 0x80;
        }
        vec.push(encoded_byte as u8);
        if x == 0 {
            break;
        }
    }
    vec
}

pub fn encode_mqtt_string(string: &str) -> Result<Vec<u8>, String> {
    let mut vec: Vec<u8> = Vec::new();

    let string_bytes = string.as_bytes();
    let len_string_bytes = string_bytes.len() as u16;

    if len_string_bytes > MAX_MQTT_STRING_BYTES {
        return Err("Incorrect length".into());
    }

    let length = len_string_bytes.to_be_bytes();
    vec.push(length[0]);
    vec.push(length[1]);
    for byte in string_bytes {
        vec.push(*byte);
    }

    Ok(vec)
}

//Result<String, Box<dyn std::error::Error>>
pub fn decode_mqtt_string(stream: &mut dyn Read) -> Result<String, std::io::Error> {
    let mut bytes = [0u8; 2];
    stream.read_exact(&mut bytes)?;
    let number = u16::from_be_bytes(bytes);
    let mut bytes_2 = vec![0; number as usize];
    stream.read_exact(&mut bytes_2)?;
    let payload_ = String::from_utf8(bytes_2);
    if let Ok(payload) = payload_ {
        return Ok(payload);
    } else {
        return Err(std::io::Error::new(ErrorKind::Other, "La cadena no es UTF-8"));
    }
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

    #[test]
    fn encode_mqtt_string_len_1_byte() {
        let string = String::from("MQTT");
        let to_test = encode_mqtt_string(&string).unwrap();

        assert_eq!(to_test, vec![0x00, 0x04, 0x4D, 0x51, 0x54, 0x54]);
    }

    #[test]
    fn encode_mqtt_string_len_0() {
        let string = String::from("");
        let to_test = encode_mqtt_string(&string).unwrap();

        assert_eq!(to_test, vec![0, 0]);
    }

    #[test]
    fn decode_mqtt_string_len_1_byte() {
        let mut buff = Cursor::new(vec![0, 4, 116, 101, 115, 116]);
        let to_test = decode_mqtt_string(&mut buff).unwrap();

        assert_eq!(to_test, String::from("test"));
    }

    #[test]
    fn decode_mqtt_string_len_0() {
        let mut buff = Cursor::new(vec![0, 0]);
        let to_test = decode_mqtt_string(&mut buff).unwrap();

        assert_eq!(to_test, String::from(""));
    }

    #[test]
    fn encode_and_decode_mqtt_string() {
        let string = String::from("mqtt");
        let encode = encode_mqtt_string(&string).unwrap();
        let mut buff = Cursor::new(encode);
        let to_test = decode_mqtt_string(&mut buff).unwrap();

        assert_eq!(to_test, String::from("mqtt"));
    }
}
