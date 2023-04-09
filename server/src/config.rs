use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default)]
pub(crate) struct AppConfig {
    pub simulation : SimulationConfig
}

#[derive(Deserialize, Serialize, Default)]
pub(crate) struct SimulationConfig {
    pub simulation_class : String,
    pub params : Vec<Param>
}

#[derive(Deserialize, Serialize, Default)]
pub(crate) struct Param {
    pub name : String,
    pub value : String
}