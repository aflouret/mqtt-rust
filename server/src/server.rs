use crate::config::Config;
use crate::session::Session;
use common::all_packets::connack::Connack;
use common::all_packets::connect::Connect;
use common::packet::{Packet, WritePacket};
use common::parser;
use std::io;
use std::net::{TcpListener, TcpStream};
use std::collections::HashMap;
use std::slice::SliceIndex;
use std::thread::current;
use common::all_packets::publish::Publish;
use common::all_packets::puback::Puback;
use crate::client_handler::{ClientHandlerReader, ClientHandlerWriter};
use std::sync::{Mutex, mpsc};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::sync::{RwLock, Arc};

pub struct Server {
    config: Config,
    clients: HashMap<String, Session>,
}
//guardar sesion de un cliente
impl Server {
    pub fn new(config: Config) -> io::Result<Self> {
        Ok(Self { config, clients: HashMap::new() })
    }

    pub fn server_run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        let address = self.config.get_address() + &*self.config.get_port();

        let listener = TcpListener::bind(&address)?;

        println!("Servidor escuchando en: {} ", &address);
        
        // En este hash guardaremos los Senders correspondientes a cada Receiver en los
        // client_handler_readers, para recibir los packets que se lean desde ahí
        let client_handler_senders = Arc::new(RwLock::new(HashMap::<u32, Sender<Packet>>::new()));

        // El channel desde el cual, desde cada cliente, enviaremos los packets recién
        // leidos al server. O sea, el server tendrá el rx, y cada client_handler_reader
        // tendrá una copia del tx.
        // NOTA: el nombre antes de "_tx" o "_rx" indica quién es el que tendrá el ownership 
        // de ese extremo del channel
        let (c_h_reader_tx, server_rx) = mpsc::channel::<(u32, Packet)>();

        // Thread encargado de leer packets del channel creado arriba y procesarlos
        // TODO: sacar unwraps del thread
        let client_handler_senders_clone = client_handler_senders.clone();
        let packet_processor_handler = thread::spawn(move || {
            loop {
                let (id, packet) = server_rx.recv().unwrap();
                let sender = client_handler_senders_clone.read().unwrap().get(&id).unwrap();
                self.process_packet(packet, sender);
            }
            //self.process_packet(server_rx, client_handler_senders_clone);
        });

        let mut id: u32 = 0;
        for stream in listener.incoming() {
            if let Ok(client_stream) = stream {
                //self.handle_client(client_stream)?;

                // Channel que tiene el server con solo este cliente. Desde el server 
                // enviaremos los packets que queremos enviarle a dicho cliente
                let (server_tx, c_h_writer_rx) = mpsc::channel::<Packet>();
                
                let mut client_handler_writer = ClientHandlerWriter::new(id, Some(client_stream.try_clone()?), c_h_writer_rx);
                let mut client_handler_reader = ClientHandlerReader::new(id, Some(client_stream), c_h_reader_tx.clone());


                // Guardamos el client_handler_writer y su id en el hash
                let mut hash = client_handler_senders.write().unwrap();
                hash.insert(id, server_tx);
                id += 1;
                
                //Guardar handlers en vector??
                
                // Thread encargado de estar leyendo paquetes que lleguen a este cliente
                // (y mandándoselos al server)
                let handler_reader = thread::spawn(move || {
                    loop {
                        client_handler_reader.receive_packet().unwrap();    
                    }
                });
                
                // Thread encargado de estar escribirle al cliente los paquetes que el server
                // le envíe
                let handler_writer = thread::spawn(move || {
                    loop {
                        client_handler_writer.send_packet().unwrap();
                    }
                });
            }
        }

        packet_processor_handler.join();

