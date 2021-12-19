use common::all_packets::disconnect::Disconnect;
use common::all_packets::puback::Puback;
use common::all_packets::publish::Publish;
use common::all_packets::unsubscribe::Unsubscribe;
use common::packet::Qos;

pub enum EventHandlers {
    Conection(HandleConection),
    Publish(HandlePublish),
    Subscribe(HandleSubscribe),
    Unsubscribe(HandleUnsubscribe),
    Disconnect(HandleDisconnect),
    InternPuback(HandleInternPuback),
    InternPublish(HandleInternPublish),
    InternPacketId(HandleInternPacketId),
}

#[derive(Debug)]
pub struct HandleConection {
    pub address: String,
    pub client_id: String,
    pub clean_session: bool,
    pub keep_alive_second: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub last_will: LastWillInfo,
}

impl HandleConection {
    pub fn new(address: String,
        client_id: String,
        clean_session: bool,
        keep_alive_second: String,
        username: Option<String>,
        password: Option<String>,
        last_will: LastWillInfo
) -> Self {
 Self { address, client_id, clean_session,
     keep_alive_second,
     username, password,
     last_will }
}

    pub fn get_address(&mut self) -> String {
        self.address.to_string()
    }
}

#[derive(Debug)]
pub struct LastWillInfo{
    pub last_will_topic: Option<String>,
    pub last_will_msg: Option<String>,
    pub last_will_qos: bool,
    pub last_will_retain: bool,
}

impl LastWillInfo {
    pub fn new(last_will_topic: Option<String>, last_will_msg: Option<String>, last_will_qos: bool, last_will_retain: bool) -> LastWillInfo {
        LastWillInfo {
            last_will_topic,
            last_will_msg,
            last_will_qos,
            last_will_retain
        }
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
        Self {topic, app_msg, qos0_level, qos1_level, retain }
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

        Self {topic, qos}
    }
}


#[derive(Debug)]
pub struct HandleUnsubscribe {
    pub unsubscribe_packet: Unsubscribe,
}

impl HandleUnsubscribe {
    pub fn new(unsubscribe_packet: Unsubscribe) -> Self {
        Self { unsubscribe_packet }
    }
}

#[derive(Debug)]
pub struct HandleDisconnect {
    pub disconnect_packet: Disconnect,
}

impl HandleDisconnect {
    pub fn new(disconnect_packet: Disconnect) -> Self {
        println!("disconnect");
        Self { disconnect_packet }
    }
}

#[derive(Debug, Clone)]
pub struct HandleInternPuback {
    //pub packet_id: u16,
    pub puback_packet: Puback,
}

impl HandleInternPuback {
    pub fn new(puback_packet: Puback ) -> Self {
        Self { puback_packet }
    }
}

#[derive(Debug, Clone)]
pub struct HandleInternPacketId {
    pub packet_id: u16,
}

impl HandleInternPacketId {
    pub fn new(packet_id: u16 ) -> Self {
        Self { packet_id }
    }
}

#[derive(Debug, Clone)]
pub struct HandleInternPublish {
    pub publish_packet: Publish,
}

impl HandleInternPublish {
    pub fn new(publish_packet: Publish) -> Self {
        Self { publish_packet }
    }
}

