use std::{time::Duration};

use anyhow::Result;
use schemars::JsonSchema;
use serde::Serialize;
use agent_mcap::{Topic, Context, INode};

use tokio::time::sleep;

#[derive(Serialize, Clone, Copy, JsonSchema)]
struct PositionMsg{
    latitude: f64,
    longitude: f64,
    heading: f32,

}

struct GPSNode {
    name: String,
    gps_topic: Topic<PositionMsg>
}

#[async_trait::async_trait]
impl INode for GPSNode {
    async fn new(name: &str, context: &mut Context) -> Result<Self> {
        Ok(GPSNode { name: String::from(name),
        gps_topic: context.advertise("gps_pos").await?
        })
    }
    async fn task(&mut self){
        loop{
            sleep(Duration::from_millis(100)).await;
            let _ = self.gps_topic.tx.send(Some(
                PositionMsg{latitude: 43.000, longitude: 6.000, heading: 10.0}
            ));
        }
    }
    // TODO : Implémenter drop pour interrompre la tâche quand on détruit le Node...
    // Peut être pas un trait du coup : https://stackoverflow.com/questions/71541765/rust-async-drop/71741467#71741467
}

#[tokio::main]
async fn main() -> Result<()>{
    let mut context = Context::new()?;
    let mut gps_node = GPSNode::new("gps_node", &mut context).await?;

    tokio::select! {
        _ = gps_node.task() => {},
        _ = tokio::signal::ctrl_c() => {} 
    };

    Ok(())
}
