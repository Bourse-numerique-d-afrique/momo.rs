use crate::{traits::{account::Account, auth::MOMOAuthorization}, responses::{token_response::TokenResponse, bcauthorize_response::BCAuthorizeResponse, oauth2tokenresponse::OAuth2TokenResponse, account_info_consent::UserInfoWithConsent, account_info::BasicUserInfoJsonResponse, transfer_result::TransferResult, cash_transfer_result::CashTransferResult}, enums::{environment::Environment, access_type::AccessType}, requests::{bc_authorize::BcAuthorize, transfer::Transfer, cash_transfer::CashTransferRequest}};
use chrono::{Utc, DateTime, NaiveDateTime};
use crate::structs::balance::Balance;
use rusqlite::{params, Connection, Result};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;



pub struct Remittance{
    pub url: String,
    pub primary_key: String,
    pub secondary_key: String,
    pub environment: Environment,
    pub api_user: String,
    pub api_key: String,
    pub conn_pool: Pool<SqliteConnectionManager>,
}

impl Remittance {
    /*
        create a new instance of Remittance product
        @param url
        @param environment
        @return Remittance
    
     */
    pub fn new(url: String, environment: Environment, api_user: String, api_key: String, primary_key: String, secondary_key: String) -> Remittance {
        let conn = Connection::open("remittance_access_tokens.db").unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS access_tokens (
                id INTEGER PRIMARY KEY,
                access_token TEXT NOT NULL,
                token_type TEXT NOT NULL,
                expires_in INTEGER NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            params![],
        ).unwrap();
        let manager = SqliteConnectionManager::file("remittance_access_tokens.db");
        let pool = r2d2::Pool::new(manager).expect("Failed to create pool.");
        Remittance{
            url,
            primary_key,
            secondary_key,
            environment,
            api_user,
            api_key,
            conn_pool: pool,
        }
    }

        /*
        This operation is used to insert an access token into the database
        @return Ok(())
     */
    fn insert_access_token(&self, access_token: &str, token_type: &str, expires_in: i32) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.conn_pool.get()?;
        conn.execute(
            "INSERT INTO access_tokens (access_token, token_type, expires_in) VALUES (?1, ?2, ?3)",
            params![access_token, token_type, expires_in],
        )?;

        Ok(())
    }

    /*
        This operation is used to get the latest access token from the database
        @return TokenResponse
     */
    async fn get_valid_access_token(&self) -> Result<TokenResponse, Box<dyn std::error::Error>> {
        let conn = self.conn_pool.get()?;
        let mut stmt = conn.prepare("SELECT * FROM access_tokens ORDER BY created_at DESC LIMIT 1")?;
        let access_result = stmt.query(params![]);
        let mut access = access_result.unwrap();
        let r = access.next().unwrap();
        if r.is_some() {
            println!("is some");
            let row = r.unwrap();
            let created_at: String = row.get(4)?;
            let naive_datetime = NaiveDateTime::parse_from_str(&created_at, "%Y-%m-%d %H:%M:%S")?;
            let date_time: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive_datetime, Utc);
            let now = Utc::now();
            let duration = now.signed_duration_since(date_time);
            let duration = duration.num_seconds();
            if duration > 3600 {
                let token: TokenResponse = self.create_access_token().await?;
                return Ok(token);
            }else{
                let token = TokenResponse{
                    access_token: row.get(1)?,
                    token_type: row.get(2)?,
                    expires_in: row.get(3)?,
                };
                return Ok(token);
            }
        }else{
            let token: TokenResponse = self.create_access_token().await?;
            return Ok(token);
        }
    }


    /*
        Cash transfer operation is used to transfer an amount from the owner’s account to a payee account.
        Status of the transaction can be validated by using GET /cashtransfer/{referenceId}
        @return Ok(())
     */
    pub async fn cash_transfer(&self, transfer: CashTransferRequest, callback_url: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client.post(format!("{}/remittance/v2_0/cashtransfer", self.url))
        .bearer_auth(access_token.access_token)
        .header("X-Target-Environment", self.environment.to_string())
        .header("Cache-Control", "no-cache")
        .header("X-Reference-Id", &transfer.external_id)
        .header("X-Callback-Url", callback_url.unwrap_or(""))
        .body(transfer)
        .send().await?;

        
        if res.status().is_success() {
            Ok(())
        }else {
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, res.text().await?)))
        }
    }

    /*
        This operation is used to get the status of a transfer.
        X-Reference-Id that was passed in the post is used as reference to the request.
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
        This operation is used to get the status of a transfer.
        X-Reference-Id that was passed in the post is used as reference to the request.
     */
    pub async fn get_transfer_status(&self, transfer_id: &str) -> Result<TransferResult, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client.post(format!("{}/remittance/v1_0/transfer/{}", self.url, transfer_id))
        .bearer_auth(access_token.access_token)
        .header("X-Target-Environment", self.environment.to_string())
        .header("Cache-Control", "no-cache")
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
        Transfer operation is used to transfer an amount from the own account to a payee account.
        Status of the transaction can validated by using the GET /transfer/{referenceId}
     */
    pub async fn transfer(&self, transfer: Transfer) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client.post(format!("{}/remittance/v1_0/preapproval", self.url))
        .bearer_auth(access_token.access_token)
        .header("X-Target-Environment", self.environment.to_string())
        .header("X-Reference-Id", &transfer.external_id)
        .header("Cache-Control", "no-cache")
        .body(transfer)
        .send()
        .await?;

    if res.status().is_success() {
        Ok(())
    }else {
        Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, res.text().await?)))
    }
    }

    
}

