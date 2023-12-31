//! Remittance Product
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

use crate::{traits::{account::Account, auth::MOMOAuthorization},
 TranserId, Currency, Environment, TokenResponse, CashTransferRequest, TransferRequest, Balance, BasicUserInfoJsonResponse, OAuth2TokenResponse, AccessTokenRequest, BCAuthorizeResponse, BcAuthorizeRequest, AccessType, CashTransferResult, TransferResult};
use chrono::Utc;
use once_cell::sync::Lazy;
use tokio::sync::Mutex;
use tokio::task;



pub struct Remittance{
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
        Cash transfer operation is used to transfer an amount from the owner’s account to a payee account.
        Status of the transaction can be validated by using GET /cashtransfer/{referenceId}
        @param transfer
        @param callback_url, optional, the url to be called when the transaction is completed
        @return Ok(())
     */
    pub async fn cash_transfer(&self, transfer: CashTransferRequest, callback_url: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let mut req = client.post(format!("{}/remittance/v2_0/cashtransfer", self.url))
        .bearer_auth(access_token.access_token)
        .header("X-Target-Environment", self.environment.to_string())
        .header("X-Reference-Id", &transfer.external_id)
        .header("Ocp-Apim-Subscription-Key", &self.primary_key)
        .header("Content-Type", "application/json")
        .body(transfer.clone());

        if let Some(callback_url) = callback_url {
            if !callback_url.is_empty() {
                req = req.header("X-Callback-Url", callback_url);
            }
        }
        
        let res = req.send().await?;

        
        if res.status().is_success() {
            Ok(transfer.external_id)
        }else {
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, res.text().await?)))
        }
    }

    /*
        This operation is used to get the status of a transfer.
        X-Reference-Id that was passed in the post is used as reference to the request.
        @param transfer_id, the id of the transfer
        @return CashTransferResult
     */
    pub async fn get_cash_transfer_status(&self, transfer_id: &str) -> Result<CashTransferResult, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client.get(format!("{}/remittance/v2_0/cashtransfer/{}", self.url, transfer_id))
        .bearer_auth(access_token.access_token)
        .header("X-Target-Environment", self.environment.to_string())
        .header("Cache-Control", "no-cache")
        .send()
        .await?;

        
        if res.status().is_success() {
            let body = res.text().await?;
            let cash_transfer_result: CashTransferResult = serde_json::from_str(&body)?;
            Ok(cash_transfer_result)
        }else {
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, res.text().await?)))
        }
    }




    /*
        Transfer operation is used to transfer an amount from the own account to a payee account.
        Status of the transaction can validated by using the GET /transfer/{referenceId}
        @param transfer ,mtnmomo::Transfer
        @return TranserId
     */
    pub async fn transfer(&self, transfer: TransferRequest) -> Result<TranserId, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client.post(format!("{}/remittance/v1_0/transfer", self.url))
        .bearer_auth(access_token.access_token)
        .header("X-Target-Environment", self.environment.to_string())
        .header("X-Reference-Id", &transfer.external_id)
        .header("Cache-Control", "no-cache")
        .header("Ocp-Apim-Subscription-Key", &self.primary_key)
        .body(transfer.clone())
        .send()
        .await?;

    if res.status().is_success() {
        Ok(TranserId(transfer.external_id))
    }else {
        Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, res.text().await?)))
    }
    }

    /*
        This operation is used to get the status of a transfer.
        X-Reference-Id that was passed in the post is used as reference to the request.
        @param transfer_id, the id of the transfer
        @return TransferResult
     */
    pub async fn get_transfer_status(&self, transfer_id: &str) -> Result<TransferResult, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client.get(format!("{}/remittance/v1_0/transfer/{}", self.url, transfer_id))
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

    
}

impl Account for Remittance {
    /*
        This operation is used to get the balance of the account.
        @return Balance
     */
    async fn get_account_balance(&self) -> Result<Balance, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client.get(format!("{}/remittance/v1_0/account/balance", self.url))
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

    /*
        This operation is used to get the balance of the account in a specific currency.
        @param currency
        @return Balance

     */
    async fn get_account_balance_in_specific_currency(&self, currency: Currency) -> Result<Balance, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client.get(format!("{}/remittance/v1_0/account/balance/{}", self.url, currency.to_string().to_lowercase()))
        .bearer_auth(access_token.access_token)
        .header("X-Target-Environment", self.environment.to_string())
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

    /*
        This operation is used to get the basic user information of the account holder.
        @param account_holder_msisdn
        @return BasicUserInfoJsonResponse
    
     */
    async fn get_basic_user_info(&self, account_holder_msisdn: &str) -> Result<BasicUserInfoJsonResponse, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client.get(format!("{}/remittance/v1_0/accountholder/msisdn/{}/basicuserinfo", self.url, account_holder_msisdn))
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


