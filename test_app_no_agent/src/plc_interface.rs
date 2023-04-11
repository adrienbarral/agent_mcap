use std::{io::Write, time::Duration, sync::Arc};
use anyhow::Result;
use tokio::{time::sleep, sync::Mutex, task::JoinHandle};
use tokio_serial::{SerialStream, SerialPortBuilder};

use crate::messages::{CommandSender, ActuatorCommandsMsg};

pub struct PLCInterface {
    port: Arc<Mutex<Option<SerialStream>>>,
    auto_reconnect_task: JoinHandle<()>
}

impl PLCInterface {
    pub fn new(com_port: String) -> Self {
         let serial_port = Arc::new(Mutex::new(None));
         PLCInterface {
            port: serial_port.clone(),
            auto_reconnect_task: tokio::spawn(async move {
                auto_reconnect(tokio_serial::new(com_port, 9600), serial_port).await;
            })
        }
    }

}

async fn auto_reconnect(serial_port_builder: SerialPortBuilder, port: Arc<Mutex<Option<SerialStream>>>) {
    loop {
        if port.lock().await.is_none() {
            // On utilise le "None" de l'option pour signifier que le port n'est pas ouvert.
            if let Ok(stream) = tokio_serial::SerialStream::open(&serial_port_builder) {
                *port.lock().await = Some(stream);
            }
        }
        sleep(Duration::from_millis(100)).await;
    }
}    


#[async_trait::async_trait]
impl CommandSender for PLCInterface {
    async fn send_command(&self, command: ActuatorCommandsMsg) -> Result<()> {
        if let Some(port) = self.port.lock().await.as_mut() {
            port.write_fmt(format_args!("$CMD,{:1},{:2}", command.engine, command.rudder))?;
        }
        Ok(())
    }
}

impl Drop for PLCInterface {
    fn drop(&mut self) {
        self.auto_reconnect_task.abort();
    }
}
