use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct Money {
    pub amount: String,
    pub currency: String
}