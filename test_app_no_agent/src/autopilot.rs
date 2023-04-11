use std::sync::Arc;
use anyhow::Result;
use log::{log, error};
use tokio::{task::JoinHandle, sync::Mutex};

use crate::messages::{PositionProvider, CommandSender, PositionMsg, ActuatorCommandsMsg};

pub struct Autopilot {
    position_provider: Arc<dyn PositionProvider + Send + Sync>,
    goal_position: Arc<Mutex<Option<PositionMsg>>>,
    task: JoinHandle<()>
}

impl Autopilot{
    pub fn new(position_provider: Arc<dyn PositionProvider + Send + Sync>,
        control_sender: Arc<dyn CommandSender + Send + Sync>) -> Self {
        let goal_position: Arc<Mutex<Option<PositionMsg>>> = Arc::new(Mutex::new(None)); 
        Autopilot {
            position_provider: position_provider.clone(),
            goal_position: goal_position.clone(),
            task: tokio::spawn(async move {
                compute_control(goal_position.clone(), position_provider.clone(), control_sender.clone()).await
            })
        }
    }
}

impl Drop for Autopilot {
    fn drop(&mut self) {
        self.task.abort();
    }
}

async fn compute_control(goal_position: Arc<Mutex<Option<PositionMsg>>>,
        position_provider: Arc<dyn PositionProvider + Send + Sync>,
        control_sender: Arc<dyn CommandSender + Send + Sync>) {
    loop{
        // TODO : Ici en cas d'erreur d'envoi de commande on aimerait peut être remonter une info à la machine
        // d'état qui gère la sécu.
        match goal_position.lock().await.take() {
            None => {
                control_sender.send_command(ActuatorCommandsMsg{engine: 0f32, rudder: 0f32}).await.
                    unwrap_or_else(|err|{error!("Can't send command : {:} !!!", err);});
            },
            Some(_goal_position) => {
                match position_provider.get_last_position().await {
                    Some(_current_pos) => {
                        // Faire quelque chose de current_pos et goal_pos ...
                        control_sender.send_command(ActuatorCommandsMsg{engine: 1f32, rudder: 0f32}).await.
                        unwrap_or_else(|err|{error!("Can't send command : {:} !!!", err);});
                    },
                    None => {
                        control_sender.send_command(ActuatorCommandsMsg{engine: 0f32, rudder: 0f32}).await.
                        unwrap_or_else(|err|{error!("Can't send command : {:} !!!", err);});
                    }
                }
            }
        }
    }
}
