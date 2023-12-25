use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Balance {
    #[serde(rename = "availableBalance")] // The available balance of the account
    available_balance: String, // The available balance of the account
    currency: String, // ISO4217 Currency
}