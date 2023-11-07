use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Pizza {
    pub name: String,
    pub toppings: Vec<String>,
}
