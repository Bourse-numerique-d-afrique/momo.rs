#[doc(hidden)]
use std::fmt;

#[doc(hidden)]
use serde::{Serialize, Deserialize};


#[derive(Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub enum Environment {
    /*
        The following environments are available
    */
    Sandbox,
    MTNUGANDA,
    MTNIVORYCOAST,
    MTNGHANA,
    MTNZAMBIA,
    MTNCAMEROON,
    MTNBENIN,
    MTNCONGO,
    MTNLIBERIA,
    MTNSWAZILAND,
    MTNGUINEACONAKRY,
    MTNSOUTHAFRICA,
    Live,
}


impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Environment::Sandbox => write!(f, "sandbox"),
            Environment::MTNUGANDA => write!(f, "mtnuganda"),
            Environment::MTNIVORYCOAST => write!(f, "mtnivorycoast"),
            Environment::MTNGHANA => write!(f, "mtnghana"),
            Environment::MTNZAMBIA => write!(f, "mtnzambia"),
            Environment::MTNCAMEROON => write!(f, "mtncameroon"),
            Environment::MTNBENIN => write!(f, "mtnbenin"),
            Environment::MTNCONGO => write!(f, "mtncongo"),
            Environment::MTNLIBERIA => write!(f, "mtnliberia"),
            Environment::MTNSWAZILAND => write!(f, "mtnswaziland"),
            Environment::MTNGUINEACONAKRY => write!(f, "mtnguineaconakry"),
            Environment::MTNSOUTHAFRICA => write!(f, "mtnsouthafrica"),
            Environment::Live => write!(f, "live"),
        }
    }
}