
#[doc(hidden)]
use std::fmt;


#[doc(hidden)]
use serde::{Serialize, Deserialize};


#[derive(Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub enum PayerIdentificationType {
    PASS,
    CPFA,
    SRSSA,
    NRIN,
    OTHR,
    DRLC,
    SOCS,
    AREG,
    IDCD,
    EMID,
}


impl fmt::Display for PayerIdentificationType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PayerIdentificationType::PASS => write!(f, "PASS"),
            PayerIdentificationType::CPFA => write!(f, "CPFA"),
            PayerIdentificationType::SRSSA => write!(f, "SRSSA"),
            PayerIdentificationType::NRIN => write!(f, "NRIN"),
            PayerIdentificationType::OTHR => write!(f, "OTHR"),
            PayerIdentificationType::DRLC => write!(f, "DRLC"),
            PayerIdentificationType::SOCS => write!(f, "SOCS"),
            PayerIdentificationType::AREG => write!(f, "AREG"),
            PayerIdentificationType::IDCD => write!(f, "IDCD"),
            PayerIdentificationType::EMID => write!(f, "EMID"),
        }
    }
}