impl Account for Remittance {
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

    async fn get_account_balance_in_specific_currency(&self, currency: String) -> Result<Balance, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client.post(format!("{}/remittance/v1_0/account/balance/{}", self.url, currency))
        .bearer_auth(access_token.access_token)
        .header("Content-Type", "application/json")
        .header("X-Target-Environment", self.environment.to_string())
        .header("Ocp-Apim-Subscription-Key", &self.primary_key)
        .header("Content-Length", "0")
        .body("")
        .send().await?;

        if res.status().is_success() {
            let body = res.text().await?;
            let balance: Balance = serde_json::from_str(&body)?;
            Ok(balance)
        }else {
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, res.text().await?)))
        }
    }

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

    async fn get_user_info_with_consent(&self) -> Result<UserInfoWithConsent, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client.post(format!("{}/remittance/oauth2/v1_0/userinfo", self.url))
        .bearer_auth(access_token.access_token)
        .header("X-Target-Environment", self.environment.to_string())
        .header("Ocp-Apim-Subscription-Key", &self.primary_key)
        .header("Cache-Control", "no-cache").send().await?;

        if res.status().is_success() {
            let body = res.text().await?;
            let basic_user_info: UserInfoWithConsent = serde_json::from_str(&body)?;
            Ok(basic_user_info)
        }else {
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, res.text().await?)))
        }
    }

    async fn validate_account_holder_status(&self,  account_holder_id: &str, account_holder_type: &str) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client.post(format!("{}/remittance/v1_0/accountholder/{}/{}/active", self.url, account_holder_id, account_holder_type))
        .bearer_auth(access_token.access_token)
        .header("X-Target-Environment", self.environment.to_string())
        .header("Ocp-Apim-Subscription-Key", &self.primary_key)
        .header("Cache-Control", "no-cache").send().await?;

        if res.status().is_success() {
            Ok(())
        }else {
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, res.text().await?)))
        }
    }
}

