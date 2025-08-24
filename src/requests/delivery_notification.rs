#[doc(hidden)]
use reqwest::Body;

#[doc(hidden)]
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DeliveryNotification {
    #[serde(rename = "notificationMessage")]
    pub notification_message: String,
}

impl From<DeliveryNotification> for Body {
    fn from(delivery_notification: DeliveryNotification) -> Self {
        Body::from(serde_json::to_string(&delivery_notification).unwrap())
    }
}
