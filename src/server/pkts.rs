use serde::{Deserialize, Serialize};

pub const LOGIN: u16 = 0;
#[derive(Serialize, Deserialize)]
pub struct Login {
    pub name: String,
    pub pwd: String,
}
