#[doc(hidden)]
use std::fmt;

#[doc(hidden)]
use crate::enums::access_type::AccessType;

#[doc(hidden)]
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BcAuthorize {
    pub scope: String,
    #[serde(rename = "login_hint")]
    pub login_hint: String,
    #[serde(rename = "access_type")]
    pub access_type: AccessType,
}

impl fmt::Display for BcAuthorize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "scope={}&login_hint={}&access_type={}",
            self.scope,
            self.login_hint,
            match self.access_type {
                AccessType::Offline => "offline",
                AccessType::Online => "online",
            }
        )
    }
}
