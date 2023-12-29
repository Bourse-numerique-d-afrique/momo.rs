//! Disbursements Product
//! 
//! 
//! 
//! 
//! 
//! 
//! 
//! 
//! 

use std::sync::Arc;

use crate::{
    enums::{environment::Environment, access_type::AccessType},
    requests::{refund::Refund, transfer::Transfer, bc_authorize::BcAuthorize, access_token::AccessTokenRequest},
    responses::{
        account_info::BasicUserInfoJsonResponse,
        bcauthorize_response::BCAuthorizeResponse, oauth2tokenresponse::OAuth2TokenResponse,
        token_response::TokenResponse, transfer_result::TransferResult, refund_result::RefundResult,
    },
    traits::{account::Account, auth::MOMOAuthorization}, structs::balance::Balance,
};

use chrono::Utc;
use once_cell::sync::Lazy;
use tokio::sync::Mutex;
use tokio::task;



pub struct Disbursements {
    pub url: String,
    pub primary_key: String,
    pub secondary_key: String,
    pub environment: Environment,
    pub api_user: String,
    pub api_key: String,
}

static ACCESS_TOKEN: Lazy<Arc<Mutex<Option<TokenResponse>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(None))
});


impl Disbursements {
    /*
       create a new instance of Disbursements product
       @param url
       @param environment
       @return Disbursements
    */
    pub fn new(url: String, environment: Environment, api_user: String, api_key: String, primary_key: String, secondary_key: String) -> Disbursements {
        Disbursements {
            url,
            primary_key,
            secondary_key,
            environment,
            api_key,
            api_user,
        }
    }


    /*
        This operation is used to get the latest access token from the database
        @return TokenResponse
     */
    async fn get_valid_access_token(&self) -> Result<TokenResponse, Box<dyn std::error::Error>> {
        let token = ACCESS_TOKEN.lock().await;
        if token.is_some() {
            let token = token.clone().unwrap();
            if token.created_at.is_some() {
                let created_at = token.created_at.unwrap();
                let expires_in = token.expires_in;
                let now = Utc::now();
                let duration = now.signed_duration_since(created_at);
                if duration.num_seconds() < expires_in as i64 {
                    return Ok(token);
                }
                let token: TokenResponse = self.create_access_token().await?;
                return Ok(token);
            }
        }
        let token: TokenResponse = self.create_access_token().await?;
        return Ok(token);
    }

    /*
       deposit operation is used to deposit an amount from the owner’s account to a payee account.
       Status of the transaction can be validated by using the GET /deposit/{referenceId}
       @return Ok(())
    */
    pub async fn deposit_v1(&self, transfer: Transfer) -> Result<String, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client
            .post(format!(
                "{}/disbursement/v1_0/deposit",
                self.url
            ))
            .bearer_auth(access_token.access_token)
            .header("Content-Type", "application/json")
            .header("X-Target-Environment", self.environment.to_string())
            .header("X-Reference-Id", &transfer.external_id)
            .header("Cache-Control", "no-cache")
            .header("Ocp-Apim-Subscription-Key", &self.primary_key)
            .body(transfer.clone())
            .send()
            .await?;

