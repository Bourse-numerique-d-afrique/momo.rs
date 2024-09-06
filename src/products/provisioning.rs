//! Provisioning for sandbox
//!
//!
//!
//!
//!
//!
//!
//!
//!

use crate::{
    requests::provisioning::ProvisioningRequest, responses::api_user_key::ApiUserKeyResult,
};

pub struct Provisioning {
    pub subscription_key: String,
    pub url: String,
}

impl Provisioning {
    pub fn new(url: String, subscription_key: String) -> Self {
        Provisioning {
            subscription_key,
            url,
        }
    }

    /// Used to create an API user in the sandbox target environment
    ///
    /// # Parameters
    ///
    /// * 'reference_id', reference identification number
    /// * 'provider_callback_host',
    ///
    /// # Returns
    ///
    /// * '()'
    pub async fn create_sandox(
        &self,
        reference_id: &str,
        provider_callback_host: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let provisioning = ProvisioningRequest {
            provider_callback_host: provider_callback_host.to_string(),
        };

        let res = client
            .post(format!("{}/v1_0/apiuser", self.url))
            .header("X-Reference-Id", reference_id)
            .header("Content-Type", "application/json")
            .header("Cache-Control", "no-cache")
            .header("Ocp-Apim-Subscription-Key", &self.subscription_key)
            .body(provisioning)
            .send()
            .await?;

        if res.status().is_success() {
            return Ok(());
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                res.text().await?,
            )))
        }
    }

    /// Used to get API user information.
    ///
    ///
    /// # Parameters
    ///
    /// * 'reference_id', reference identification number
    ///
    ///
    /// # Returns
    ///
    /// * '()'
    #[allow(dead_code)]
    pub async fn get_api_information(
        &self,
        reference_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client
            .get(format!("{}/v1_0/apiuser/{}", self.url, reference_id))
            .header("Cache-Control", "no-cache")
            .header("Ocp-Apim-Subscription-Key", &self.subscription_key)
            .send()
            .await?;

        if res.status().is_success() {
            return Ok(());
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                res.text().await?,
            )))
        }
    }

    /// Used to create an API key for an API user in the sandbox target environment.
    ///
    /// # Parameters
    ///
    /// * 'reference_id', reference identification number
    ///
    ///
    /// # Returns
    ///
    /// * 'ApiUserKeyResult'
    pub async fn create_api_information(
        &self,
        reference_id: &str,
    ) -> Result<ApiUserKeyResult, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client
            .post(format!("{}/v1_0/apiuser/{}/apikey", self.url, reference_id))
            .header("Cache-Control", "no-cache")
            .header("Ocp-Apim-Subscription-Key", &self.subscription_key)
            .header("Content-Length", "0")
            .body("")
            .send()
            .await?;

        if res.status().is_success() {
            let response = res.text().await?;
            let api_key: ApiUserKeyResult = serde_json::from_str(&response)?;
            Ok(api_key)
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                res.text().await?,
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;
    use std::env;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_0() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");
        let subscription_key =
            std::env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let provisioning = Provisioning::new(mtn_url, subscription_key);
        let reference_id = Uuid::new_v4().to_string();
        let result = provisioning.create_sandox(&reference_id, "test").await;
        assert_eq!(result.is_ok(), true);
        let resullt = provisioning.get_api_information(&reference_id).await;
        assert_eq!(resullt.is_ok(), true);
        let result = provisioning.create_api_information(&reference_id).await;
        let api_key = result.unwrap();
        assert_eq!(api_key.clone().api_key.len() > 0, true);

        println!("{:?}", reference_id);
        println!("{:?}", api_key.clone());
    }
}
