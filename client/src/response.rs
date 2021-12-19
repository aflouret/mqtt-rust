
use common::all_packets::publish::Publish;

#[derive(Debug)]
pub enum ResponseHandlers {
    PublishResponse(PublishResponse),
    PubackResponse(PubackResponse),
}


#[derive(Debug)]
pub struct PublishResponse {
    pub publish_packet: Publish,
    pub msgs: Vec<String>,
    pub msg_correct: String,
}

impl PublishResponse {
    pub fn new(publish_packet: Publish, msgs: Vec<String>, msg_correct: String) -> Self {
        Self{publish_packet, msgs, msg_correct }
    }

}

#[derive(Debug)]
pub struct PubackResponse {
    pub msg: String

}

impl PubackResponse {
    pub fn new(msg: String) -> Self {
        Self{msg}
    }

}