            if res.status().is_success() {
                Ok(transfer.external_id)
            }else {
                Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, res.text().await?)))
            }
    }

    /*
       deposit operation is used to deposit an amount from the owner’s account to a payee account.
       Status of the transaction can be validated by using the GET /deposit/{referenceId}
       @return Ok(())
    */
    pub async fn deposit_v2(&self, transfer: Transfer) -> Result<String, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client
            .post(format!(
                "{}/disbursement/v2_0/deposit",
                self.url
            ))
            .bearer_auth(access_token.access_token)
            .header("Content-Type", "application/json")
            .header("X-Target-Environment", self.environment.to_string())
            .header("X-Reference-Id", &transfer.external_id)
            .header("Cache-Control", "no-cache")
            .header("Ocp-Apim-Subscription-Key", &self.primary_key)
            .body(transfer.clone())
            .send()
            .await?;

        if res.status().is_success() {
            Ok(transfer.external_id)
        }else {
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, res.text().await?)))
        }
    }

    /*
       This operation is used to get the status of a deposit.
       X-Reference-Id that was passed in the post is used as reference to the request.
    */
    pub async fn get_deposit_status(&self, deposit_id: String) -> Result<TransferResult, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client
            .get(format!(
                "{}/disbursement/v1_0/deposit/{}",
                self.url, deposit_id
            ))
            .bearer_auth(access_token.access_token)
            .header("X-Target-Environment", self.environment.to_string())
            .header("Cache-Control", "no-cache")
            .header("Ocp-Apim-Subscription-Key", &self.primary_key)
            .send()
            .await?;

            if res.status().is_success() {
                let body = res.text().await?;
                let transfer_result: TransferResult = serde_json::from_str(&body)?;
                Ok(transfer_result)
            }else {
                Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, res.text().await?)))
            }
    }

    /*
       This operation is used to get the status of a refund.
       X-Reference-Id that was passed in the post is used as reference to the request.

       @return Ok(())
    */
    pub async fn get_refund_status(&self, reference_id: &str) -> Result<RefundResult, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client
            .get(format!(
                "{}/disbursement/v1_0/refund/{}",
                self.url, reference_id
            ))
            .bearer_auth(access_token.access_token)
            .header("X-Target-Environment", self.environment.to_string())
            .header("Cache-Control", "no-cache")
            .header("Ocp-Apim-Subscription-Key", &self.primary_key)
            .send()
            .await?;
        
        if res.status().is_success() {
            let body = res.text().await?;
            let refund_result: RefundResult = serde_json::from_str(&body)?;
            Ok(refund_result)
        }else {
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, res.text().await?)))
        }
    }

    /*
       This operation is used to get the status of a transfer.
       X-Reference-Id that was passed in the post is used as reference to the request.
       @return Ok(())
    */
    pub async fn get_transfer_status(&self, transfer_id: &str) -> Result<TransferResult, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client
            .get(format!(
                "{}/disbursement/v1_0/transfer/{}",
                self.url, transfer_id
            ))
            .bearer_auth(access_token.access_token)
            .header("X-Target-Environment", self.environment.to_string())
            .header("Cache-Control", "no-cache")
            .header("Ocp-Apim-Subscription-Key", &self.primary_key)
            .send()
            .await?;

        if res.status().is_success() {
            let body = res.text().await?;
            let transfer_result: TransferResult = serde_json::from_str(&body)?;
            Ok(transfer_result)
        }else {
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, res.text().await?)))
        }
    }

    /*
       refund operation is used to refund an amount from the owner’s account to a payee account.
       Status of the transaction can be validated by using the GET /refund/{referenceId}
       @return Ok(())
    */
    pub async fn refund_v1(&self, refund: Refund, _callback_url: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let refund_id = uuid::Uuid::new_v4().to_string();
        let access_token = self.get_valid_access_token().await?;
        let res = client
            .post(format!(
                "{}/disbursement/v1_0/refund",
                self.url
            ))
            .bearer_auth(access_token.access_token)
            .header("X-Reference-Id", &refund_id)
            .header("X-Target-Environment", self.environment.to_string())
            .header("Content-Type", "application/json")
            .header("Ocp-Apim-Subscription-Key", &self.primary_key)
            .body(refund)
            .send()
            .await?;

        if res.status().is_success() {
            Ok(refund_id)
        }else {
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, res.text().await?)))
        }
    }

    /*
       refund operation is used to refund an amount from the owner’s account to a payee account.
       Status of the transaction can be validated by using the GET /refund/{referenceId}
       @return Ok(())
    */
    pub async fn refund_v2(&self, refund: Refund, _callback_url: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let refund_id = uuid::Uuid::new_v4().to_string();
        let access_token = self.get_valid_access_token().await?;
        let res = client
            .post(format!(
                "{}/disbursement/v2_0/refund",
                self.url
            ))
            .bearer_auth(access_token.access_token)
            .header("X-Reference-Id", &refund_id)
            .header("X-Target-Environment", self.environment.to_string())
            .header("Content-Type", "application/json")
            .header("Ocp-Apim-Subscription-Key", &self.primary_key)
            .body(refund)
            .send()
            .await?;

        if res.status().is_success() {
            Ok(refund_id)
        }else {
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, res.text().await?)))
        }
    }

    /*
       transfer operation is used to transfer an amount from the owner’s account to a payee account.
       Status of the transaction can be validated by using the GET /transfer/{referenceId}
       @return Ok(())
    */
    pub async fn transfer(&self, transfer: Transfer) -> Result<String, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client
            .post(format!(
                "{}/disbursement/v1_0/transfer",
                self.url
            ))
            .bearer_auth(access_token.access_token)
            .header("X-Target-Environment", self.environment.to_string())
            .header("X-Reference-Id", &transfer.external_id)
            .header("Cache-Control", "no-cache")
            .header("Ocp-Apim-Subscription-Key", &self.primary_key)
            .body(transfer.clone())
            .send()
            .await?;

        if res.status().is_success() {
            Ok(transfer.external_id)
        }else {
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, res.text().await?)))
        }
    }
}

