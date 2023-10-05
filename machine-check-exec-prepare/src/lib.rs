use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Preparation {
    pub target_build_args: Vec<String>,
}
