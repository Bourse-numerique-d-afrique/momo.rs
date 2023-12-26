use crate::{
    enums::{environment::Environment, access_type::AccessType},
    requests::{refund::Refund, transfer::Transfer, bc_authorize::BcAuthorize},
    responses::{
        account_info::BasicUserInfoJsonResponse, account_info_consent::UserInfoWithConsent,
        bcauthorize_response::BCAuthorizeResponse, oauth2tokenresponse::OAuth2TokenResponse,
        token_response::TokenResponse,
    },
    traits::{account::Account, auth::MOMOAuthorization},
};

use chrono::{Utc, DateTime, NaiveDateTime};
use rusqlite::{params, Connection, Result};

use crate::structs::balance::Balance;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

pub struct Disbursements {
    pub url: String,
    pub primary_key: String,
    pub secondary_key: String,
    pub environment: Environment,
    pub api_user: String,
    pub api_key: String,
    pub conn_pool: Pool<SqliteConnectionManager>,
}

impl Disbursements {
    /*
       create a new instance of Disbursements product
       @param url
       @param environment
       @return Disbursements
    */
    pub fn new(url: String, environment: Environment, api_user: String, api_key: String, primary_key: String, secondary_key: String) -> Disbursements {
        let conn = Connection::open("disbursement_access_tokens.db").unwrap();
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
        Disbursements {
            url,
            primary_key,
            secondary_key,
            environment,
            api_key,
            api_user,
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
       deposit operation is used to deposit an amount from the owner’s account to a payee account.
       Status of the transaction can be validated by using the GET /deposit/{referenceId}
       @return Ok(())
    */
    pub async fn deposit_v1(&self) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client
            .post(format!(
                "{}/disbursement/v1_0/deposit",
                self.url
            ))
            .bearer_auth(access_token.access_token)
            .header("X-Target-Environment", self.environment.to_string())
            .header("X-Reference-Id", "value")
            .header("Cache-Control", "no-cache")
            .body(Transfer {
                amount: todo!(),
                currency: todo!(),
                external_id: todo!(),
                payee: todo!(),
                payer_message: todo!(),
                payee_note: todo!(),
            })
            .send()
            .await?;

        let response = res.text().await?;
        Ok(())
    }

    /*
       deposit operation is used to deposit an amount from the owner’s account to a payee account.
       Status of the transaction can be validated by using the GET /deposit/{referenceId}
       @return Ok(())
    */
    pub async fn deposit_v2(&self) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client
            .post(format!(
                "{}/disbursement/v2_0/deposit",
                self.url
            ))
            .bearer_auth(access_token.access_token)
            .header("X-Target-Environment", self.environment.to_string())
            .header("X-Reference-Id", "value")
            .header("Cache-Control", "no-cache")
            .body(Transfer {
                amount: todo!(),
                currency: todo!(),
                external_id: todo!(),
                payee: todo!(),
                payer_message: todo!(),
                payee_note: todo!(),
            })
            .send()
            .await?;

        let response = res.text().await?;
        Ok(())
    }

    /*
       This operation is used to get the status of a deposit.
       X-Reference-Id that was passed in the post is used as reference to the request.
    */
    pub async fn get_deposit_status(&self) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client
            .post(format!(
                "{}/disbursement/v1_0/deposit/{}",
                self.url, "referenceId"
            ))
            .bearer_auth(access_token.access_token)
            .header("X-Target-Environment", self.environment.to_string())
            .header("Cache-Control", "no-cache")
            .send()
            .await?;

        let response = res.text().await?;
        Ok(())
    }

    /*
       This operation is used to get the status of a refund.
       X-Reference-Id that was passed in the post is used as reference to the request.

       @return Ok(())
    */
    pub async fn get_refund_status(&self, reference_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client
            .post(format!(
                "{}/disbursement/v1_0/refund/{}",
                self.url, reference_id
            ))
            .bearer_auth(access_token.access_token)
            .header("X-Target-Environment", self.environment.to_string())
            .header("Cache-Control", "no-cache")
            .send()
            .await?;
        let response = res.text().await?;
        Ok(())
    }

    /*
       This operation is used to get the status of a transfer.
       X-Reference-Id that was passed in the post is used as reference to the request.
       @return Ok(())
    */
    pub async fn get_transfer_status(&self) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client
            .post(format!(
                "{}/disbursement/v1_0/transfer/{}",
                self.url, "referenceId"
            ))
            .bearer_auth(access_token.access_token)
            .header("X-Target-Environment", self.environment.to_string())
            .header("Cache-Control", "no-cache")
            .send()
            .await?;

        let response = res.text().await?;
        Ok(())
    }

    /*
       refund operation is used to refund an amount from the owner’s account to a payee account.
       Status of the transaction can be validated by using the GET /refund/{referenceId}
       @return Ok(())
    */
    pub async fn refund_v1(&self, refund: Refund) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client
            .post(format!(
                "{}/disbursement/v1_0/refund",
                self.url
            ))
            .bearer_auth(access_token.access_token)
            .header("X-Target-Environment", self.environment.to_string())
            .header("X-Reference-Id", "")
            .header("Cache-Control", "no-cache")
            .body(refund)
            .send()
            .await?;

        let response = res.text().await?;
        Ok(())
    }

    /*
       refund operation is used to refund an amount from the owner’s account to a payee account.
       Status of the transaction can be validated by using the GET /refund/{referenceId}
       @return Ok(())
    */
    pub async fn refund_v2(&self, refund: Refund) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client
            .post(format!(
                "{}/disbursement/v2_0/refund",
                self.url
            ))
            .bearer_auth(access_token.access_token)
            .header("X-Target-Environment", self.environment.to_string())
            .header("X-Reference-Id", "")
            .header("Cache-Control", "no-cache")
            .body(refund)
            .send()
            .await?;

        let response = res.text().await?;
        Ok(())
    }

    /*
       transfer operation is used to transfer an amount from the owner’s account to a payee account.
       Status of the transaction can be validated by using the GET /transfer/{referenceId}
       @return Ok(())
    */
    pub async fn transfer(&self, transfer: Transfer) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client
            .post(format!(
                "{}/disbursement/v1_0/transfer",
                self.url
            ))
            .bearer_auth(access_token.access_token)
            .header("X-Target-Environment", self.environment.to_string())
            .header("X-Reference-Id", "")
            .header("Cache-Control", "no-cache")
            .body(transfer)
            .send()
            .await?;

        let response = res.text().await?;
        Ok(())
    }
}

