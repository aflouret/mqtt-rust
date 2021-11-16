use std::sync::mpsc::Sender;
use common::all_packets::connect::{Connect, ConnectPayload};
use common::all_packets::publish::{Publish, PublishFlags};
use common::packet::Packet;

//Se encarga de ser el nexo entre la interfaz gráfica(o cualquier otra cosa) y el client
//Se encarga de enviar paquetes(creados o no) al client
pub struct ClientController {
    //ch_id: String,
    //ch_type: TypeOfInterface -- Puede ser que el ch sea un CLI o una interfaz grafica, o web.
    sender_to_client: Sender<Packet>
}


impl ClientController {
    pub fn new(channel: Sender<Packet>) -> Result<ClientController, Box<dyn std::error::Error>> {
        Ok(Self{sender_to_client: channel})
    }


    pub fn send_packet(self, packet: Packet) {
        self.sender_to_client.send(packet);
    }

    pub fn build_connect_packet(self, id: &str) { //Devuelve un result por si
        //va ir tomando los valores que ponga el usuario en los labels.
        let connect_packet = Connect::new(
            ConnectPayload::new(
                id.to_string(),
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
        self.send_packet(Packet::Connect(connect_packet));
    }

    pub fn build_publish_(self) { //Devuelve un result por si
        //va ir tomando los valores que ponga el usuario en los labels.
        let publish_packet = Publish::new(
            PublishFlags::new(0b0100_1011),
            "Topic".to_string(),
            Some(15),
            "Message".to_string(),
        );
        self.send_packet(Packet::Publish(publish_packet));
    }
}