use common::all_packets::disconnect::Disconnect;
use common::all_packets::unsubscribe::Unsubscribe;
use common::packet::Qos;

pub enum EventHandlers {
    HandleConection(HandleConection),
    HandlePublish(HandlePublish),
    HandleSubscribe(HandleSubscribe),
    HandleUnsubscribe(HandleUnsubscribe),
    HandleDisconnect(HandleDisconnect),
    HandleInternPacketId(HandleInternPacketId)
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
            last_will_retain: last_will_retain, last_will_qos: last_will_qos,
            keep_alive_second: keep_alive_second,
            username: username, password: password,
            last_will_msg: last_will_msg,
            last_will_topic: last_will_topic }
    }

    pub fn get_address(&mut self) -> String {
        self.address.to_string()
    }
}

#[derive(Debug)]
pub struct HandlePublish {
    pub topic: String,
    pub app_msg: String,
    pub qos0_level: bool,
    pub qos1_level: bool,
    pub retain: bool,
}

impl HandlePublish {
    pub fn new(topic: String, app_msg: String, qos0_level: bool, qos1_level:bool, retain: bool) -> Self {
        Self {topic: topic, app_msg: app_msg, qos0_level: qos0_level, qos1_level: qos1_level, retain: retain }
    }
}


#[derive(Debug)]
pub struct HandleSubscribe {
    pub topic: String,
    pub qos: Qos,
}

impl HandleSubscribe {
    pub fn new(topic: String, qos0_is_active: bool) -> Self {
        let qos = match qos0_is_active {
            true => Qos::AtMostOnce,
            false => Qos::AtLeastOnce,
        };

        Self {topic: topic, qos}
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
        println!("disconnect");
        Self { disconnect_packet: disconnect_packet }
    }
}

#[derive(Debug, Clone)]
pub struct HandleInternPacketId {
    pub packet_id: u16,
}

impl HandleInternPacketId {
    pub fn new(packet_id: u16) -> Self {
        Self { packet_id: packet_id }
    }
}