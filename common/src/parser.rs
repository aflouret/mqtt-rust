mod connect;

pub fn indentify_package(byte: u8) -> Result<> {
    match byte {
        0x10 => { ConnectBuilder::new() }
        // 0x20 => { ConnackBuilder... }
        _ => {Err()}
    } 
}

pub fn get_remaining_length(encoded_bytes: Vec<u8>) -> Result<u32,()> {
    let mut multiplier = 1;    
    let mut value = 0;
    for encoded_byte in encoded_bytes {
        value += (encoded_byte & 127) * multiplier;
        multiplier *= 128;
        if multiplier > 128*128*128 {
           Err() }
        if (encoded_byte & 128) == 0 {break};
    }
}


