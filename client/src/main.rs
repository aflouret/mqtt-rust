use std::{io, thread};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender, SendError};
use std::time::Duration;
use crate::client::Client;

extern crate glib;
extern crate gtk;

use glib::clone;
use gtk::{Application, Builder, TextView};
use gtk::prelude::*;
use common::all_packets::connect::{Connect, ConnectPayload};
use common::all_packets::publish::{Publish, PublishFlags};
use common::all_packets::subscribe::{Subscribe};
use common::all_packets::unsubscribe::{Unsubscribe};
use common::packet::{Packet, Qos, Subscription};
use crate::handlers::EventHandlers;
use crate::handlers::HandleConection;
use crate::handlers::HandlePublish;
use crate::handlers::HandleSubscribe;
use crate::handlers::HandleUnsubscribe;
use crate::response::{PublishResponse, ResponseHandlers};

mod client;
mod handlers;
mod response;

// -> Result<(), Box<dyn std::error::Error>>
/*
    //Sender: Client envía a Window , Recv: Window recibe data a mostrar
    let (sender_cli, recv_window) = mpsc::channel::<Packet>();
 */
fn main() {
    let application = gtk::Application::new(Some("com.taller.pong"), Default::default());

    application.connect_activate(|app| {
        let (sender_conection, recv_conection) = mpsc::channel::<EventHandlers>();
        let (client_sender, window_recv) = mpsc::channel::<ResponseHandlers>();
        let handler_to_client = thread::spawn(move || {
            let mut client = Client::new("User".to_owned());
            client.start_client(recv_conection, client_sender);
        });
        let builder = build_ui(app);
        setup(builder, sender_conection.clone(), window_recv);
    });

    application.run();
}

///Devuelve un objeto gtk::Builder al levantar un .glade
fn build_ui(app: &gtk::Application) -> gtk::Builder {
    let glade_src = include_str!("interface.glade");
    let mut builder = gtk::Builder::from_string(glade_src);
    let response_publish: gtk::Label = builder.object("response_publish").unwrap();
    response_publish.set_text("");
    let window: gtk::Window = builder.object("main_window").unwrap();
    window.set_application(Some(app));
    window.show_all();
    builder
}

