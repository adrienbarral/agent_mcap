use schemars::JsonSchema;
use serde::Serialize;

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