impl Account for Disbursements {
    async fn get_account_balance(&self) -> Result<Balance, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client
            .post(format!(
                "{}/disbursement/v1_0/account/balance",
                self.url
            ))
            .header("Authorization", format!("Basic {}", ""))
            .header("X-Target-Environment", self.environment.to_string())
            .header("Cache-Control", "no-cache")
            .send()
            .await?;

        let response = res.text().await?;
        let balance: Balance = serde_json::from_str(&response)?;

        Ok(balance)
    }

    async fn get_account_balance_in_specific_currency(
        &self,
        currency: String,
    ) -> Result<Balance, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client
            .post(format!(
                "{}/disbursement/v1_0/preapproval",
                self.url
            ))
            .header("Authorization", format!("Basic {}", ""))
            .header("X-Target-Environment", self.environment.to_string())
            .header("Cache-Control", "no-cache")
            .send()
            .await?;

        let response = res.text().await?;
        let balance: Balance = serde_json::from_str(&response)?;
        Ok(balance)
    }

    async fn get_basic_user_info(
        &self, account_holder_msisdn: &str
    ) -> Result<BasicUserInfoJsonResponse, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client
            .post(format!(
                "{}/disbursement/v1_0/preapproval",
                self.url
            ))
            .header("Authorization", format!("Basic {}", ""))
            .header("X-Target-Environment", self.environment.to_string())
            .header("Cache-Control", "no-cache")
            .send()
            .await?;

        let body = res.text().await?;
        let basic_user_info: BasicUserInfoJsonResponse = serde_json::from_str(&body)?;
        Ok(basic_user_info)
    }

    async fn get_user_info_with_consent(
        &self,
    ) -> Result<UserInfoWithConsent, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client
            .post(format!(
                "{}/disbursement/v1_0/preapproval",
                self.url
            ))
            .header("Authorization", format!("Basic {}", ""))
            .header("X-Target-Environment", self.environment.to_string())
            .header("Cache-Control", "no-cache")
            .send()
            .await?;

        let body = res.text().await?;
        let basic_user_info: UserInfoWithConsent = serde_json::from_str(&body)?;
        Ok(basic_user_info)
    }

    async fn validate_account_holder_status(
        &self,
        account_holder_id: &str, account_holder_type: &str
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client
            .post(format!(
                "{}/disbursement/v1_0/preapproval",
                self.url
            ))
            .header("Authorization", format!("Basic {}", ""))
            .header("X-Target-Environment", self.environment.to_string())
            .header("Cache-Control", "no-cache")
            .send()
            .await?;

        let response = res.text().await?;
        Ok(())
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
            .send()
            .await?;

        let body = res.text().await?;
        let token_response: TokenResponse = serde_json::from_str(&body)?;
        Ok(token_response)
    }

    async fn create_o_auth_2_token(
        &self,
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
            .header("Cache-Control", "no-cache")
            .send()
            .await?;

        let body = res.text().await?;
        let token_response: OAuth2TokenResponse = serde_json::from_str(&body)?;
        Ok(token_response)
    }

    async fn bc_authorize(&self, msisdn: String) -> Result<BCAuthorizeResponse, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client
            .post(format!(
                "{}/disbursement/v1_0/bc-authorize",
                self.url
            ))
            .basic_auth(self.api_user.to_string(), Some(self.api_key.to_string()))
            .header("X-Target-Environment", self.environment.to_string())
            .header("X-Callback-Url", "callback")
            .header("Content-type", "application/x-www-form-urlencoded")
            .header("Cache-Control", "no-cache")
            .body(BcAuthorize{login_hint: format!("ID:{}/MSISDN", msisdn), scope: "profile".to_string(), access_type: AccessType::Offline}) // scope can be profile
            .send()
            .await?;

        let body = res.text().await?;
        let token_response: BCAuthorizeResponse = serde_json::from_str(&body)?;
        Ok(token_response)
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use dotenv::dotenv;
    use crate::{
        enums::environment::Environment,
        products::disbursements::Disbursements,
        traits::{account::Account, auth::MOMOAuthorization},
    };

    #[tokio::test]
    async fn test_get_account_balance() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");

        let disbursements = Disbursements::new(
            mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key );
        let balance = disbursements.get_account_balance().await.unwrap();
        println!("{:?}", balance);
    }

    #[tokio::test]
    async fn test_get_account_balance_in_specific_currency() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");

        let disbursements = Disbursements::new(
            mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key
        );
        let balance = disbursements.get_account_balance_in_specific_currency("EUR".to_string()).await.unwrap();
        println!("{:?}", balance);
    }

    #[tokio::test]
    async fn test_get_basic_user_info() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");
        let disbursements = Disbursements::new(
            mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key
        );
        let basic_user_info = disbursements.get_basic_user_info("256774290781").await.unwrap();
        println!("{:?}", basic_user_info);
    }

    #[tokio::test]
    async fn test_get_user_info_with_consent() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");
        let disbursements = Disbursements::new(
            mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key
        );
        let user_info_with_consent = disbursements.get_user_info_with_consent().await.unwrap();
        println!("{:?}", user_info_with_consent);
    }

    #[tokio::test]
    async fn test_validate_account_holder_status() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");
        let disbursements = Disbursements::new(
            mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key
        );
        let validate_account_holder_status = disbursements.validate_account_holder_status("256774290781", "MSISDN").await.unwrap();
        println!("{:?}", validate_account_holder_status);
    }

    #[tokio::test]
    async fn test_create_access_token() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");
        let disbursements = Disbursements::new(
            mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key
        );
        let access_token = disbursements.create_access_token().await.unwrap();
        println!("{:?}", access_token);
    }

    #[tokio::test]
    async fn test_create_o_auth_2_token() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");
        let disbursements = Disbursements::new(
            mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key
        );
        let o_auth_2_token = disbursements.create_o_auth_2_token().await.unwrap();
        println!("{:?}", o_auth_2_token);
    }

    #[tokio::test]
    async fn test_bc_authorize() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");
        let disbursements = Disbursements::new(
            mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key
        );
        let bc_authorize = disbursements.bc_authorize("256774290781".to_string()).await.unwrap();
        println!("{:?}", bc_authorize);
    }

    #[tokio::test]
    async fn test_deposit_v1() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");
        let disbursements = Disbursements::new(
            mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key
        );
        disbursements.deposit_v1().await.unwrap();
    }

    #[tokio::test]
    async fn test_deposit_v2() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");
        let disbursements = Disbursements::new(
            mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key);
        disbursements.deposit_v2().await.unwrap();
    }

    #[tokio::test]
    async fn test_get_deposit_status() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");
        let disbursements = Disbursements::new(
            mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key
        );
        disbursements.get_deposit_status().await.unwrap();
    }

    #[tokio::test]
    async fn test_get_refund_status() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");
        let disbursements = Disbursements::new(
            mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key
        );
        disbursements.get_refund_status("reference_id").await.unwrap();
    }



    #[tokio::test]
    async fn test_refund_v1() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");
        let disbursements = Disbursements::new(
            mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key
        );

        let refund = crate::requests::refund::Refund {
            amount: "1000".to_string(),
            currency: "EUR".to_string(),
            external_id: "123456789".to_string(),
            payee_note: "payee_note".to_string(),
            payer_message: "payer_message".to_string(),
            reference_id_to_refund: "reference_id".to_string(),
        };
        disbursements.refund_v1(refund).await.unwrap();
    }

    #[tokio::test]
    async fn test_refund_v2() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");
        let disbursements = Disbursements::new(
            mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key
        );

        let refund = crate::requests::refund::Refund {
            amount: "1000".to_string(),
            currency: "EUR".to_string(),
            external_id: "123456789".to_string(),
            payee_note: "payee_note".to_string(),
            payer_message: "payer_message".to_string(),
            reference_id_to_refund: "reference_id".to_string(),
        };
        disbursements.refund_v2(refund).await.unwrap();
    }

    #[tokio::test]
    async fn test_transfer() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");
        let disbursements = Disbursements::new(
            mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key
        );

        let transfer = crate::requests::transfer::Transfer {
            amount: "1000".to_string(),
            currency: "EUR".to_string(),
            external_id: "123456789".to_string(),
            payee_note: "payee_note".to_string(),
            payer_message: "payer_message".to_string(),
            payee: crate::structs::party::Party {
                party_id_type: "MSISDN".to_string(),
                party_id: "256774290781".to_string(),
            },
        };
        disbursements.transfer(transfer).await.unwrap();
    }

    #[tokio::test]
    async fn test_get_transfer_status() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");
        let disbursements = Disbursements::new(
            mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key
        );
        disbursements.get_transfer_status().await.unwrap();
    }

  

}
