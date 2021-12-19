use crate::client_handler::ClientHandler;
use crate::config::Config;
use crate::packet_processor::PacketProcessor;
use common::packet::Packet;
use std::collections::HashMap;
use std::io;
use std::net::TcpListener;

use common::logging::logger::{LogMessage, Logger};
use std::sync::mpsc::Sender;
use std::sync::{mpsc, Mutex};
use std::sync::{Arc, RwLock};

pub type PacketResult = Result<Packet, Box<dyn std::error::Error + Send>>;
pub type ArcSenderPacket = Arc<Mutex<Sender<PacketResult>>>;

pub struct Server {
    config: Config,
    logger: Arc<Logger>,
}
impl Server {
    pub fn new(config: Config, logger: Arc<Logger>) -> io::Result<Self> {
        Ok(Self { config, logger })
    }

    pub fn server_run(self) -> Result<(), Box<dyn std::error::Error>> {
        //Inicializacion
        let address = self.config.get_address() + &*self.config.get_port();
        let listener = TcpListener::bind(&address)?;
        println!("Servidor escuchando en: {} ", &address);
        self.logger.log_msg(LogMessage::new(
            "Servidor escuchando:".to_string(),
            "8080".to_string(),
        ))?;
        let senders_to_c_h_writers = Arc::new(RwLock::new(HashMap::<
            u32,
            ArcSenderPacket,
        >::new()));
        let (c_h_reader_tx, packet_proc_rx) =
            mpsc::channel::<(u32, PacketResult)>();

        let packet_processor = PacketProcessor::new(
            packet_proc_rx,
            senders_to_c_h_writers.clone(),
            self.logger.clone(),
        );
        let packet_processor_join_handle = packet_processor.run();

        self.handle_connections(listener, senders_to_c_h_writers, c_h_reader_tx);

        packet_processor_join_handle.join().unwrap();
        Ok(())
    }

    fn handle_connections(
        &self,
        listener: TcpListener,
        senders_to_c_h_writers: Arc<
            RwLock<
                HashMap<u32, ArcSenderPacket>,
            >,
        >,
        c_h_reader_tx: Sender<(u32, PacketResult)>,
    ) {
        //let mut id: u32 = 0;
        let mut join_handles = vec![];

        for (id, stream) in listener.incoming().flatten().enumerate() {
            //if let Ok(stream) = stream {
            let client_handler = ClientHandler::new(
                id as u32,
                stream,
                senders_to_c_h_writers.clone(),
                c_h_reader_tx.clone(),
            );

            if let Ok(join_handle) = client_handler.run() {
                join_handles.push(join_handle);
            };

                //id += 1;
            //}
        }

        for handle in join_handles {
            handle.join().unwrap();
        }
    }
}
