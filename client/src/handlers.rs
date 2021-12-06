
use common::all_packets::connect::{Connect, ConnectPayload};
use common::all_packets::publish::Publish;
use common::all_packets::subscribe::Subscribe;
use common::all_packets::unsubscribe::Unsubscribe;

pub enum EventHandlers {
    HandleConection(HandleConection),
    HandlePublish(HandlePublish),
    HandleSubscribe(HandleSubscribe),
    HandleUnsubscribe(HandleUnsubscribe),
}

#[derive(Debug)]
pub struct HandleConection {
    pub connect_packet: Connect,
    pub address: String,
}

impl HandleConection {
    pub fn new(connect_packet: Connect, address: String)  -> Self {
        Self{connect_packet: connect_packet, address: address}
    }

    pub fn get_address(&mut self) -> String {
        self.address.to_string()
    }
}

#[derive(Debug)]
pub struct HandlePublish {
    pub publish_packet: Publish,
}

impl HandlePublish {
    pub fn new(publish_packet: Publish)  -> Self {
        Self{publish_packet: publish_packet}
    }

}


#[derive(Debug)]
pub struct HandleSubscribe {
    pub subscribe_packet: Subscribe,
}

impl HandleSubscribe {
    pub fn new(subscribe_packet: Subscribe) -> Self {
        Self { subscribe_packet: subscribe_packet }
    }
}



#[derive(Debug)]
pub struct HandleUnsubscribe {
    pub unsubscribe_packet: Unsubscribe,
}

impl HandleUnsubscribe {
    pub fn new(unsubscribe_packet: Unsubscribe)  -> Self {
        Self{unsubscribe_packet: unsubscribe_packet}
    }

}