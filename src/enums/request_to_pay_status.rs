#[doc(hidden)]
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub enum RequestToPayStatus {
    SUCCESSFULL,
    FAILED,
}
