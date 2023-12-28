use std::fmt;



use serde::{Serialize, Deserialize};


#[derive(Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub enum PartyIdType {
    MSISDN,
    EMAIL,
    PARTYCODE,
}



impl fmt::Display for PartyIdType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PartyIdType::MSISDN => write!(f, "MSISDN"),
            PartyIdType::EMAIL => write!(f, "EMAIL"),
            PartyIdType::PARTYCODE => write!(f, "PARTY_CODE"),
        }
    }
}