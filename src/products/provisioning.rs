use crate::{responses::api_user_key::ApiUserKeyResult, requests::provisioning::ProvisioningRequest};




pub struct Provisioning {
    pub primary_key: String,
    pub url: String,
}

impl Provisioning {
    pub fn new(url: String) -> Self {
        dotenv::dotenv().ok();
        let primary_key = std::env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        Provisioning {
            primary_key,
            url
        }
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    /*
        Used to create an API user in the sandbox target environment.
     */
    pub async fn create_sandox(&self, reference_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let provisioning = ProvisioningRequest{
            provider_callback_host: "string".to_string()
        };

        println!("provisioning: {:?}",serde_json::to_string(&provisioning)?);
        let res = client.post(format!("{}v1_0/apiuser", self.url))
        .header("X-Reference-Id", reference_id)
        .header("Content-Type", "application/json")
        .header("Ocp-Apim-Subscription-Key", &self.primary_key)
        .body(serde_json::to_string(&provisioning)?)
        .send().await?;

        let response = res.text().await?;
        println!("response create_sandbox: {:?}", response);
        Ok(())
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    /*
        Used to get API user information.
     */
    pub async fn get_api_information(&self, reference_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client.post(format!("{}/v1_0/apiuser/{}", self.url, reference_id))
        .header("Cache-Control", "no-cache")
        .header("Content-Length", "0")
        .header("Ocp-Apim-Subscription-Key", &self.primary_key)
        .body("")
        .send().await?;

        
        let response = res.text().await?;
        println!("response get_api_information: {:?}", response);
        Ok(())
    }


    /*
        Used to create an API key for an API user in the sandbox target environment.
     */
    pub async fn create_api_information(&self, reference_id: &str) -> Result<ApiUserKeyResult, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client.post(format!("{}/v1_0/apiuser/{}/apikey",self.url, reference_id))
        .header("Cache-Control", "no-cache")
        .header("Ocp-Apim-Subscription-Key", &self.primary_key)
        .header("Content-Length", "0")
        .body("")
        .send().await?;

        let response = res.text().await?;
        let api_key: ApiUserKeyResult = serde_json::from_str(&response)?;
        println!("response create_api_information: {:?} {:?}", response, reference_id);
        Ok(api_key)
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
        let provisioning = Provisioning::new(mtn_url);
        let reference_id = Uuid::new_v4().to_string();
        let result = provisioning.create_sandox(&reference_id).await;
        assert_eq!(result.is_ok(), true);
        let resullt = provisioning.get_api_information(&reference_id).await;
        assert_eq!(resullt.is_ok(), true);
        let result = provisioning.create_api_information(&reference_id).await;
        assert_eq!(result.unwrap().api_key.len() > 0, true);
    }


}