use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Money {
    pub amount: String,
    pub currency: String
}