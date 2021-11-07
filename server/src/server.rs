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

        for stream in listener.incoming() {
            if let Ok(mut client_stream) = stream {
                self.handle_client(client_stream)?;

            }
        }

        Ok(())
    }

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

        while client_session.is_active() {
            let received_packet = parser::read_packet(&mut client_stream_clone)?;
            match received_packet {
                Packet::Publish(publish_packet) => {println!("Recibi el publish");},
                _ => {return Err("Invalid packet".into())},
            }
        }

        Ok(())
    }

    fn handle_connect_packet(&mut self, connect_packet: Connect, client_stream: TcpStream) -> Result<&Session, Box<dyn std::error::Error>> {
        println!("Se recibió el connect packet");

        let client_id = connect_packet.connect_payload.client_id.to_owned();
        let clean_session = connect_packet.connect_flags.clean_session;
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
