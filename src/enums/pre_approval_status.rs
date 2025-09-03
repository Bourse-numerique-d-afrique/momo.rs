use serde::{Deserialize, Serialize};

/// This is the status of the pre-approval.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum PreApprovalStatus {
    /// The pre-approval is pending.
    PENDING,
    /// The pre-approval is successful.
    SUCCESSFUL,
    /// The pre-approval has failed.
    FAILED,
    /// The pre-approval has been created.
    CREATED,
}