    /*
        This operation is used to get the basic user information of the account holder.
        @param access_token
        @return BasicUserInfoJsonResponse
     */
    async fn get_user_info_with_consent(&self, access_token: String) -> Result<BasicUserInfoJsonResponse, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client.get(format!("{}/remittance/oauth2/v1_0/userinfo", self.url))
        .bearer_auth(access_token)
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


    /*
        This operation is used to validate the status of an account holder.
        @param account_holder_id
        @param account_holder_type
        @return Ok(())
     */
    async fn validate_account_holder_status(&self,  account_holder_id: &str, account_holder_type: &str) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client.get(format!("{}/remittance/v1_0/accountholder/{}/{}/active", self.url, account_holder_type, account_holder_id))
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

impl MOMOAuthorization for Remittance {

    /*
        This operation is used to create an access token.
        @return TokenResponse
    
     */
    async fn create_access_token(&self) -> Result<TokenResponse, Box<dyn std::error::Error>> {
        
        let client = reqwest::Client::new();
        let res = client.post(format!("{}/remittance/token/", self.url))
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

    /*
        This operation is used to create an OAuth2 token.
        @param auth_req_id, this is the auth_req_id of the request to pay
        @return OAuth2TokenResponse
    
     */
    async fn create_o_auth_2_token(&self, auth_req_id: String) -> Result<OAuth2TokenResponse, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client.post(format!("{}/remittance/oauth2/token/", self.url))
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

    /*
        This operation is used to authorize a user.
        @param msisdn, this is the phone number of the user
        @param callback_url, this is the url that will be used to notify the client of the status of the transaction
        @return BCAuthorizeResponse
     */
    async fn bc_authorize(&self, msisdn: String, callback_url: Option<&str>) -> Result<BCAuthorizeResponse, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let mut req = client.post(format!("{}/remittance/v1_0/bc-authorize", self.url))
        .bearer_auth(access_token.access_token)
        .header("X-Target-Environment", self.environment.to_string())
        .header("Content-type", "application/x-www-form-urlencoded")
        .header("Ocp-Apim-Subscription-Key", &self.primary_key)
        .body(BcAuthorizeRequest{login_hint: format!("ID:{}/MSISDN", msisdn), scope: "profile".to_string(), access_type: AccessType::Offline}.to_string());

        if let Some(callback_url) = callback_url {
            if !callback_url.is_empty() {
                req = req.header("X-Callback-Url", callback_url);
            }
        }
        
        let res = req.send().await?;

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
    use super::*;
    use dotenv::dotenv;
    use std::env;

    use crate::{Remittance, PartyIdType, Party};

    // #[tokio::test]
    // async fn test_cash_transfer() {
    //     dotenv().ok();
    //     let url = env::var("MTN_URL").expect("MTN_URL not set");
    //     let primary_key = env::var("MTN_REMITTANCE_PRIMARY_KEY").expect("MTN_REMITTANCE_PRIMARY_KEY not set");
    //     let secondary_key = env::var("MTN_REMITTANCE_SECONDARY_KEY").expect("MTN_REMITTANCE_SECONDARY_KEY not set");
    //     let api_user = env::var("MTN_API_USER").expect("MTN_API_USER not set");
    //     let api_key = env::var("MTN_API_KEY").expect("MTN_API_KEY not set");
    //     let remittance = Remittance::new(url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key);

    //     let payee = Party {
    //         party_id_type: PartyIdType::MSISDN,
    //         party_id: "256774290781".to_string(),
    //     };
    //     let transfer = CashTransferRequest::new("1000".to_string(), Currency::EUR, payee, "UG".to_string(), "1000".to_string(),
    //          Currency::EUR, "payer_message".to_string(), "payee_note".to_string(), PayerIdentificationType::PASS,
    //          "256774290781".to_string(), "256774290781".to_string(), "John".to_string(),
    //          "Doe".to_string(), "en".to_string(), "test@email.com".to_string(), "256774290781".to_string(), "M".to_string());
    //     remittance.cash_transfer(transfer, None).await.unwrap();
    // }


    // #[tokio::test]
    // async fn test_get_cash_transfer_status(){
    //     dotenv().ok();
    //     let url = env::var("MTN_URL").expect("MTN_URL not set");
    //     let primary_key = env::var("MTN_REMITTANCE_PRIMARY_KEY").expect("MTN_REMITTANCE_PRIMARY_KEY not set");
    //     let secondary_key = env::var("MTN_REMITTANCE_SECONDARY_KEY").expect("MTN_REMITTANCE_SECONDARY_KEY not set");
    //     let api_user = env::var("MTN_API_USER").expect("MTN_API_USER not set");
    //     let api_key = env::var("MTN_API_KEY").expect("MTN_API_KEY not set");
    //     let remittance = Remittance::new(url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key);
    //     remittance.get_cash_transfer_status("transfer_id").await.unwrap();
    // }


    
    #[tokio::test]
    async fn test_transfer(){
        dotenv().ok();
        let url = env::var("MTN_URL").expect("MTN_URL not set");
        let primary_key = env::var("MTN_REMITTANCE_PRIMARY_KEY").expect("MTN_REMITTANCE_PRIMARY_KEY not set");
        let secondary_key = env::var("MTN_REMITTANCE_SECONDARY_KEY").expect("MTN_REMITTANCE_SECONDARY_KEY not set");
        let api_user = env::var("MTN_API_USER").expect("MTN_API_USER not set");
        let api_key = env::var("MTN_API_KEY").expect("MTN_API_KEY not set");
        let remittance = Remittance::new(url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key);
        let transfer = TransferRequest::new("100".to_string(), Currency::EUR, Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "256774290781".to_string(),
        }, "payer_message".to_string(), "payee_note".to_string());

        let transer_result = remittance.transfer(transfer.clone()).await;
        assert!(transer_result.is_ok());
        assert_eq!(transer_result.unwrap().as_string(), transfer.external_id);
    }

