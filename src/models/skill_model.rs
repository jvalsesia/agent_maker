use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SkillModel {
    pub id: String,
    pub name: String,
    pub description: String,
    pub instructions: String,
    pub created_ms: i64,
}
