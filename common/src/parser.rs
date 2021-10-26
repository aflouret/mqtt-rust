use crate::all_packets::connect::Connect;
use crate::all_packets::connack::Connack;
use crate::packet::{Packet, ReadPacket};
use std::io::{Read};

// Devuelve el packet correspondiente a lo que leyó del stream.
pub fn read_packet(stream: &mut dyn Read) -> Result<Packet, Box<dyn std::error::Error>> {
    let mut indetifier_byte = [0u8; 1];
    stream.read_exact(&mut indetifier_byte)?;

    match indetifier_byte[0] {
        0x10 => Ok(Connect::read_from(stream)?),
        0x20 => { Ok(Connack::read_from(stream)?) }
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

// Módulo de unit-testeo. Todo lo que está acá compila y corre sólo al hacer cargo test.
// https://doc.rust-lang.org/book/ch11-03-test-organization.html
#[cfg(test)]
mod tests {
    //use std::net::{TcpListener, TcpStream};
    //use std::io::Write;
    use std::io::Cursor;
    use super::*;
    
    // Importante: al llamar a get_stream_to_test_decode, usar un puerto distinto para
    // cada test, ya que cargo corre los tests en paralelo y causa problemas usar el 
    // mismo puerto. 
    /*
    #[test]
    fn decode_remaining_length_1_byte_max_min(){
        let length_to_test_min = 0;
        let mut stream = get_stream_to_test_decode(length_to_test_min, "8080");
        let length_decoded = decode_remaining_length(&mut stream).unwrap();
        assert_eq!(length_decoded, length_to_test_min);

        let length_to_test_max = 127;
        let mut stream = get_stream_to_test_decode(length_to_test_max, "8080");
        let length_decoded = decode_remaining_length(&mut stream).unwrap();
        assert_eq!(length_decoded, length_to_test_max);
    }

    #[test]
    fn decode_remaining_length_2_bytes_max_min(){
        let length_to_test_min = 128;
        let mut stream = get_stream_to_test_decode(length_to_test_min, "8081");
        let length_decoded = decode_remaining_length(&mut stream).unwrap();
        assert_eq!(length_decoded, length_to_test_min);

        let length_to_test_max = 16383;
        let mut stream = get_stream_to_test_decode(length_to_test_max, "8081");
        let length_decoded = decode_remaining_length(&mut stream).unwrap();
        assert_eq!(length_decoded, length_to_test_max);
    }
    */

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

    // Función auxiliar para el testeo, que crea un servidor y cliente en dos threads distintos
    // y devuelve el socket del servidor para que desde el test se lea lo que mandó el cliente: 
    // un remaining length encodeado.
    /*
    fn get_stream_to_test_decode(length_to_test: u32, port: &str) -> TcpStream {
        let port = port.to_owned();
        let listener = TcpListener::bind("0.0.0.0:".to_owned() + &port).unwrap();
        let join_handle_client = std::thread::spawn(move || {
            let mut socket = TcpStream::connect("127.0.0.1:".to_owned() + &port).unwrap();
            let length_encoded = encode_remaining_length(length_to_test);
            for byte in length_encoded {
                socket.write(&[byte]).unwrap();
            }
        });
        join_handle_client.join().unwrap();
        let client_stream = listener.accept().unwrap().0;
        client_stream
    }
    */

    #[test]
    fn decode_length_1_byte_min(){
        let mut buff = Cursor::new(vec![1]);
        let to_test = decode_remaining_length(&mut buff).unwrap();
        assert_eq!(to_test, 1);
    }

    #[test]
    fn decode_length_1_byte_max(){
        let mut buff = Cursor::new(vec![127]);
        let to_test = decode_remaining_length(&mut buff).unwrap();
        assert_eq!(to_test, 127);
    }

    #[test]
    fn decode_length_2_byte_min(){
        let mut buff = Cursor::new(vec![128, 1]);
        let to_test = decode_remaining_length(&mut buff).unwrap();
        assert_eq!(to_test, 128);
    }

    #[test]
    fn decode_length_2_byte_max(){
        let mut buff = Cursor::new(vec![255, 127]);
        let to_test = decode_remaining_length(&mut buff).unwrap();
        assert_eq!(to_test, 16383);
    }

    #[test]
    fn decode_length_3_byte_min(){
        let mut buff = Cursor::new(vec![128, 128, 1]);
        let to_test = decode_remaining_length(&mut buff).unwrap();
        assert_eq!(to_test, 16384);
    }

    #[test]
    fn decode_length_3_byte_max(){
        let mut buff = Cursor::new(vec![255, 255, 127]);
        let to_test = decode_remaining_length(&mut buff).unwrap();
        assert_eq!(to_test, 2097151);
    }

    #[test]
    fn decode_length_4_byte_min(){
        let mut buff = Cursor::new(vec![128, 128, 128, 1]);
        let to_test = decode_remaining_length(&mut buff).unwrap();
        assert_eq!(to_test, 2097152);
    }

    #[test]
    fn decode_length_4_byte_max(){
        let mut buff = Cursor::new(vec![255, 255, 255, 127]);
        let to_test = decode_remaining_length(&mut buff).unwrap();
        assert_eq!(to_test, 268435455);
    }

    #[test]
    fn error_decode_length(){
        let mut buff = Cursor::new(vec![255, 255, 255, 255, 127]);
        let to_test = decode_remaining_length(&mut buff);

        assert_eq!(to_test.is_err(), true);
    }
}