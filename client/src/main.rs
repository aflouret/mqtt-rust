use crate::client::Client;
use common::all_packets::connect::Connect;
use common::all_packets::connect::ConnectPayload;
mod client;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::new("Pepito".to_owned(), "127.0.0.1:8080".to_owned())?;

    let connect_packet = Connect::new(
        ConnectPayload::new(
            "u".to_owned(),
            Some("u".to_owned()),
            Some("u".to_owned()),
            Some("u".to_owned()),
            Some("u".to_owned()),
        ),
        60,
        true,
        true,
        true,
    );

    client.client_run(connect_packet)?;

    Ok(())
}

/*fn client_run(address: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut socket = TcpStream::connect(address)?;

    let connect_packet = Connect::new(
        ConnectPayload::new(
            "u".to_owned(),
            Some("u".to_owned()),
            Some("u".to_owned()),
            Some("u".to_owned()),
            Some("u".to_owned()),
        ),
        ConnectFlags::new(false, false, false, false, false, false),
        60,
    );

    connect_packet.write_to(&mut socket)?;
    println!("Se envió el connect packet");

    let received_packet = parser::read_packet(&mut socket)?;
    if let Packet::Connack(connack_packet) = received_packet {
        println!(
            "Se recibió el connack packet. Session present: {}. Connect return code: {}",
            connack_packet.session_present, connack_packet.connect_return_code
        );
    }

    Ok(())
}
*/
