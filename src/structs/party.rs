#[doc(hidden)]
use serde::{Deserialize, Serialize};

use crate::enums::party_id_type::PartyIdType;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Party {
    /// Party identifies a account holder in the wallet platform. Party consists of two parameters, type and partyId. Each type have its own validation of the partyId
    /// MSISDN - Mobile Number validated according to ITU-T E.164. Validated with IsMSISDN
    /// EMAIL - Validated to be a valid e-mail format. Validated with IsEmail
    /// PARTY_CODE - UUID of the party. Validated with IsUuid
    #[serde(rename = "partyIdType")]
    pub party_id_type: PartyIdType,
    #[serde(rename = "partyId")]
    pub party_id: String,
}