    #[tokio::test]
    async fn test_get_transfer_status(){
        dotenv().ok();
        let url = env::var("MTN_URL").expect("MTN_URL not set");
        let primary_key = env::var("MTN_REMITTANCE_PRIMARY_KEY").expect("MTN_REMITTANCE_PRIMARY_KEY not set");
        let secondary_key = env::var("MTN_REMITTANCE_SECONDARY_KEY").expect("MTN_REMITTANCE_SECONDARY_KEY not set");
        let api_user = env::var("MTN_API_USER").expect("MTN_API_USER not set");
        let api_key = env::var("MTN_API_KEY").expect("MTN_API_KEY not set");
        let remittance = Remittance::new(url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key);
        let transfer = TransferRequest::new("100".to_string(), Currency::EUR, Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "256774290781".to_string(),
        }, "payer_message".to_string(), "payee_note".to_string());
        let transfer_result = remittance.transfer(transfer.clone()).await;
        assert!(transfer_result.is_ok());

        let status_result = remittance.get_transfer_status(transfer_result.unwrap().as_str()).await;
        assert!(status_result.is_ok());
        assert_eq!(status_result.unwrap().status, "SUCCESSFUL");
    }


    #[tokio::test] 
    async fn test_get_basic_user_info(){
        dotenv().ok();
        let url = env::var("MTN_URL").expect("MTN_URL not set");
        let primary_key = env::var("MTN_REMITTANCE_PRIMARY_KEY").expect("MTN_REMITTANCE_PRIMARY_KEY not set");
        let secondary_key = env::var("MTN_REMITTANCE_SECONDARY_KEY").expect("MTN_REMITTANCE_SECONDARY_KEY not set");
        let api_user = env::var("MTN_API_USER").expect("MTN_API_USER not set");
        let api_key = env::var("MTN_API_KEY").expect("MTN_API_KEY not set");
        let remittance = Remittance::new(url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key);
        let basic_user_info = remittance.get_basic_user_info("256774290781").await.unwrap();
        assert_ne!(basic_user_info.given_name.len(), 0);
    }



    #[tokio::test]
    async fn test_validate_account_holder_status(){
        dotenv().ok();
        let url = env::var("MTN_URL").expect("MTN_URL not set");
        let primary_key = env::var("MTN_REMITTANCE_PRIMARY_KEY").expect("MTN_REMITTANCE_PRIMARY_KEY not set");
        let secondary_key = env::var("MTN_REMITTANCE_SECONDARY_KEY").expect("MTN_REMITTANCE_SECONDARY_KEY not set");
        let api_user = env::var("MTN_API_USER").expect("MTN_API_USER not set");
        let api_key = env::var("MTN_API_KEY").expect("MTN_API_KEY not set");
        let remittance = Remittance::new(url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key);
        let holder_status_result = remittance.validate_account_holder_status("256774290781", "msisdn").await;
        assert!(holder_status_result.is_ok());
    }

