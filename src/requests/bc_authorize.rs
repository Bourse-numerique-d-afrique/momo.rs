use reqwest::Body;
use crate::enums::access_type::AccessType;
use serde::{Serialize, Deserialize};



#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BcAuthorize {
    pub login_hint: String,
    pub scope: String,
    pub access_type: AccessType
}


impl From<BcAuthorize> for Body {
    fn from(bc_authorize: BcAuthorize) -> Self {
        Body::from(serde_json::to_string(&bc_authorize).unwrap())
    }
}