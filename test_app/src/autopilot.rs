use std::{time::Duration, sync::Arc};
use crate::messages::{PositionMsg, ActuatorCommandsMsg};

use anyhow::Result;
use log::debug;
use agent_mcap::{Topic, Context, synchronize_data};

use tokio::{time::sleep, sync::Mutex};

pub struct Autopilot {
    name: String, 
    actuators_command: Topic<ActuatorCommandsMsg>,
    last_position: Arc<Mutex<Option<PositionMsg>>>,
    current_goal_point: Arc<Mutex<Option<PositionMsg>>>
}

fn compute_commands(_last_position: &PositionMsg, _goal: &PositionMsg) -> ActuatorCommandsMsg {
    // Perform complex computation here !
    ActuatorCommandsMsg { rudder: 0.0f32, engine: 0.0f32 }
}

// TODO : recevoir le général state pour tout arrêter sur transition de reaching à autre chose...
impl Autopilot {
    pub async fn new(name: &str, context: &mut Context, gps_topic: &Topic<PositionMsg>, goal_point: &Topic<PositionMsg>) -> Result<Self> {
        let res = Autopilot {
            name: String::from(name),
            actuators_command: context.advertise("actuators_command").await?,
            last_position: Arc::new(Mutex::new(None)),
            current_goal_point: Arc::new(Mutex::new(None)),
        };

        synchronize_data(gps_topic, res.last_position.clone());
        synchronize_data(goal_point, res.current_goal_point.clone());

        Ok(res)
    }
    pub async fn task(&mut self){
        loop{
            sleep(Duration::from_millis(100)).await;
            // Control loop here
            match (self.current_goal_point.lock().await.take(), self.check_and_get_last_position().await) {
                (Some(gp), Some(pos)) => {
                    let command = compute_commands(&pos, &gp);
                    self.actuators_command.publish(command);
                },
                _ => {
                    // No goal point, we stop every things.
                    self.actuators_command.publish(ActuatorCommandsMsg { rudder: 0.0, engine: 0.0 });
                }
            }
        }
    }
    async fn check_and_get_last_position(&self) -> Option<PositionMsg> {
        match self.last_position.lock().await.take() {
            None => None,
            Some(pos) => {
                if pos.is_valid {
                    debug!("Position is invalid in autopilot");
                    return None;
                    // On pourrait également vérifier l'age de la position...
                }else {
                    return Some(pos);
                }
            }
        }
    }
}
