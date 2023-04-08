use crate::messages::{PositionMsg, Event, EventType};
use std::{time::Duration};
use anyhow::Result;
use agent_mcap::{Topic, Context};
use tokio::{time::sleep, sync::broadcast::Sender};

pub struct GPSNode {
    name: String,
    gps_topic: Topic<PositionMsg>,
    event_topic: Sender<Option<Event>>
}

impl GPSNode {
    pub async fn new(name: &str, context: &mut Context, event: &Topic<Event>) -> Result<Self> {
        Ok(GPSNode { name: String::from(name),
        gps_topic: context.advertise("gps_pos").await?,
        event_topic: event.tx.clone()
        })
    }
    pub async fn task(&mut self){
        loop{
            sleep(Duration::from_millis(100)).await;
            self.gps_topic.publish(PositionMsg{latitude: 43.000, longitude: 6.000, heading: 10.0, is_valid: true});

            if self.event_topic.send(Some(Event{emitter: self.name.clone(), event: EventType::GPS_FIX_LOST })).is_err() {
                // Discutable de tout arrêter si on arrive pas à émettre !
                return;
            };
        }
    }
}
