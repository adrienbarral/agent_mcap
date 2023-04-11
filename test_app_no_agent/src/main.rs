use std::boxed::Box;
use std::sync::Arc;

mod messages;
mod gps_node;
mod autopilot;
mod plc_interface;

use crate::autopilot::Autopilot;
use crate::gps_node::GpsNode;
use crate::plc_interface::PLCInterface;

#[tokio::main]

async fn main() {
    let gps_node = Arc::<GpsNode>::new(GpsNode::new());
    let plc_interface = Arc::<PLCInterface>::new(PLCInterface::new("/dev/ttyS0".to_string()));
    let autopilot = Box::<Autopilot>::new(Autopilot::new(gps_node, plc_interface));

    tokio::signal::ctrl_c().await;
}