        Ok(())
    }

    fn process_packet(&self, packet: Packet, sender: &Sender<Packet>) -> Result<(), Box<dyn std::error::Error>> {    
        println!("estamos en process_packet");
        let response_packet = match packet {
                Packet::Connect(connect_packet) => {
                    //self.handle_connect_packet(connect_packet)?;
                    // TODO: enviar connack usando "sender"
                }
                Packet::Publish(publish_packet) => {
                    //self.handle_publish_packet(publish_packet)?;
                    // TODO: enviar puback usando "sender"
                },
                _ => { return Err("Invalid packet".into()) },
            };
        Ok(())
    }

    // fn process_packet(&self, receiver: Receiver<(u32, Packet)>, client_handlers: Arc<RwLock<HashMap<u32, Sender<Packet>>>>) -> Result<(), Box<dyn std::error::Error>>{
    //     loop {
    //         //lee packet del channel
    //         let (id, packet) = receiver.recv()?;
        
    //         //match al tipo de packet 
    //         let response_packet = match packet {
    //             Packet::Connect(connect_packet) => {
    //                 //handle_connect_packet(connect_packet)?;
    //                 let hash = client_handlers.read().unwrap();
    //                 hash.get(&id);
    //             }
    //             Packet::Publish(publish_packet) => {
    //                 //self.handle_publish_packet(publish_packet)?;
    //             },
    //             _ => { return Err("Invalid packet".into()) },
    //         };
           
    //     }
    // }

    // Leemos y escribimos packets, etc.
    fn handle_client(&mut self, mut client_stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
        let received_packet = parser::read_packet(&mut client_stream)?;

        //Preguntar si connack y los otros paquetes los manda el servidor o la sesion
        let mut client_stream_clone = client_stream.try_clone()?;

        let client_session = if let Packet::Connect(connect_packet) = received_packet {
            self.handle_connect_packet(connect_packet, client_stream)?
        }
        else {
            return Err("No connect packet received".into())
        };

       // while client_session.is_active() {
        loop {
            let received_packet = parser::read_packet(&mut client_stream_clone)?;
            match received_packet {
                Packet::Publish(publish_packet) => {
                    self.handle_publish_packet(publish_packet)?;
                },
                _ => { return Err("Invalid packet".into()) },
            }
        }

        Ok(())
    }

    fn handle_publish_packet(&mut self, publish_packet: Publish) -> Result<(), Box<dyn std::error::Error>> {
        println!("Se recibió el publish packet");
        //Sacamos el packet_id del pubblish
        //Sacar info del publish
        //Mandamos el puback al client.
        let packet_id = 1 as u16;
        let puback_packet_response = Puback::new(packet_id);
        let current_session = self.clients.get_mut("u").unwrap(); //TODO: sacar unwrap
        let mut socket = current_session.get_socket().try_clone().unwrap();
        println!("{:?}",socket);
        puback_packet_response.write_to(&mut socket)?;
        println!("Se envio correctamente el PUBACK");
        Ok(())
    }

    fn handle_connect_packet(&mut self, connect_packet: Connect, client_stream: TcpStream) -> Result<&Session, Box<dyn std::error::Error>> {
        println!("Se recibió el connect packet");

        let client_id = connect_packet.connect_payload.client_id.to_owned();
        let clean_session = connect_packet.clean_session;
        let exists_previous_session = self.clients.contains_key(&client_id);

        // Si hay un cliente con mismo client_id conectado, lo desconectamos
        if let Some(previous_session) = self.clients.get_mut(&client_id){
            previous_session.disconnect();            
        }

        // Si no se quiere conexión persistente o no había una sesión con mismo client_id, creamos una nueva
        // Si se quiere una conexión persistente y ya había una sesión, la retomamos
        if clean_session || ! exists_previous_session {
            let new_session = Session::new(client_stream, connect_packet)?;
            self.clients.insert(new_session.get_client_id().to_string(), new_session);
        }
        let current_session = self.clients.get_mut(&client_id).unwrap(); //TODO: sacar unwrap
        current_session.connect();

        // Enviamos el connack con 0 return code y el correspondiente flag de session_present:
        // si hay clean_session, session_present debe ser false. Sino, depende de si ya teníamos sesión
        let session_present;
        if clean_session { session_present = false; }
        else { session_present = exists_previous_session; } // TODO: revisar esto, línea 683 pdf
        
        let connack_packet = Connack::new(session_present, 0);
        connack_packet.write_to(current_session.get_socket())?;
        println!("Se envió el connack packet");

        Ok(current_session)
    }
}