impl MOMOAuthorization for Remittance {
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
            self.insert_access_token(&token_response.access_token, &token_response.token_type, token_response.expires_in)?;
            Ok(token_response)
        } else {
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, res.text().await?)))
        }
    }

    async fn create_o_auth_2_token(&self) -> Result<OAuth2TokenResponse, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client.post(format!("{}/remittance/oauth2/token/", self.url))
        .basic_auth(self.api_user.to_string(), Some(self.api_key.to_string()))
        .header("X-Target-Environment", self.environment.to_string())
        .header("Content-type", "application/x-www-form-urlencoded")
        .header("Cache-Control", "no-cache")
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

    async fn bc_authorize(&self, msisdn: String) -> Result<BCAuthorizeResponse, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client.post(format!("{}/remittance/v1_0/bc-authorize", self.url))
        .header("X-Target-Environment", self.environment.to_string())
        .header("X-Callback-Url", "callback")
        .header("Content-type", "application/x-www-form-urlencoded")
        .header("Cache-Control", "no-cache")
        .body(BcAuthorize{login_hint: format!("ID:{}/MSISDN", msisdn), scope: "profile".to_string(), access_type: AccessType::Offline}) // scope can be profile
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
    use crate::{enums::environment::Environment, products::remittance::Remittance, traits::account::Account, requests::{cash_transfer::CashTransferRequest, transfer::Transfer}, structs::party::Party};
    use dotenv::dotenv;
    use std::env;
    use crate::structs::balance::Balance;

    #[tokio::test]
    async fn test_cash_transfer() {
        dotenv().ok();
        let url = env::var("URL").expect("URL not set");
        let primary_key = env::var("PRIMARY_KEY").expect("PRIMARY_KEY not set");
        let secondary_key = env::var("SECONDARY_KEY").expect("SECONDARY_KEY not set");
        let api_user = env::var("API_USER").expect("API_USER not set");
        let api_key = env::var("API_KEY").expect("API_KEY not set");
        let remittance = Remittance::new(url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key);
        let transer = CashTransferRequest {
            amount: "1000".to_string(),
            currency: "EUR".to_string(),
            external_id: "123456789".to_string(),
            payee_note: "payee_note".to_string(),
            payer_message: "payer_message".to_string(),
            payee: Party {
                party_id_type: "MSISDN".to_string(),
                party_id: "256774290781".to_string(),
            },
            originating_country: "UG".to_string(),
            original_amount: "1000".to_string(),
            original_currency: "EUR".to_string(),
            payer_identification_type: "MSISDN".to_string(),
            payer_identification_number: "256774290781".to_string(),
            payer_identity: "256774290781".to_string(),
            payer_first_name: "John".to_string(),
            payer_surname: "Doe".to_string(),
            payer_language_code: "en".to_string(),
            payer_email: "".to_string(),
            payer_gender: "M".to_string(),
            payer_msisdn: "256774290781".to_string(),
        };
        remittance.cash_transfer(transer, None).await.unwrap();
    }


    
    #[tokio::test]
    async fn test_transfer(){
        dotenv().ok();
        let url = env::var("URL").expect("URL not set");
        let primary_key = env::var("PRIMARY_KEY").expect("PRIMARY_KEY not set");
        let secondary_key = env::var("SECONDARY_KEY").expect("SECONDARY_KEY not set");
        let api_user = env::var("API_USER").expect("API_USER not set");
        let api_key = env::var("API_KEY").expect("API_KEY not set");
        let remittance = Remittance::new(url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key);
        let transfer = Transfer {
            amount: "1000".to_string(),
            currency: "EUR".to_string(),
            external_id: "123456789".to_string(),
            payee_note: "payee_note".to_string(),
            payer_message: "payer_message".to_string(),
            payee: Party {
                party_id_type: "MSISDN".to_string(),
                party_id: "256774290781".to_string(),
            },
        };
        remittance.transfer(transfer).await.unwrap();
    }

    #[tokio::test]
    async fn test_get_cash_transfer_status(){
        dotenv().ok();
        let url = env::var("URL").expect("URL not set");
        let primary_key = env::var("PRIMARY_KEY").expect("PRIMARY_KEY not set");
        let secondary_key = env::var("SECONDARY_KEY").expect("SECONDARY_KEY not set");
        let api_user = env::var("API_USER").expect("API_USER not set");
        let api_key = env::var("API_KEY").expect("API_KEY not set");
        let remittance = Remittance::new(url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key);
        remittance.get_cash_transfer_status("transfer_id").await.unwrap();
    }


    #[tokio::test]
    async fn test_get_transfer_status(){
        dotenv().ok();
        let url = env::var("URL").expect("URL not set");
        let primary_key = env::var("PRIMARY_KEY").expect("PRIMARY_KEY not set");
        let secondary_key = env::var("SECONDARY_KEY").expect("SECONDARY_KEY not set");
        let api_user = env::var("API_USER").expect("API_USER not set");
        let api_key = env::var("API_KEY").expect("API_KEY not set");
        let remittance = Remittance::new(url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key);
        remittance.get_transfer_status("transfer_id").await.unwrap();
    }

    #[tokio::test]
    async fn test_get_basic_user_info(){
        dotenv().ok();
        let url = env::var("URL").expect("URL not set");
        let primary_key = env::var("PRIMARY_KEY").expect("PRIMARY_KEY not set");
        let secondary_key = env::var("SECONDARY_KEY").expect("SECONDARY_KEY not set");
        let api_user = env::var("API_USER").expect("API_USER not set");
        let api_key = env::var("API_KEY").expect("API_KEY not set");
        let remittance = Remittance::new(url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key);
        let basic_user_info = remittance.get_basic_user_info("256774290781").await.unwrap();
        println!("{:?}", basic_user_info);
    }

    #[tokio::test]
    async fn test_get_user_info_with_consent(){
        dotenv().ok();
        let url = env::var("URL").expect("URL not set");
        let primary_key = env::var("PRIMARY_KEY").expect("PRIMARY_KEY not set");
        let secondary_key = env::var("SECONDARY_KEY").expect("SECONDARY_KEY not set");
        let api_user = env::var("API_USER").expect("API_USER not set");
        let api_key = env::var("API_KEY").expect("API_KEY not set");
        let remittance = Remittance::new(url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key);
        let user_info_with_consent = remittance.get_user_info_with_consent().await.unwrap();
        println!("{:?}", user_info_with_consent);
    }

    #[tokio::test]
    async fn test_validate_account_holder_status(){
        dotenv().ok();
        let url = env::var("URL").expect("URL not set");
        let primary_key = env::var("PRIMARY_KEY").expect("PRIMARY_KEY not set");
        let secondary_key = env::var("SECONDARY_KEY").expect("SECONDARY_KEY not set"); 
        let api_user = env::var("API_USER").expect("API_USER not set");
        let api_key = env::var("API_KEY").expect("API_KEY not set"); 
        let remittance = Remittance::new(url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key);
        remittance.validate_account_holder_status("256774290781", "msisdn").await.unwrap();
    }

    #[tokio::test]
    async fn test_get_account_balance() {
        dotenv().ok();
        let url = env::var("URL").expect("URL not set");
        let primary_key = env::var("PRIMARY_KEY").expect("PRIMARY_KEY not set");
        let secondary_key = env::var("SECONDARY_KEY").expect("SECONDARY_KEY not set");
        let api_user = env::var("API_USER").expect("API_USER not set");
        let api_key = env::var("API_KEY").expect("API_KEY not set");
        let remittance = Remittance::new(url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key);
        let balance: Balance = remittance.get_account_balance().await.unwrap();
        println!("{:?}", balance);
    }

    #[tokio::test]
    async fn test_get_account_balance_in_specific_currency() {
        dotenv().ok();
        let url = env::var("URL").expect("URL not set");
        let primary_key = env::var("PRIMARY_KEY").expect("PRIMARY_KEY not set");
        let secondary_key = env::var("SECONDARY_KEY").expect("SECONDARY_KEY not set");
        let api_user = env::var("API_USER").expect("API_USER not set");
        let api_key = env::var("API_KEY").expect("API_KEY not set");
        let remittance = Remittance::new(url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key);
        let balance: Balance = remittance.get_account_balance_in_specific_currency("EUR".to_string()).await.unwrap();
        println!("{:?}", balance);
    }

}