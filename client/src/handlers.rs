use common::all_packets::connect::{Connect, ConnectPayload};
use common::all_packets::disconnect::Disconnect;
use common::all_packets::publish::Publish;
use common::all_packets::subscribe::Subscribe;
use common::all_packets::unsubscribe::Unsubscribe;

pub enum EventHandlers {
    HandleConection(HandleConection),
    HandlePublish(HandlePublish),
    HandleSubscribe(HandleSubscribe),
    HandleUnsubscribe(HandleUnsubscribe),
    HandleDisconnect(HandleDisconnect),
}

#[derive(Debug)]
pub struct HandleConection {
    pub address: String,
    pub client_id: String,
    pub clean_session: bool,
    pub last_will_retain: bool,
    pub last_will_qos: bool,
    pub keep_alive_second: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub last_will_msg: Option<String>,
    pub last_will_topic: Option<String>,
}

impl HandleConection {
    pub fn new(address: String,
               client_id: String,
               clean_session: bool,
               last_will_retain: bool,
               last_will_qos: bool,
               keep_alive_second: String,
               username: Option<String>,
               password: Option<String>,
               last_will_msg: Option<String>,
               last_will_topic: Option<String>,
    ) -> Self {
        Self { address: address, client_id: client_id, clean_session: clean_session,
            last_will_retain: last_will_retain, last_will_qos: last_will_qos, keep_alive_second: keep_alive_second,
            username: username, password: password, last_will_msg: last_will_msg,
            last_will_topic: last_will_topic }
    }

    pub fn get_address(&mut self) -> String {
        self.address.to_string()
    }
}

#[derive(Debug)]
pub struct HandlePublish {
    pub publish_packet: Publish,
    pub topic: String,
    pub app_msg: String,
    pub qos_level: bool,
    pub retain: bool,
}

impl HandlePublish {
    pub fn new(publish_packet: Publish) -> Self {
        Self { publish_packet: publish_packet, topic: "".to_string(), app_msg: "".to_string(), qos_level: false, retain: false }
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
    pub fn new(unsubscribe_packet: Unsubscribe) -> Self {
        Self { unsubscribe_packet: unsubscribe_packet }
    }
}

#[derive(Debug)]
pub struct HandleDisconnect {
    pub disconnect_packet: Disconnect,
}

impl HandleDisconnect {
    pub fn new(disconnect_packet: Disconnect) -> Self {
        Self { disconnect_packet: disconnect_packet }
    }
}