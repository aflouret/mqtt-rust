use std::{io, thread};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender, SendError};
use crate::client::Client;

extern crate glib;
extern crate gtk;

use glib::clone;
use gtk::{Application, Builder};
use gtk::prelude::*;
use common::all_packets::connect::{Connect, ConnectPayload};
use common::all_packets::publish::{Publish, PublishFlags};
use common::all_packets::subscribe::{Subscribe, Topic};
use common::packet::{Packet, Qos};

mod client;
mod client_controller;

// -> Result<(), Box<dyn std::error::Error>>
/*
    //Sender: Client envía a Window , Recv: Window recibe data a mostrar
    let (sender_cli, recv_window) = mpsc::channel::<Packet>();
 */
fn main() {
    let application = gtk::Application::new(Some("com.taller.pong"), Default::default());

    application.connect_activate(|app| {
        let (sender, recv) = mpsc::channel::<Packet>();
        let mut client = Client::new("User".to_owned(), "127.0.0.1:8080").unwrap();
        let handler_to_client = thread::spawn(move || {
            client.client_run(recv);
        });
        let builder = build_ui(app);
        setup(builder,sender);
    });

    application.run();
}

/*fn handle_button_connect_packet(sender: Sender<Packet>) -> Result<Packet, Err()> {

        let connect_packet = handle_button_connect_packet(sender.clone());



}*/
///Devuelve un objeto gtk::Builder al levantar un .glade
fn build_ui(app: &gtk::Application) -> gtk::Builder {
    let glade_src = include_str!("interface.glade");
    let mut builder = gtk::Builder::from_string(glade_src);
    let window: gtk::Window = builder.object("main_window").unwrap();
    window.set_application(Some(app));
    window.show_all();
    builder
}

fn setup(builder: gtk::Builder, sender: Sender<Packet>) {
    //let (sender, recv) = mpsc::channel::<Packet>();
    let (client_sender, window_recv) = mpsc::channel::<String>();
    /*    let mut main_window = MainWindow::new(builder.clone(),sender.clone()).unwrap();
        main_window.build();*/
    handle_connect_tab(builder.clone(), sender.clone());
    handle_publish_tab(builder.clone(), sender.clone());
    handle_subscribe_tab(builder.clone(), sender);
    // main_window.handle_publish_tab(sender);
    let (intern_sender, intern_recv) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    thread::spawn(move || {
        let response = window_recv.recv().unwrap();
        println!("RESPONSE: {:?}", &response);
        intern_sender.send(response);
    });
    let username_label: gtk::Label = builder.object("usr_label").unwrap();

    intern_recv.attach(None, move |text: String| {
        username_label.set_text(text.as_str());
        glib::Continue(true)
    });
}

fn handle_connect_tab(builder: gtk::Builder, sender: Sender<Packet>) {
    let connect_button: gtk::Button = builder.object("connect_button").unwrap();
    let ip_entry: gtk::Entry = builder.object("ip_entry").unwrap();
    let port_entry: gtk::Entry = builder.object("port_entry").unwrap();
    let client_id_entry: gtk::Entry = builder.object("clientId_entry").unwrap();
    let username_entry: gtk::Entry = builder.object("username_entry").unwrap();
    let password_entry: gtk::Entry = builder.object("pass_entry").unwrap();
    let last_will_msg_entry: gtk::Entry = builder.object("lastWillMsg_entry").unwrap();
    let last_will_topic_entry: gtk::Entry = builder.object("lastWillTopic_entry").unwrap();
    let address = (&ip_entry.text()).to_string() + ":" + &*(&port_entry.text()).to_string();
    connect_button.connect_clicked(clone!(@weak username_entry  => move |_| {
            sender.send(Packet::Connect(Connect::new(
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

fn handle_publish_tab(builder: gtk::Builder, sender: Sender<Packet>) {
    let topic_pub_entry: gtk::Entry = builder.object("topic_publish_entry").unwrap();
    let app_msg_entry: gtk::Entry = builder.object("appmsg_entry").unwrap();
    let qos_0_rb: gtk::RadioButton = builder.object("qos_0_radiobutton").unwrap();
    let qos_1_rb: gtk::RadioButton = builder.object("qos_1_radiobutton").unwrap();
    let retain_checkbox: gtk::CheckButton = builder.object("retain_checkbox").unwrap();
    let publish_button: gtk::Button = builder.object("publish_button").unwrap();
    publish_button.connect_clicked(clone!( @weak topic_pub_entry => move |_| {
        sender.send(Packet::Publish(Publish::new(
            PublishFlags::new(0b0100_1011),
            (&topic_pub_entry.text()).to_string(),
            None,
            (&app_msg_entry.text()).to_string(),
        )));
    }));
}

fn handle_subscribe_tab(builder: gtk::Builder, sender: Sender<Packet>) {
    let suscribe_button: gtk::Button = builder.object("suscribe_button").unwrap();
    let topic_subscribe_entry: gtk::Entry = builder.object("topic_suscribe_entry").unwrap();
    suscribe_button.connect_clicked(clone!( @weak topic_subscribe_entry => move |_| {
        sender.send(Packet::Subscribe(Subscribe::new(10,
                    Topic{name: (&topic_subscribe_entry.text()).to_string(), qos: Qos::AtMostOnce}
        )));
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
