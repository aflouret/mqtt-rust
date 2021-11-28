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
        setup(app);
    });

    application.run();
}

/*fn handle_button_connect_packet(sender: Sender<Packet>) -> Result<Packet, Err()> {

        let connect_packet = handle_button_connect_packet(sender.clone());



}*/

fn setup(app: &gtk::Application) {
    let glade_src = include_str!("interface.glade");
    let builder = gtk::Builder::from_string(glade_src);

    let window: gtk::Window = builder.object("main_window").unwrap();
    let (sender, recv) = mpsc::channel::<Packet>();
/*    let mut main_window = MainWindow::new(builder.clone(),sender.clone()).unwrap();
    main_window.build();*/
    handle_connect_tab(builder.clone(), sender.clone());
    handle_publish_tab(builder.clone(), sender.clone());
    handle_subscribe_tab(builder, sender);
   // main_window.handle_publish_tab(sender);
    let mut client = Client::new("User".to_owned(), "127.0.0.1:8080").unwrap();
    thread::spawn(move || {
        client.client_run(recv);
    });

    window.set_application(Some(app));
    window.show_all();
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
    println!("Value of retain: {:?}", retain_checkbox.value_type());
    println!("Value of qos: {:?}", qos_1_rb.value_type());
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


/*
        let handle_send_packets = thread::spawn(move || {
            loop {
                //self.build(sender.clone());
                let connect_button: gtk::Button = self.builder.object("connect_button").unwrap();
                let ip_label: gtk::Label = self.builder.object("ip_label").unwrap();
                connect_button.connect_clicked(clone!(@weak ip_label => move |_| {
           // ip_label.set_text("PING!");
            sender.send(Packet::Connect(connect_packet)).unwrap();
                }));
            }
        });

        let handle_recv_packets = thread::spawn(move || {
            loop {
                MainWindow::process_received_packets();
            }
        });

        handle_recv_packets.join().unwrap();
        handle_send_packets.join().unwrap();
 */


/*    let mut client = Client::new("Pepito".to_owned(), "127.0.0.1:8080".to_owned())?;

    let connect_packet = Connect::new(
        ConnectPayload::new(
            "u".to_owned(),
            Some("u".to_owned()),
            Some("u".to_owned()),
            Some("u".to_owned()),
            Some("u".to_owned()),
        ),
        60,
        true,
        true,
        true,
    );

    client.client_run(connect_packet)?;

    Ok(())*/


/*fn client_run(address: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut socket = TcpStream::connect(address)?;

    let connect_packet = Connect::new(
        ConnectPayload::new(
            "u".to_owned(),
            Some("u".to_owned()),
            Some("u".to_owned()),
            Some("u".to_owned()),
            Some("u".to_owned()),
        ),
        ConnectFlags::new(false, false, false, false, false, false),
        60,
    );

    connect_packet.write_to(&mut socket)?;
    println!("Se envió el connect packet");

    let received_packet = parser::read_packet(&mut socket)?;
    if let Packet::Connack(connack_packet) = received_packet {
        println!(
            "Se recibió el connack packet. Session present: {}. Connect return code: {}",
            connack_packet.session_present, connack_packet.connect_return_code
        );
    }

    Ok(())
}
*/