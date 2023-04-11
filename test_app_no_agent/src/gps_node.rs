use std::{time::Duration, sync::Arc};
use async_trait::async_trait;
use tokio::{time::sleep, sync::Mutex, task::JoinHandle};

use crate::messages::{PositionMsg, PositionProvider};

pub struct GpsNode {
    last_position: Arc<Mutex<Option<PositionMsg>>>,
    task: JoinHandle<()>
}

impl GpsNode {
    pub fn new() -> GpsNode {
        let last_position = Arc::new(Mutex::new(None));
        let res = GpsNode{
            last_position: last_position.clone(),
            task: tokio::spawn(async move {read_serial_port(last_position).await})
        };
    
        return res;
    }

    
}

impl Drop for GpsNode {
    fn drop(&mut self) {
        self.task.abort();
    }
}

pub async fn read_serial_port(last_position: Arc<Mutex<Option<PositionMsg>>>) {
    loop{
        sleep(Duration::from_millis(100)).await;
        *last_position.lock().await = Some(PositionMsg{latitude: 43.000, longitude: 6.000, heading: 10.0, is_valid: true});
    }
}

#[async_trait]
impl PositionProvider for GpsNode {
    async fn get_last_position(&self) -> Option<PositionMsg> {
        self.last_position.lock().await.clone()
    }
}