impl Account for Disbursements {
    async fn get_account_balance(&self) -> Result<Balance, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client
            .get(format!(
                "{}/disbursement/v1_0/account/balance",
                self.url
            ))
            .bearer_auth(access_token.access_token)
            .header("X-Target-Environment", self.environment.to_string())
            .header("Cache-Control", "no-cache")
            .header("Ocp-Apim-Subscription-Key", &self.primary_key)
            .send().await?;

            if res.status().is_success() {
                let body = res.text().await?;
                let balance: Balance = serde_json::from_str(&body)?;
                Ok(balance)
            }else {
                Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, res.text().await?)))
            }
    }

    async fn get_account_balance_in_specific_currency(
        &self,
        currency: String,
    ) -> Result<Balance, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client
            .get(format!(
                "{}/disbursement/v1_0/account/balance/{}",
                self.url,
                currency.to_lowercase()
            ))
            .bearer_auth(access_token.access_token)
            .header("X-Target-Environment", self.environment.to_string())
            .header("Cache-Control", "no-cache")
            .header("Ocp-Apim-Subscription-Key", &self.primary_key)
            .send().await?;
    
            if res.status().is_success() {
                let body = res.text().await?;
                let balance: Balance = serde_json::from_str(&body)?;
                Ok(balance)
            }else {
                Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, res.text().await?)))
            }
    }

    async fn get_basic_user_info(
        &self, account_holder_msisdn: &str
    ) -> Result<BasicUserInfoJsonResponse, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client
            .get(format!(
                "{}/disbursement/v1_0/accountholder/msisdn/{}/basicuserinfo",
                self.url,
                account_holder_msisdn
            ))
            .bearer_auth(access_token.access_token)
            .header("Content-Type", "application/json")
            .header("X-Target-Environment", self.environment.to_string())
            .header("Ocp-Apim-Subscription-Key", &self.primary_key)
            .header("Cache-Control", "no-cache").send().await?;
        

            if res.status().is_success() {
                let body = res.text().await?;
                let basic_user_info: BasicUserInfoJsonResponse = serde_json::from_str(&body)?;
                Ok(basic_user_info)
            }else {
                Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, res.text().await?)))
            }
    }

    async fn get_user_info_with_consent(
        &self,
        access_token: String
    ) -> Result<BasicUserInfoJsonResponse, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client
            .get(format!(
                "{}/disbursement/oauth2/v1_0/userinfo",
                self.url
            ))
            .bearer_auth(access_token)
            .header("Content-Type", "application/json")
            .header("X-Target-Environment", self.environment.to_string())
            .header("Ocp-Apim-Subscription-Key", &self.primary_key)
            .header("Cache-Control", "no-cache")
            .send().await?;
    
            if res.status().is_success() {
                let body = res.text().await?;
                let basic_user_info: BasicUserInfoJsonResponse = serde_json::from_str(&body)?;
                Ok(basic_user_info)
            }else {
                Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, res.text().await?)))
            }
    }

    async fn validate_account_holder_status(
        &self,
        account_holder_id: &str, account_holder_type: &str
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client
            .get(format!(
                "{}/disbursement/v1_0/accountholder/{}/{}/active",
                self.url,
                account_holder_type.to_lowercase(),
                account_holder_id.to_lowercase()
            ))
            .bearer_auth(access_token.access_token)
            .header("X-Target-Environment", self.environment.to_string())
            .header("Ocp-Apim-Subscription-Key", &self.primary_key)
            .send().await?;
    
            if res.status().is_success() {
                Ok(())
            }else {
                Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, res.text().await?)))
            }
    }
}

