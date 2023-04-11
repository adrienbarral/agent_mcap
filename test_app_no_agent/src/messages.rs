use schemars::JsonSchema;
use serde::Serialize;
use anyhow::Result;

#[async_trait::async_trait]
pub trait PositionProvider {
    async fn get_last_position(&self) -> Option<PositionMsg>;
}

#[async_trait::async_trait]
pub trait CommandSender {
    async fn send_command(&self, command: ActuatorCommandsMsg) -> Result<()>;
}

#[derive(Serialize, Clone, JsonSchema)]
pub struct PositionMsg{
    pub latitude: f64,
    pub longitude: f64,
    pub heading: f32,
    pub is_valid: bool,
}

#[derive(Serialize, Clone, JsonSchema)]
pub struct ActuatorCommandsMsg {
    pub rudder: f32,
    pub engine: f32
}

#[derive(Clone, JsonSchema, Serialize)]
pub enum State {
    IDLE, 
    REACHING,
    OUT_OF_ORDER(String)
}

#[derive(Serialize, Clone, JsonSchema)]
pub struct StateMsg {
    pub state: State
}

#[derive(Clone, JsonSchema, Serialize)]

pub enum EventType{
    TOO_FAR_OF_TRAJECTORY,
    NO_GPS_DATA_RECEIVED,
    GPS_FIX_LOST,
}

#[derive(Clone, JsonSchema, Serialize)]
pub struct Event{
    pub emitter: String,
    pub event: EventType
}