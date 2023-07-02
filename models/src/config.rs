use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, Clone, PartialEq)]
pub struct AppConfig {
    pub simulation: SimulationConfig,
}

impl AppConfig {
    pub fn get_param(&self, name: &str) -> Option<String> {
        self.simulation
            .params
            .iter()
            .find(|x| x.name == name)
            .map(|x| x.value.clone())
    }
}

#[derive(Deserialize, Serialize, Default, Clone, PartialEq)]
pub struct SimulationConfig {
    pub simulation_class: String,
    pub params: Vec<Param>,
}

#[derive(Deserialize, Serialize, Default, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub value: String,
}
