use crate::{traits::{account::Account, auth::MOMOAuthorization}, responses::{token_response::TokenResponse, bcauthorize_response::BCAuthorizeResponse, oauth2tokenresponse::OAuth2TokenResponse, account_info_consent::UserInfoWithConsent, account_info::BasicUserInfoJsonResponse}, enums::environment::Environment};
use base64::{engine::general_purpose, Engine as _};
use crate::structs::balance::Balance;



pub struct Remittance{
    pub url: String,
    pub primary_key: String,
    pub secondary_key: String,
    pub environment: Environment,
    pub api_user: String,
    pub api_key: String,
}

impl Remittance {
    /*
        create a new instance of Remittance product
        @param url
        @param environment
        @return Remittance
    
     */
    pub fn new(url: String, environment: Environment, api_user: String, api_key: String, primary_key: String, secondary_key: String) -> Remittance {
        Remittance{
            url,
            primary_key,
            secondary_key,
            environment,
            api_user,
            api_key,
        }
    }


    /*
        Cash transfer operation is used to transfer an amount from the owner’s account to a payee account.
        Status of the transaction can be validated by using GET /cashtransfer/{referenceId}
        @return Ok(())
     */
    pub async fn cash_transfer(&self) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client.post(format!("{}/remittance/v1_0/preapproval", self.url))
        .header("Authorization", format!("Basic {}", ""))
        .header("X-Target-Environment", self.environment.to_string())
        .header("Cache-Control", "no-cache")
        .send().await?;

        
        let response = res.text().await?;
        Ok(())
    }

    /*
        This operation is used to get the status of a transfer.
        X-Reference-Id that was passed in the post is used as reference to the request.
     */
    pub async fn get_cash_transfer_status(&self) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client.post(format!("{}/remittance/v1_0/preapproval", self.url))
        .header("Authorization", format!("Basic {}", ""))
        .header("X-Target-Environment", self.environment.to_string())
        .header("Cache-Control", "no-cache")
        .send().await?;

        
        let response = res.text().await?;
        Ok(())
    }


    /*
        This operation is used to get the status of a transfer.
        X-Reference-Id that was passed in the post is used as reference to the request.
     */
    pub async fn get_transfer_status(&self) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client.post(format!("{}/remittance/v1_0/preapproval", self.url))
        .header("Authorization", format!("Basic {}", ""))
        .header("X-Target-Environment", self.environment.to_string())
        .header("Cache-Control", "no-cache")
        .send().await?;

        
        let response = res.text().await?;
        Ok(())
    }

    /*
        Transfer operation is used to transfer an amount from the own account to a payee account.
        Status of the transaction can validated by using the GET /transfer/{referenceId}
     */
    pub async fn transfer(&self) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client.post(format!("{}/remittance/v1_0/preapproval", self.url))
        .header("Authorization", format!("Basic {}", ""))
        .header("X-Target-Environment", self.environment.to_string())
        .header("Cache-Control", "no-cache")
        .send().await?;

        
        let response = res.text().await?;
        Ok(())
    }

    
}

impl Account for Remittance {
    async fn get_account_balance(&self) -> Result<Balance, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client.post(format!("{}/remittance/v1_0/preapproval", self.url))
        .header("Authorization", format!("Basic {}", ""))
        .header("X-Target-Environment", self.environment.to_string())
        .header("Cache-Control", "no-cache")
        .send().await?;

        
        let response = res.text().await?;
        let balance: Balance = serde_json::from_str(&response)?;
        Ok(balance)
    }

    async fn get_account_balance_in_specific_currency(&self, currency: String) -> Result<Balance, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client.post(format!("{}/remittance/v1_0/preapproval", self.url))
        .header("Authorization", format!("Basic {}", ""))
        .header("X-Target-Environment", self.environment.to_string())
        .header("Cache-Control", "no-cache")
        .send().await?;

        
        let response = res.text().await?;
        let balance: Balance = serde_json::from_str(&response)?;
        Ok(balance)
    }

    async fn get_basic_user_info(&self) -> Result<BasicUserInfoJsonResponse, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client.post(format!("{}/remittance/v1_0/preapproval", self.url))
        .header("Authorization", format!("Basic {}", ""))
        .header("X-Target-Environment", self.environment.to_string())
        .header("Cache-Control", "no-cache")
        .send().await?;

        
        let response = res.text().await?;
        let basic_user_info: BasicUserInfoJsonResponse = serde_json::from_str(&response)?;
        Ok(basic_user_info)
    }

    async fn get_user_info_with_consent(&self) -> Result<UserInfoWithConsent, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client.post(format!("{}/remittance/v1_0/preapproval", self.url))
        .header("Authorization", format!("Basic {}", ""))
        .header("X-Target-Environment", self.environment.to_string())
        .header("Cache-Control", "no-cache")
        .send().await?;

        
        let response = res.text().await?;
        let user_info_with_consent: UserInfoWithConsent = serde_json::from_str(&response)?;
        Ok(user_info_with_consent)
    }

    async fn validate_account_holder_status(&self, account_holder_id: String) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client.post(format!("{}/remittance/v1_0/preapproval", self.url))
        .header("Authorization", format!("Basic {}", ""))
        .header("X-Target-Environment", self.environment.to_string())
        .header("Cache-Control", "no-cache")
        .send().await?;

        
        let response = res.text().await?;
        Ok(())
    }
}

impl MOMOAuthorization for Remittance {
    async fn create_access_token(&self) -> Result<TokenResponse, Box<dyn std::error::Error>> {
        let authorization = self.encode(&self.primary_key, &self.secondary_key);
        let client = reqwest::Client::new();
        let res = client.post(format!("{}/remittance/token/", self.url))
        .header("Authorization", format!("Basic {}", authorization))
        .header("Cache-Control", "no-cache")
        .send().await?;

        let body = res.text().await?;
        let token_response: TokenResponse = serde_json::from_str(&body)?;
        Ok(token_response)
    }

    async fn create_o_auth_2_token(&self) -> Result<OAuth2TokenResponse, Box<dyn std::error::Error>> {
        let authorization = self.encode(&self.primary_key, &self.secondary_key);
        let client = reqwest::Client::new();
        let res = client.post(format!("{}/remittance/oauth2/token/", self.url))
        .header("Authorization", format!("Basic {}", authorization))
        .header("X-Target-Environment", self.environment.to_string())
        .header("Content-type", "application/x-www-form-urlencoded")
        .header("Cache-Control", "no-cache")
        .send().await?;

        let body = res.text().await?;
        let token_response: OAuth2TokenResponse = serde_json::from_str(&body)?;
        Ok(token_response)
    }

    async fn bc_authorize(&self) -> Result<BCAuthorizeResponse, Box<dyn std::error::Error>> {
        let authorization = self.encode(&self.primary_key, &self.secondary_key);
        let client = reqwest::Client::new();
        let res = client.post(format!("{}/remittance/v1_0/bc-authorize", self.url))
        .header("Authorization", format!("Basic {}", authorization))
        .header("X-Target-Environment", self.environment.to_string())
        .header("X-Callback-Url", "callback")
        .header("Content-type", "application/x-www-form-urlencoded")
        .header("Cache-Control", "no-cache")
        .send().await?;

        let body = res.text().await?;
        let token_response: BCAuthorizeResponse = serde_json::from_str(&body)?;
        Ok(token_response)
    }

    fn encode(&self, user_id: &str, user_api_key: &str) -> String {
        let concatenated_str = format!("{}:{}", user_id, user_api_key);
        let encoded_str = general_purpose::STANDARD.encode(&concatenated_str);
        encoded_str
    }
}