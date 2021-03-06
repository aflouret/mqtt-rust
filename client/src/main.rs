use crate::client::Client;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

extern crate glib;
extern crate gtk;

use crate::handlers::{
    EventHandlers, HandleConection, HandleDisconnect, HandlePublish, HandleSubscribe,
    HandleUnsubscribe, LastWillInfo,
};
use crate::response::{PubackResponse, PublishResponse, ResponseHandlers};
use common::all_packets::disconnect::Disconnect;
use common::all_packets::unsubscribe::Unsubscribe;
use glib::clone;
use gtk::prelude::*;

mod client;
mod client_puback_processor;
mod handlers;
mod response;

fn main() {
    let application = gtk::Application::new(None, Default::default());
    application.connect_activate(|app| {
        let builder = build_ui(app);
        let (sender_connection, recv_connection) = mpsc::channel::<EventHandlers>();
        let (client_sender, window_recv) = mpsc::channel::<ResponseHandlers>();
        setup(builder, sender_connection.clone(), window_recv);
        thread::spawn(move || {
            let client = Client::new();
            client
                .start_client(recv_connection, client_sender, sender_connection)
                .unwrap();
        });
    });

    application.run();
}

///Devuelve un objeto gtk::Builder al levantar un .glade
fn build_ui(app: &gtk::Application) -> gtk::Builder {
    let glade_src = include_str!("interface.glade");
    let builder = gtk::Builder::from_string(glade_src);
    let response_publish: gtk::Label = builder.object("response_publish").unwrap();
    response_publish.set_text("");
    let window: gtk::Window = builder.object("main_window").unwrap();
    window.set_application(Some(app));
    window.show_all();
    builder
}

fn setup(
    builder: gtk::Builder,
    sender_conec: Sender<EventHandlers>,
    window_recv: Receiver<ResponseHandlers>,
) {
    handle_connect_tab(builder.clone(), sender_conec.clone());
    handle_publish_tab(builder.clone(), sender_conec.clone());
    handle_subscribe_tab(builder.clone(), sender_conec.clone());
    handle_unsubscribe(builder.clone(), sender_conec);
    let (intern_sender, intern_recv) =
        glib::MainContext::channel::<ResponseHandlers>(glib::PRIORITY_DEFAULT);
    thread::spawn(move || loop {
        if let Ok(response) = window_recv.recv() {
            match response {
                ResponseHandlers::PublishResponse(publish) => {
                    intern_sender
                        .send(ResponseHandlers::PublishResponse(PublishResponse::new(
                            publish.publish_packet,
                            publish.msgs.clone(),
                            publish.msg_correct,
                        )))
                        .unwrap();
                }
                ResponseHandlers::PubackResponse(puback) => {
                    intern_sender
                        .send(ResponseHandlers::PubackResponse(PubackResponse::new(
                            puback.msg,
                        )))
                        .unwrap();
                }
            }
        }
    });

    let response_publish: gtk::Label = builder.object("response_publish").unwrap();
    let buffer: gtk::TextBuffer = builder.object("textbuffer1").unwrap();
    let mut joined_string: String = "".to_string();
    intern_recv.attach(None, move |response: ResponseHandlers| {
        match response {
            ResponseHandlers::PublishResponse(publish) => {
                if let Some(msg) = publish.msgs.last() {
                    joined_string += msg;
                }
                buffer.set_text(&joined_string);
            }
            ResponseHandlers::PubackResponse(puback) => {
                let puback_msg = puback.msg;
                response_publish.set_text(&*puback_msg);
            }
        }
        glib::Continue(true)
    });
}

