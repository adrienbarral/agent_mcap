mod messages;
mod autopilot;
mod gps_node;
mod general_state_controller;

use gps_node::GPSNode;
use general_state_controller::GeneralStateController;
use anyhow::Result;
use agent_mcap::{Topic, Context};


// TODO créer un noeud qui va "écouter" sur de l'UDP pour faire l'interface avec un système tiers (genre IHM). C'est avec lui qu'on donne les ordres de changement
// d'états... A voir si on ne met pas cette fonction dans le General State Controller...
#[tokio::main]
async fn main() -> Result<()>{
    
    let mut context = Context::new()?;
    let mut state_controller = GeneralStateController::new("state_controller", &mut context).await?;
    let mut gps_node = GPSNode::new("gps_node", &mut context, &state_controller.event).await?;

    tokio::select! {
        _ = gps_node.task() => {},
        _ = tokio::signal::ctrl_c() => {} 
    };

    Ok(())
}
