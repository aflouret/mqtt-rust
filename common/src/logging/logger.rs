use std::fmt::Debug;
use std::fs::File;
use std::io::prelude::*;
use std::sync::mpsc::Sender;
use std::sync::{mpsc, Mutex};
use std::thread;

pub struct Logger {
    logger_send: Mutex<Sender<LogMessage>>,
}

impl Logger {
    pub fn new(file_path: &str) -> Result<Logger, Box<dyn std::error::Error>> {
        if let Ok(mut file) = File::create(file_path) {
            let (sender, receiver) = mpsc::channel::<LogMessage>();
            thread::spawn(move || loop {
                let msg = receiver.recv();
                if let Ok(m) = msg {
                    file.write_all(m.msg_to_string().as_bytes()).unwrap();
                }
            });

            let sender_mtx = Mutex::new(sender);
            return Ok(Logger {
                logger_send: sender_mtx,
            });
        }
        Err("No se pudo crear el logger".into())
    }

    pub fn log_msg(&self, msg: LogMessage) -> Result<(), Box<dyn std::error::Error>> {
        println!("{:?}", msg);
        if let Ok(sender) = self.logger_send.lock() {
            sender.send(msg)?;
        } else {
            return Err("Error al loggear el mensaje".into());
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct LogMessage {
    message: String,
    client_id: String,
}

impl LogMessage {
    pub fn new(msg: String, client: String) -> LogMessage {
        LogMessage {
            message: msg,
            client_id: client,
        }
    }

    pub fn msg_to_string(self) -> String {
        self.message + " " + &*self.client_id + "\n"
    }
}

#[cfg(test)]
pub mod test_logger {
    use crate::logging::logger::LogMessage;

    #[test]
    fn test_log_message_01_() {
        let message = LogMessage::new(
            "Servidor inicializado correctamente en:".to_string(),
            "8080".to_string(),
        );
        let msg_from_log_message = "Servidor inicializado correctamente en: 8080\n".to_string();
        assert_eq!(message.msg_to_string(), msg_from_log_message);
    }
}