impl MOMOAuthorization for Disbursements {
    async fn create_access_token(&self) -> Result<TokenResponse, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client
            .post(format!("{}/disbursement/token/", self.url))
            .basic_auth(self.api_user.to_string(), Some(self.api_key.to_string()))
            .header("Cache-Control", "no-cache")
            .header("Content-type", "application/x-www-form-urlencoded")
            .header("Ocp-Apim-Subscription-Key", &self.primary_key)
            .header("Content-Length", "0")
            .body("")
            .send().await?;

        if res.status().is_success() {
            let body = res.text().await?;
            let token_response: TokenResponse = serde_json::from_str(&body)?;
            let cloned = token_response.clone();
            let _t = task::spawn(async move {
                let mut token = ACCESS_TOKEN.lock().await;
                *token = Some(token_response.clone());
            });
            Ok(cloned)
        } else {
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, res.text().await?)))
        }
    }

    async fn create_o_auth_2_token(
        &self,
        auth_req_id: String
    ) -> Result<OAuth2TokenResponse, Box<dyn std::error::Error>> {

        let client = reqwest::Client::new();
        let res = client
            .post(format!(
                "{}/disbursement/oauth2/token/",
                self.url
            ))
            .basic_auth(self.api_user.to_string(), Some(self.api_key.to_string()))
            .header("X-Target-Environment", self.environment.to_string())
            .header("Content-type", "application/x-www-form-urlencoded")
            .header("Ocp-Apim-Subscription-Key", &self.primary_key)
            .body(AccessTokenRequest{grant_type: "urn:openid:params:grant-type:ciba".to_string(), auth_req_id})
            .send()
            .await?;

            if res.status().is_success() {
                let body = res.text().await?;
                let token_response: OAuth2TokenResponse = serde_json::from_str(&body)?;
                Ok(token_response)
            }else {
                Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, res.text().await?)))
            }
    }

    async fn bc_authorize(&self, msisdn: String, _callback_url: Option<&str>) -> Result<BCAuthorizeResponse, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client
            .post(format!(
                "{}/disbursement/v1_0/bc-authorize",
                self.url
            ))
            .bearer_auth(access_token.access_token)
            .header("X-Target-Environment", "sandbox")
            .header("Content-type", "application/x-www-form-urlencoded")
            .header("Ocp-Apim-Subscription-Key", &self.primary_key)
            .body(BcAuthorize{login_hint: format!("ID:{}/MSISDN", msisdn), scope: "profile".to_string(), access_type: AccessType::Offline}.to_string()) // scope can be profile
            .send()
            .await?;

            if res.status().is_success() {
                let body = res.text().await?;
                let token_response: BCAuthorizeResponse = serde_json::from_str(&body)?;
                Ok(token_response)
            }else {
                Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, res.text().await?)))
            }
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use dotenv::dotenv;
    use crate::{
        enums::{environment::Environment, currency::Currency, party_id_type::PartyIdType},
        products::{disbursements::Disbursements, collection::Collection},
        traits::{account::Account, auth::MOMOAuthorization}, requests::{transfer::Transfer, refund::Refund, request_to_pay::RequestToPay}, structs::party::Party,
    };

    #[tokio::test]
    async fn test_get_account_balance() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_DISBURSEMENT_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_DISBURSEMENT_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");

        let disbursements = Disbursements::new(
            mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key );
        let balance_result = disbursements.get_account_balance().await;
        if balance_result.is_ok() {
            let balance = balance_result.unwrap();
            assert_eq!(balance.currency, "EUR");
        }
    }

    #[tokio::test]
    async fn test_get_account_balance_in_specific_currency() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_DISBURSEMENT_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_DISBURSEMENT_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");

        let disbursements = Disbursements::new(
            mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key
        );
        let balance_result = disbursements.get_account_balance_in_specific_currency("EUR".to_string()).await;
        if balance_result.is_ok() {
            let balance = balance_result.unwrap();
            assert_eq!(balance.currency, "EUR");
        }
    }

    #[tokio::test] 
    async fn test_get_basic_user_info() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_DISBURSEMENT_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_DISBURSEMENT_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");
        let disbursements = Disbursements::new(
            mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key
        );
        let basic_user_info = disbursements.get_basic_user_info("256774290781").await.unwrap();
        assert_ne!(basic_user_info.given_name.len(), 0);
    }


    #[tokio::test]
    async fn test_validate_account_holder_status() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_DISBURSEMENT_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_DISBURSEMENT_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");
        let disbursements = Disbursements::new(
            mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key
        );
        let validate_account_holder_status_result = disbursements.validate_account_holder_status("256774290781", "MSISDN").await;
        assert!(validate_account_holder_status_result.is_ok());
    }


    #[tokio::test]
    async fn test_bc_authorize() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_DISBURSEMENT_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_DISBURSEMENT_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");
        let disbursements = Disbursements::new(
            mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key
        );
        let bc_authorize_res = disbursements.bc_authorize("563607".to_string(), None).await;
        assert!(bc_authorize_res.is_ok());
        assert_ne!(bc_authorize_res.unwrap().auth_req_id.len(), 0);
    }


    #[tokio::test]
    async fn test_create_o_auth_2_token() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_DISBURSEMENT_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_DISBURSEMENT_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");
        let disbursements = Disbursements::new(
            mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key
        );

        let bc_authorize_res = disbursements.bc_authorize("563607".to_string(), None).await;
        assert!(bc_authorize_res.is_ok());

        let res = disbursements.create_o_auth_2_token(bc_authorize_res.unwrap().auth_req_id).await.expect("Error creating o auth 2 token");
        assert_ne!(res.access_token.len(), 0);
    }

    #[tokio::test]
    async fn test_get_user_info_with_consent() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_DISBURSEMENT_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_DISBURSEMENT_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");
        let disbursements = Disbursements::new(
            mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key
        );
        let bc_authorize_res = disbursements.bc_authorize("563607".to_string(), None).await;
        assert!(bc_authorize_res.is_ok());

        let res = disbursements.create_o_auth_2_token(bc_authorize_res.unwrap().auth_req_id).await.expect("Error creating o auth 2 token");
        assert_ne!(res.access_token.len(), 0);
        let user_info_with_consent = disbursements.get_user_info_with_consent(res.access_token).await.unwrap();
        assert_ne!(user_info_with_consent.family_name.len(), 0);
    }




    #[tokio::test]
    async fn test_deposit_v1() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_DISBURSEMENT_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_DISBURSEMENT_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");
        let disbursements = Disbursements::new(
            mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key
        );


        let payee = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "256774290781".to_string(),
        };
        let transfer = Transfer::new("100".to_string(), "EUR".to_string(), payee, "payer_message".to_string(), "payee_note".to_string());
        let result = disbursements.deposit_v1(transfer.clone()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), transfer.external_id);
    }

    #[tokio::test]
    async fn test_deposit_v2() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_DISBURSEMENT_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_DISBURSEMENT_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");
        let disbursements = Disbursements::new(
            mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key);

            let payee = Party {
                party_id_type: PartyIdType::MSISDN,
                party_id: "256774290781".to_string(),
            };
            let transfer = Transfer::new("100".to_string(), "EUR".to_string(), payee, "payer_message".to_string(), "payee_note".to_string());
            let result = disbursements.deposit_v1(transfer.clone()).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), transfer.external_id);
    }

    #[tokio::test]
    async fn test_get_deposit_status() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_DISBURSEMENT_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_DISBURSEMENT_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");
        let disbursements = Disbursements::new(
            mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key
        );

        let payee = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "256774290781".to_string(),
        };
        let transfer = Transfer::new("100".to_string(), "EUR".to_string(), payee, "payer_message".to_string(), "payee_note".to_string());
        let result = disbursements.deposit_v1(transfer.clone()).await;
        assert!(result.is_ok());
        let status_result = disbursements.get_deposit_status(result.unwrap()).await;
        assert!(status_result.is_ok());
    }


    #[tokio::test]
    async fn test_refund_v1() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_DISBURSEMENT_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_DISBURSEMENT_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");
        let disbursements = Disbursements::new(
            mtn_url.clone(), Environment::Sandbox, api_user.clone(), api_key.clone(), primary_key, secondary_key
        );

        let collection_primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let collection_secondary_key = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");

        let collection = Collection::new(
            mtn_url, Environment::Sandbox, api_user, api_key, collection_primary_key, collection_secondary_key
        );

        let payer : Party = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "+242064818006".to_string(),
        };
        let request = RequestToPay::new("100".to_string(), Currency::EUR, payer, "test_payer_message".to_string(), "test_payee_note".to_string());
        let res = collection.request_to_pay(request).await;
        assert!(res.is_ok());

        
        let refund = Refund::new("100".to_string(), Currency::EUR.to_string(), "payer_message".to_string(), "payee_note".to_string(), res.unwrap());
        let refund_res = disbursements.refund_v1(refund, None).await;
        assert!(refund_res.is_ok());
        assert_ne!(refund_res.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_refund_v2() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_DISBURSEMENT_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_DISBURSEMENT_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");
        let disbursements = Disbursements::new(
            mtn_url.clone(), Environment::Sandbox, api_user.clone(), api_key.clone(), primary_key, secondary_key
        );

        let collection_primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let collection_secondary_key = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");

        let collection = Collection::new(
            mtn_url, Environment::Sandbox, api_user, api_key, collection_primary_key, collection_secondary_key
        );

        let payer : Party = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "+242064818006".to_string(),
        };
        let request = RequestToPay::new("100".to_string(), Currency::EUR, payer, "test_payer_message".to_string(), "test_payee_note".to_string());
        let res = collection.request_to_pay(request).await;
        assert!(res.is_ok());

        
        let refund = Refund::new("100".to_string(), Currency::EUR.to_string(), "payer_message".to_string(), "payee_note".to_string(), res.unwrap());
        let refund_res = disbursements.refund_v2(refund, None).await;
        assert!(refund_res.is_ok());
        assert_ne!(refund_res.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_get_refund_status() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_DISBURSEMENT_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_DISBURSEMENT_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");
        let disbursements = Disbursements::new(
            mtn_url.clone(), Environment::Sandbox, api_user.clone(), api_key.clone(), primary_key, secondary_key
        );
        let collection_primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let collection_secondary_key = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");

        let collection = Collection::new(
            mtn_url, Environment::Sandbox, api_user, api_key, collection_primary_key, collection_secondary_key
        );

        let payer : Party = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "+242064818006".to_string(),
        };
        let request = RequestToPay::new("100".to_string(), Currency::EUR, payer, "test_payer_message".to_string(), "test_payee_note".to_string());
        let res = collection.request_to_pay(request).await;
        assert!(res.is_ok());

        
        let refund = Refund::new("100".to_string(), Currency::EUR.to_string(), "payer_message".to_string(), "payee_note".to_string(), res.unwrap());
        let refund_res = disbursements.refund_v2(refund, None).await;
        assert!(refund_res.is_ok());
        let refund_status_res = disbursements.get_refund_status(&refund_res.unwrap()).await.unwrap();
        assert_ne!(refund_status_res.status.len(), 0);
    }

    #[tokio::test]
    async fn test_transfer() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_DISBURSEMENT_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_DISBURSEMENT_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");
        let disbursements = Disbursements::new(
            mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key
        );

        let transfer = Transfer::new("100".to_string(), "EUR".to_string(), Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "256774290781".to_string(),
        }, "payer_message".to_string(), "payee_note".to_string());
        let transfer_result = disbursements.transfer(transfer.clone()).await;
        assert!(transfer_result.is_ok());
        assert_eq!(transfer_result.unwrap(), transfer.external_id);
    }

    #[tokio::test]
    async fn test_get_transfer_status() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_DISBURSEMENT_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_DISBURSEMENT_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");
        let disbursements = Disbursements::new(
            mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key
        );



        let transfer = Transfer::new("100".to_string(), "EUR".to_string(), Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "256774290781".to_string(),
        }, "payer_message".to_string(), "payee_note".to_string());
        let transfer_result = disbursements.transfer(transfer.clone()).await;
        assert!(transfer_result.is_ok());

        let status_result = disbursements.get_transfer_status(&transfer_result.unwrap()).await;
        assert!(status_result.is_ok());

    }

  

}
