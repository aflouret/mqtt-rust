use common::all_packets::connack::Connack;
use common::all_packets::puback::Puback;
use common::all_packets::publish::Publish;
use common::all_packets::suback::Suback;
#[derive(Debug)]
pub enum ResponseHandlers {
    ConnackResponse(ConnackResponse),
    SubackResponse(SubackResponse),
    PublishResponse(PublishResponse),
    PubackResponse(PubackResponse),
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
pub struct PublishResponse {
    pub publish_packet: Publish,
    pub msgs: Vec<String>,
    pub msg_correct: String,
}

impl PublishResponse {
    pub fn new(publish_packet: Publish, msgs: Vec<String>, msg_correct: String) -> Self {
        Self{publish_packet: publish_packet, msgs: msgs, msg_correct: msg_correct }
    }

}

#[derive(Debug)]
pub struct PubackResponse {
    pub msg: String

}

impl PubackResponse {
    pub fn new(msg: String) -> Self {
        Self{msg: msg}
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