fn setup(builder: gtk::Builder, sender_conec: Sender<EventHandlers>, window_recv: Receiver<ResponseHandlers>) {
    handle_connect_tab(builder.clone(), sender_conec.clone());
    handle_publish_tab(builder.clone(), sender_conec.clone());
    handle_subscribe_tab(builder.clone(), sender_conec.clone());
    handle_unsubscribe(builder.clone(), sender_conec);
    // main_window.handle_publish_tab(sender);
    let (intern_sender, intern_recv) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    thread::spawn(move || {
        loop {
            if let Ok(response) = window_recv.recv() {
                match response {
                    ResponseHandlers::PublishResponse(publish) => {
                        intern_sender.send(publish).unwrap();
                    },
                    _ => ()
                }
            }
        }
    });

    let response_publish: gtk::Label = builder.object("response_publish").unwrap();
    let buffer: gtk::TextBuffer = builder.object("textbuffer1").unwrap();
    let mut joined_string: String = "".to_string();
    intern_recv.attach(None, move |publish: PublishResponse| {
        if let Some(msg) = publish.msgs.get(publish.msgs.len() - 1) {
            joined_string += msg;
        }
        buffer.set_text(&joined_string);
        let publish_correct_msg = publish.msg_correct;
        response_publish.set_text(&*publish_correct_msg);
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
    connect_button.connect_clicked(clone!(@weak username_entry  => move |_| {
        let address = (&ip_entry.text()).to_string() + ":" + &*(&port_entry.text()).to_string();
        let event_conection = EventHandlers::HandleConection(HandleConection::
            new(address.clone(),(&client_id_entry.text()).to_string(),
                true, true, true, (&keep_alive_entry.text()).to_string(),
                Some((&username_entry.text()).to_string()),
                Some((&password_entry.text()).to_string()),
                Some((&last_will_msg_entry.text()).to_string()),
                Some((&last_will_topic_entry.text()).to_string()),
        ));
        sender.send(event_conection);
/*        username_entry.set_text("");
        password_entry.set_text("");
        client_id_entry.set_text("");
        last_will_msg_entry.set_text("");
        last_will_topic_entry.set_text("");
        ip_entry.set_text("");
        port_entry.set_text("");*/
        }));

/*    let disconnect_button: gtk::Button = builder.object("disconnect_button").unwrap();
    disconnect_button.connect_clicked(clone! (@weak disconnect_button => move |_| {
        sender.send(Disconnect::new())
    }));*/
}

fn handle_publish_tab(builder: gtk::Builder, sender: Sender<EventHandlers>) {
    let topic_pub_entry: gtk::Entry = builder.object("topic_publish_entry").unwrap();
    let app_msg_entry: gtk::Entry = builder.object("appmsg_entry").unwrap();
    let qos_0_rb: gtk::RadioButton = builder.object("qos_0_radiobutton").unwrap();
    let qos_1_rb: gtk::RadioButton = builder.object("qos_1_radiobutton").unwrap();
    let retain_checkbox: gtk::CheckButton = builder.object("retain_checkbox").unwrap();
    let publish_button: gtk::Button = builder.object("publish_button").unwrap();
    publish_button.connect_clicked(clone!( @weak topic_pub_entry => move |_| {
         let b = qos_1_rb.is_active();
        println!("{:?}", &b);
        let r = retain_checkbox.is_active();
        println!("{:?}", &r);
        let publish_packet = Publish::new(
            PublishFlags::new(0b0011_0001),  //0b0011_101   0b0011_000
            (&topic_pub_entry.text()).to_string(),
            None,
            (&app_msg_entry.text()).to_string(),
        );
        let event_publish = EventHandlers::HandlePublish(HandlePublish::new(publish_packet));
        sender.send(event_publish);
    }));
}

fn handle_subscribe_tab(builder: gtk::Builder, sender: Sender<EventHandlers>) {
    let subscribe_button: gtk::Button = builder.object("suscribe_button").unwrap();
    let topic_subscribe_entry: gtk::Entry = builder.object("topic_suscribe_entry").unwrap();
    let text_view: gtk::TextView = builder.object("text_view").unwrap();
    let buffer: gtk::TextBuffer = builder.object("textbuffer1").unwrap();
    /*    buffer.set_text("Probando");
        text_view.buffer().unwrap();*/
    /*    text_view.set_tooltip_text(Some("Probando"));*/
    subscribe_button.connect_clicked(clone!( @weak topic_subscribe_entry => move |_| {
        //text_view.buffer().unwrap();
        let mut subscribe_packet = Subscribe::new(10);
        subscribe_packet.add_subscription(Subscription{topic_filter: (&topic_subscribe_entry.text()).to_string(), max_qos: Qos::AtLeastOnce});

        let event_subscribe = EventHandlers::HandleSubscribe(HandleSubscribe::new(subscribe_packet));
        sender.send(event_subscribe);
    }));
}

fn handle_unsubscribe(builder: gtk::Builder, sender: Sender<EventHandlers>) {
    let unsubscribe_button: gtk::Button = builder.object("unsubscribe_button").unwrap();
    let unsubscribe_topic_entry: gtk::Entry = builder.object("topic_unsubscribe_entry").unwrap();

    unsubscribe_button.connect_clicked(clone!( @weak unsubscribe_topic_entry => move |_| {
        let mut unsubs_packet = Unsubscribe::new(10);
        unsubs_packet.add_topic((&unsubscribe_topic_entry.text()).to_string());

        let event_unsubscribe = EventHandlers::HandleUnsubscribe(HandleUnsubscribe::new(unsubs_packet));
        sender.send(event_unsubscribe);
    }));
}

struct MainWindow {
    builder: gtk::Builder,
    sender: Sender<Packet>,
}

impl MainWindow {
    pub fn new(builder: gtk::Builder, sender: Sender<Packet>) -> io::Result<Self> {
        Ok(Self { builder: builder, sender: sender })
    }
    pub fn build(self) {
        let connect_button: gtk::Button = self.builder.object("connect_button").unwrap();
        let ip_entry: gtk::Entry = self.builder.object("ip_entry").unwrap();
        let port_entry: gtk::Entry = self.builder.object("port_entry").unwrap();
        let client_id_entry: gtk::Entry = self.builder.object("clientId_entry").unwrap();
        let username_entry: gtk::Entry = self.builder.object("username_entry").unwrap();
        let password_entry: gtk::Entry = self.builder.object("pass_entry").unwrap();
        let last_will_msg_entry: gtk::Entry = self.builder.object("lastWillMsg_entry").unwrap();
        let last_will_topic_entry: gtk::Entry = self.builder.object("lastWillTopic_entry").unwrap();
        let address = (&ip_entry.text()).to_string() + ":" + &*(&port_entry.text()).to_string();
        connect_button.connect_clicked(clone!(@weak username_entry  => move |_| {
            self.sender.send(Packet::Connect(Connect::new(
            ConnectPayload::new((&client_id_entry.text()).to_string(),
                                Some((&last_will_topic_entry.text()).to_string()),
                                Some((&last_will_msg_entry.text()).to_string()),
                                Some((&username_entry.text()).to_string()),
                                Some((&password_entry.text()).to_string()),
            ),
            60,
            true,
            true,
            true,
        )));
                    username_entry.set_text("");
        password_entry.set_text("");
        client_id_entry.set_text("");
        last_will_msg_entry.set_text("");
        last_will_topic_entry.set_text("");
        ip_entry.set_text("");
        port_entry.set_text("");
        }));
    }


    pub fn handle_publish_tab(self) {
        let topic_pub_entry: gtk::Entry = self.builder.object("topic_publish_entry").unwrap();
        let app_msg_entry: gtk::Entry = self.builder.object("appmsg_entry").unwrap();
        let qos_0_rb: gtk::RadioButton = self.builder.object("qos_0_radiobutton").unwrap();
        let qos_1_rb: gtk::RadioButton = self.builder.object("qos_1_radiobutton").unwrap();
        let retain_checkbox: gtk::CheckButton = self.builder.object("retain_checkbox").unwrap();
        let publish_button: gtk::Button = self.builder.object("publish_button").unwrap();
        println!("Value of retain: {:?}", retain_checkbox.to_value());
        println!("Value of qos: {:?}", qos_1_rb.to_value());
        publish_button.connect_clicked(clone!( @weak topic_pub_entry => move |_| {
            self.sender.send(Packet::Publish(Publish::new(
                PublishFlags::new(0b0100_1011),
                (&topic_pub_entry.text()).to_string(),
                None,
                (&app_msg_entry.text()).to_string(),
        )));
    }));
    }
}
