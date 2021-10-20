fn indentify_package(bytes: &Vec<u8>) {
    let fixed_header = &bytes[0];
    match fixed_header {
        0x10 => { parse_connect_package(&bytes[1..]) }
    } 
}

fn parse_connect_package(bytes: &Vec<u8>) -> Result<Packet,()>{
    let remaining_length = get_remaining_lenth(&bytes[0..4])?;
    verify_mqtt_bytes(&bytes[3..7])?
}

fn get_remaining_length(encoded_bytes: Vec<u8>) -> Result<u32,()> {
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

fn verify_mqtt_bytes(bytes: &Vec<u8>) -> Result<(),String>{
    if bytes[0] != 0x6d || bytes[1] != 0x71 || bytes[2] != 0x76 || bytes[3] != 0x76 {
        Err("No es MQTT")
    }
    Ok(())
}
