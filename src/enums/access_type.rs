#[doc(hidden)]
use std::fmt;

#[doc(hidden)]
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub enum AccessType {
    #[serde(rename = "online")]
    Online,
    #[serde(rename = "offline")]
    Offline,
}

impl fmt::Display for AccessType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AccessType::Online => write!(f, "online"),
            AccessType::Offline => write!(f, "offline"),
        }
    }
}
