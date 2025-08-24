#[doc(hidden)]
use serde::{Deserialize, Serialize};

use crate::enums::currency::Currency;

#[derive(Debug, Serialize, Deserialize)]
pub struct Balance {
    #[serde(rename = "availableBalance")] // The available balance of the account
    pub available_balance: String, // The available balance of the account
    pub currency: Currency, // ISO4217 Currency
}
