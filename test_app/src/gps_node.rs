use crate::messages::PositionMsg;
use std::{time::Duration};
use anyhow::Result;
use agent_mcap::{Topic, Context};
use tokio::{time::sleep};

pub struct GPSNode {
    name: String,
    gps_topic: Topic<PositionMsg>
}

impl GPSNode {
    pub async fn new(name: &str, context: &mut Context) -> Result<Self> {
        Ok(GPSNode { name: String::from(name),
        gps_topic: context.advertise("gps_pos").await?
        })
    }
    pub async fn task(&mut self){
        loop{
            sleep(Duration::from_millis(100)).await;
            self.gps_topic.publish(PositionMsg{latitude: 43.000, longitude: 6.000, heading: 10.0, is_valid: true});
        }
    }
}
