use std::{io, thread};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender, SendError};
use crate::client::Client;

extern crate glib;
extern crate gtk;

use glib::clone;
use gtk::Application;
use gtk::prelude::*;
use common::all_packets::connect::{Connect, ConnectPayload};
use common::packet::Packet;

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
        thread::spawn(move || {
           client.client_run(recv);
        });
/*        client.client_run(recv);*/
        build_ui(app, sender);
    });

    application.run();
}

fn build_ui(app: &gtk::Application, sender: Sender<Packet>) {
    let glade_src = include_str!("interface.glade");
    let builder = gtk::Builder::from_string(glade_src);

    let window: gtk::Window = builder.object("main_window").unwrap();
    let mut main_window = MainWindow::new(builder).unwrap();
    main_window.build(sender);

    window.set_application(Some(app));
    window.show_all();
}

struct MainWindow {
    builder: gtk::Builder,
}

impl MainWindow {
    pub fn new(builder: gtk::Builder) -> io::Result<Self> {
        Ok(Self { builder: builder})
    }
    pub fn build(self, sender: Sender<Packet>) {
        let connect_button: gtk::Button = self.builder.object("connect_button").unwrap();
        let ip_label: gtk::Label = self.builder.object("ip_label").unwrap();
        let ip_entry: gtk::Entry = self.builder.object("ip_entry").unwrap();
        let port_entry: gtk::Entry = self.builder.object("port_entry").unwrap();
        let client_id_entry: gtk::Entry = self.builder.object("clientId_entry").unwrap();
        let username_entry: gtk::Entry = self.builder.object("username_entry").unwrap();
        let mut usr = username_entry.text().to_owned().to_string();
        let password_entry: gtk::Entry = self.builder.object("pass_entry").unwrap();
        let last_will_msg_entry: gtk::Entry = self.builder.object("lastWillMsg_entry").unwrap();
        let last_will_topic_entry: gtk::Entry = self.builder.object("lastWillTopic_entry").unwrap();
        connect_button.connect_clicked(clone!(@weak ip_label => move |_| {
            println!("{}",&usr);
            sender.send(Packet::Connect(Connect::new(
                        ConnectPayload::new(
                            client_id_entry.to_owned().to_string(),
                            Some(last_will_topic_entry.to_owned().to_string()),
                            Some(last_will_msg_entry.to_owned().to_string()),
                            Some(usr.to_owned()),
                            Some(password_entry.to_owned().to_string()),
                        ),
                        60,
                        true,
                        true,
                        true,
            )));
        }));
        //sender.send()
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