    #[tokio::test]
    async fn test_get_account_balance() {
        dotenv().ok();
        let url = env::var("MTN_URL").expect("MTN_URL not set");
        let primary_key = env::var("MTN_REMITTANCE_PRIMARY_KEY").expect("MTN_REMITTANCE_PRIMARY_KEY not set");
        let secondary_key = env::var("MTN_REMITTANCE_SECONDARY_KEY").expect("MTN_REMITTANCE_SECONDARY_KEY not set");
        let api_user = env::var("MTN_API_USER").expect("MTN_API_USER not set");
        let api_key = env::var("MTN_API_KEY").expect("MTN_API_KEY not set");
        let remittance = Remittance::new(url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key);
        let  balance_result= remittance.get_account_balance().await;
        if balance_result.is_ok() {
            assert!(balance_result.is_ok());
            assert_eq!(balance_result.unwrap().currency, Currency::EUR);
        }
        
    }

    // #[tokio::test]
    // async fn test_get_account_balance_in_specific_currency() {
    //     dotenv().ok();
    //     let url = env::var("MTN_URL").expect("MTN_URL not set");
    //     let primary_key = env::var("MTN_REMITTANCE_PRIMARY_KEY").expect("MTN_REMITTANCE_PRIMARY_KEY not set");
    //     let secondary_key = env::var("MTN_REMITTANCE_SECONDARY_KEY").expect("MTN_REMITTANCE_SECONDARY_KEY not set");
    //     let api_user = env::var("MTN_API_USER").expect("MTN_API_USER not set");
    //     let api_key = env::var("MTN_API_KEY").expect("MTN_API_KEY not set");
    //     let remittance = Remittance::new(url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key);
    //     let balance: Balance = remittance.get_account_balance_in_specific_currency(Currency::EUR).await.unwrap();
    //     println!("{:?}", balance);
    //     // todo()
    // }


    #[tokio::test]
    async fn test_bc_authorize(){
        dotenv().ok();
        let url = env::var("MTN_URL").expect("MTN_URL not set");
        let api_user = env::var("MTN_API_USER").expect("MTN_API_USER not set");
        let api_key = env::var("MTN_API_KEY").expect("MTN_API_KEY not set");

        let primary_key = env::var("MTN_REMITTANCE_PRIMARY_KEY").expect("MTN_REMITTANCE_PRIMARY_KEY not set");
        let secondary_key = env::var("MTN_REMITTANCE_SECONDARY_KEY").expect("MTN_REMITTANCE_SECONDARY_KEY not set");

        let remittance = Remittance::new(url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key);
        let bc_authorize_result = remittance.bc_authorize("563607".to_string(), None).await;
        assert!(bc_authorize_result.is_ok());
        assert_ne!(bc_authorize_result.unwrap().auth_req_id.len(), 0);
    }

    #[tokio::test]
    async fn test_create_o_auth_2_token(){
        dotenv().ok();
        let url = env::var("MTN_URL").expect("MTN_URL not set");
        let api_user = env::var("MTN_API_USER").expect("MTN_API_USER not set");
        let api_key = env::var("MTN_API_KEY").expect("MTN_API_KEY not set");

        let primary_key = env::var("MTN_REMITTANCE_PRIMARY_KEY").expect("MTN_REMITTANCE_PRIMARY_KEY not set");
        let secondary_key = env::var("MTN_REMITTANCE_SECONDARY_KEY").expect("MTN_REMITTANCE_SECONDARY_KEY not set");

        let remittance = Remittance::new(url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key);
        let bc_authorize_result = remittance.bc_authorize("563607".to_string(), None).await;
        assert!(bc_authorize_result.is_ok());
        let auth_req_id = bc_authorize_result.unwrap().auth_req_id;
        let res = remittance.create_o_auth_2_token(auth_req_id).await;
        assert!(res.is_ok());
        assert_ne!(res.unwrap().access_token.len(), 0);
    }

    #[tokio::test]
    async fn test_get_user_info_with_consent(){
        dotenv().ok();
        let url = env::var("MTN_URL").expect("MTN_URL not set");
        let primary_key = env::var("MTN_REMITTANCE_PRIMARY_KEY").expect("MTN_REMITTANCE_PRIMARY_KEY not set");
        let secondary_key = env::var("MTN_REMITTANCE_SECONDARY_KEY").expect("MTN_REMITTANCE_SECONDARY_KEY not set");
        let api_user = env::var("MTN_API_USER").expect("MTN_API_USER not set");
        let api_key = env::var("MTN_API_KEY").expect("MTN_API_KEY not set");
        let remittance = Remittance::new(url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key);
        let bc_authorize_result = remittance.bc_authorize("563607".to_string(), None).await;
        assert!(bc_authorize_result.is_ok());
        let auth_req_id = bc_authorize_result.unwrap().auth_req_id;
        let res = remittance.create_o_auth_2_token(auth_req_id).await;
        assert!(res.is_ok());
        let user_info_with_consent = remittance.get_user_info_with_consent(res.unwrap().access_token).await.unwrap();
        assert_ne!(user_info_with_consent.family_name.len(), 0);
    }

}