fn handle_connect_tab(builder: gtk::Builder, sender: Sender<EventHandlers>) {
    let connect_button: gtk::Button = builder.object("connect_button").unwrap();
    let ip_entry: gtk::Entry = builder.object("ip_entry").unwrap();
    let port_entry: gtk::Entry = builder.object("port_entry").unwrap();
    let client_id_entry: gtk::Entry = builder.object("clientId_entry").unwrap();
    let username_entry: gtk::Entry = builder.object("username_entry").unwrap();
    let password_entry: gtk::Entry = builder.object("pass_entry").unwrap();
    let last_will_msg_entry: gtk::Entry = builder.object("lastWillMsg_entry").unwrap();
    let last_will_topic_entry: gtk::Entry = builder.object("lastWillTopic_entry").unwrap();
    let keep_alive_entry: gtk::Entry = builder.object("keep_alive_entry").unwrap();
    let clean_session: gtk::CheckButton = builder.object("clean_session_check").unwrap();
    let last_will_retain: gtk::CheckButton = builder.object("last_will_retain_check").unwrap();
    let last_will_qos: gtk::CheckButton = builder.object("last_will_qos_check").unwrap();

/*    ip_entry.set_text("200.126.213.24");
    port_entry.set_text("8080");*/
    keep_alive_entry.set_text("300");

    let sender_for_disconnect = sender.clone();
    connect_button.connect_clicked(clone!(@weak username_entry  => move |_| {
    let address = (&ip_entry.text()).to_string() + ":" + &*(&port_entry.text()).to_string();
    let a  = clean_session.is_active();
    let b = last_will_retain.is_active();
    let c = last_will_qos.is_active();

    let mut username: Option<String> = None;
    if &username_entry.text() != "" {
        username = Some((&username_entry.text()).to_string());
    }

    let mut password: Option<String> = None;
    if &password_entry.text() != "" {
        password = Some((&password_entry.text()).to_string());
    }

    let mut last_will_msg: Option<String> = None;
    if &last_will_msg_entry.text() != "" {
        last_will_msg = Some((&last_will_msg_entry.text()).to_string());
    }

    let mut last_will_topic: Option<String> = None;
    if &last_will_topic_entry.text() != "" {
        last_will_topic = Some((&last_will_topic_entry.text()).to_string());
    }

    let event_conection = EventHandlers::Conection(HandleConection::
        new(address,(&client_id_entry.text()).to_string(),
            a,
            (&keep_alive_entry.text()).to_string(),
            username,
            password,
            LastWillInfo::new(last_will_topic, last_will_msg, c, b)
    ));
    sender.send(event_conection).unwrap();
    }));

    let disconnect_button: gtk::Button = builder.object("disconnect_button").unwrap();
    disconnect_button.connect_clicked(clone! (@weak disconnect_button => move |_| {
        let event_disconection = EventHandlers::Disconnect(HandleDisconnect::new(Disconnect::new()));
        sender_for_disconnect.send(event_disconection).unwrap();
    }));
}

fn handle_publish_tab(builder: gtk::Builder, sender: Sender<EventHandlers>) {
    let topic_pub_entry: gtk::Entry = builder.object("topic_publish_entry").unwrap();
    let app_msg_entry: gtk::Entry = builder.object("appmsg_entry").unwrap();
    let qos_0_rb: gtk::RadioButton = builder.object("qos_0_radiobutton").unwrap();
    let qos_1_rb: gtk::RadioButton = builder.object("qos_1_radiobutton").unwrap();
    let retain_checkbox: gtk::CheckButton = builder.object("retain_checkbox").unwrap();
    let publish_button: gtk::Button = builder.object("publish_button").unwrap();
    publish_button.connect_clicked(clone!( @weak topic_pub_entry => move |_| {
        let event_publish = EventHandlers::Publish(HandlePublish::new(
            (&topic_pub_entry.text()).to_string(),(&app_msg_entry.text()).to_string(),
            qos_0_rb.is_active(), qos_1_rb.is_active(), retain_checkbox.is_active()));
        sender.send(event_publish).unwrap();
    }));
}

fn handle_subscribe_tab(builder: gtk::Builder, sender: Sender<EventHandlers>) {
    let subscribe_button: gtk::Button = builder.object("suscribe_button").unwrap();
    let topic_subscribe_entry: gtk::Entry = builder.object("topic_suscribe_entry").unwrap();
    let _buffer: gtk::TextBuffer = builder.object("textbuffer1").unwrap();

    let qos_0_rb: gtk::RadioButton = builder.object("qos_0_rb_subscribe").unwrap();

    subscribe_button.connect_clicked(clone!( @weak topic_subscribe_entry => move |_| {
        let event_subscribe = EventHandlers::Subscribe(HandleSubscribe::new(
            (&topic_subscribe_entry.text()).to_string(), qos_0_rb.is_active()));
        sender.send(event_subscribe).unwrap();
    }));
}

fn handle_unsubscribe(builder: gtk::Builder, sender: Sender<EventHandlers>) {
    let unsubscribe_button: gtk::Button = builder.object("unsubscribe_button").unwrap();
    let unsubscribe_topic_entry: gtk::Entry = builder.object("topic_unsubscribe_entry").unwrap();

    unsubscribe_button.connect_clicked(clone!( @weak unsubscribe_topic_entry => move |_| {
        let mut unsubs_packet = Unsubscribe::new(10);
        unsubs_packet.add_topic((&unsubscribe_topic_entry.text()).to_string());

        let event_unsubscribe = EventHandlers::Unsubscribe(HandleUnsubscribe::new(unsubs_packet));
        sender.send(event_unsubscribe).unwrap();
    }));
}
