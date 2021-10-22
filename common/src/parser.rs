use crate::all_packets::connect::ConnectBuilder;

pub fn indentify_package(byte: u8) -> Result<ConnectBuilder, String> {
    match byte {
        0x10 => { Ok(ConnectBuilder::new()) }
        // 0x20 => { ConnackBuilder... } etc etc

        _ => {Err("Ningún packet tiene ese código".to_owned())}
    }
    
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


