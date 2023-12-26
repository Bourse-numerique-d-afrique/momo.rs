use crate::{traits::{account::Account, auth::MOMOAuthorization}, responses::{token_response::TokenResponse, bcauthorize_response::BCAuthorizeResponse, oauth2tokenresponse::OAuth2TokenResponse, account_info_consent::UserInfoWithConsent, account_info::BasicUserInfoJsonResponse}, enums::{environment::Environment, access_type::AccessType}, requests::bc_authorize::BcAuthorize};
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
        let manager = SqliteConnectionManager::file("collection_access_tokens.db");
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
    pub async fn cash_transfer(&self) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client.post(format!("{}/remittance/v1_0/preapproval", self.url))
        .bearer_auth(access_token.access_token)
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

    async fn get_basic_user_info(&self, account_holder_msisdn: &str) -> Result<BasicUserInfoJsonResponse, Box<dyn std::error::Error>> {
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

    async fn validate_account_holder_status(&self,  account_holder_id: &str, account_holder_type: &str) -> Result<(), Box<dyn std::error::Error>> {
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
        
        let client = reqwest::Client::new();
        let res = client.post(format!("{}/remittance/token/", self.url))
        .basic_auth(self.api_user.to_string(), Some(self.api_key.to_string()))
        .header("Cache-Control", "no-cache")
        .send().await?;

        let body = res.text().await?;
        let token_response: TokenResponse = serde_json::from_str(&body)?;
        Ok(token_response)
    }

    async fn create_o_auth_2_token(&self) -> Result<OAuth2TokenResponse, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client.post(format!("{}/remittance/oauth2/token/", self.url))
        .basic_auth(self.api_user.to_string(), Some(self.api_key.to_string()))
        .header("X-Target-Environment", self.environment.to_string())
        .header("Content-type", "application/x-www-form-urlencoded")
        .header("Cache-Control", "no-cache")
        .send().await?;

        let body = res.text().await?;
        let token_response: OAuth2TokenResponse = serde_json::from_str(&body)?;
        Ok(token_response)
    }

    async fn bc_authorize(&self, msisdn: String) -> Result<BCAuthorizeResponse, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client.post(format!("{}/remittance/v1_0/bc-authorize", self.url))
        .basic_auth(self.api_user.to_string(), Some(self.api_key.to_string()))
        .header("X-Target-Environment", self.environment.to_string())
        .header("X-Callback-Url", "callback")
        .header("Content-type", "application/x-www-form-urlencoded")
        .header("Cache-Control", "no-cache")
        .body(BcAuthorize{login_hint: format!("ID:{}/MSISDN", msisdn), scope: "profile".to_string(), access_type: AccessType::Offline}) // scope can be profile
        .send().await?;

        let body = res.text().await?;
        let token_response: BCAuthorizeResponse = serde_json::from_str(&body)?;
        Ok(token_response)
    }
}


#[cfg(test)]
mod tests {
    use crate::{enums::environment::Environment, products::remittance::Remittance, traits::account::Account};
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
        remittance.cash_transfer().await.unwrap();
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
        remittance.transfer().await.unwrap();
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
        remittance.get_cash_transfer_status().await.unwrap();
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
        remittance.get_transfer_status().await.unwrap();
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