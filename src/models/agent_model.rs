use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct AgentModel {
    pub id: String,
    pub name: String,
    pub preamble: String,
    pub prompt: String,
    pub response: String,
}
