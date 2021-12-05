use common::all_packets::connack::Connack;
use common::all_packets::puback::Puback;
use common::all_packets::suback::Suback;

pub enum ResponseHandlers {
    ConnackResponse(ConnackResponse),
    PubackResponse(PubackResponse),
    SubackResponse(SubackResponse),
}


#[derive(Debug)]
pub struct ConnackResponse {
    pub connack_packet: Connack,
    pub msg: String,
}

impl ConnackResponse {
    pub fn new(connack_packet: Connack, msg: String) -> Self {
        Self{connack_packet: connack_packet, msg: msg}
    }

}

#[derive(Debug)]
pub struct PubackResponse {
    pub puback_packet: Puback,
}

impl PubackResponse {
    pub fn new(puback_packet: Puback) -> Self {
        Self{puback_packet: puback_packet}
    }

}

#[derive(Debug)]
pub struct SubackResponse {
    pub suback_packet: Suback,
}

impl SubackResponse {
    pub fn new(suback_packet: Suback) -> Self {
        Self{suback_packet: suback_packet}
    }

}