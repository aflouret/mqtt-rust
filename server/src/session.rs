use crate::topic_filters;
use common::all_packets::connect::Connect;
use common::all_packets::publish::Publish;
use common::packet::{Qos, Subscription};

//Manjea datos del cliente
#[derive(Debug)]
pub struct Session {
    client_handler_id: Option<u32>,
    client_data: ClientData,
    client_subscriptions: Vec<Subscription>,
    pub last_will_msg: Option<String>,
    pub last_will_topic: Option<String>,
    pub last_will_qos: Option<Qos>,
    pub last_will_retain: bool,
    pub unacknowledged_messages: Vec<Publish>,
    pub is_clean_session: bool,
}

impl Session {
    pub fn new(
        client_handler_id: u32,
        packet_connect: Connect,
    ) -> Result<Session, Box<dyn std::error::Error>> {
        let client_data = parse_connect_data(&packet_connect);
        let mut qos = None;
        if let Some(_msg) = &packet_connect.connect_payload.last_will_message {
            if packet_connect.last_will_qos {
                qos = Some(Qos::AtLeastOnce);
            } else {
                qos = Some(Qos::AtMostOnce);
            }
        }

        Ok(Session {
            client_handler_id: Some(client_handler_id),
            client_data,
            client_subscriptions: vec![],
            unacknowledged_messages: vec![],
            is_clean_session: packet_connect.clean_session,
            last_will_qos: qos,
            last_will_msg: packet_connect.connect_payload.last_will_message,
            last_will_topic: packet_connect.connect_payload.last_will_topic,
            last_will_retain: packet_connect.last_will_retain,
        })
    }

    pub fn get_client_id(&self) -> &String {
        &self.client_data.client_id
    }

    pub fn get_client_handler_id(&self) -> Option<u32> {
        self.client_handler_id
    }

    pub fn is_active(&self) -> bool {
        self.client_handler_id.is_some()
    }

    pub fn connect(&mut self, client_handler_id: u32) {
        self.client_handler_id = Some(client_handler_id);
    }

    pub fn disconnect(&mut self) {
        self.client_handler_id = None;

        if self.last_will_msg.is_some() {
            self.last_will_msg = None;
            self.last_will_topic = None;
            self.last_will_qos = None;
            self.last_will_retain = false;
        }
    }

    pub fn is_subscribed_to(&self, topic_name: &str) -> Option<Qos> {
        for subscription in &self.client_subscriptions {
            if topic_filters::filter_matches_topic(&subscription.topic_filter, topic_name) {
                return Some(subscription.max_qos);
            }
        }
        None
    }

    pub fn add_subscription(&mut self, mut subscription: Subscription) {
        if subscription.max_qos == Qos::ExactlyOnce {
            subscription.max_qos = Qos::AtLeastOnce;
        }

        self.client_subscriptions
            .retain(|s| s.topic_filter != subscription.topic_filter);
        self.client_subscriptions.push(subscription);
    }

    pub fn remove_subscription(&mut self, topic_filter: String) {
        self.client_subscriptions
            .retain(|s| s.topic_filter != topic_filter);
    }

    pub fn store_publish_packet(&mut self, publish_packet: Publish) {
        self.unacknowledged_messages.push(publish_packet);
    }

    pub fn update_last_will(&mut self, connect_packet: &Connect){
        self.last_will_msg = connect_packet.connect_payload.last_will_message.clone();
        self.last_will_topic = connect_packet.connect_payload.last_will_topic.clone();
        self.last_will_retain = connect_packet.last_will_retain;
        if connect_packet.last_will_qos {
            self.last_will_qos = Some(Qos::AtLeastOnce);
        } else {
            self.last_will_qos = Some(Qos::AtMostOnce);
        }
    }
}

fn parse_connect_data(packet_connect: &Connect) -> ClientData {
    ClientData {
        client_id: packet_connect.connect_payload.client_id.to_owned(),
        username: packet_connect.connect_payload.username.clone(),
        password: packet_connect.connect_payload.password.clone(),
    }
}

#[derive(PartialEq, Debug)]
pub struct ClientData {
    client_id: String,
    username: Option<String>,
    password: Option<String>,
}
