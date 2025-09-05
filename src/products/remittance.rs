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

use crate::{
    common::{http_client::MomoHttpClient, token_manager::ProductType},
    BCAuthorizeResponse, Balance, BasicUserInfoJsonResponse, CashTransferRequest,
    CashTransferResult, Currency, Environment, OAuth2TokenResponse, TokenResponse, TranserId,
    TransferRequest, TransferResult,
};

use super::account::Account;

pub struct Remittance {
    pub url: String,
    pub primary_key: String,
    pub secondary_key: String,
    pub environment: Environment,
    pub api_user: String,
    pub api_key: String,
    account: Account,
    http_client: MomoHttpClient,
}

impl Remittance {
    /// Create a new instance of Remittance product
    ///
    /// # Parameters
    ///
    /// * 'url',  MTN Core API url
    /// * 'environment', the environment of the installation
    /// * 'api_user'
    /// * 'api_key'
    /// * 'primary_key'
    /// * 'secondary_key'
    ///
    ///
    /// # Returns
    ///
    /// * 'Remittance', the instance of remittance
    pub fn new(
        url: String,
        environment: Environment,
        api_user: String,
        api_key: String,
        primary_key: String,
        secondary_key: String,
    ) -> Remittance {
        let account = Account {};
        let http_client = MomoHttpClient::new(
            url.clone(),
            ProductType::Remittance,
            environment,
            api_user.clone(),
            api_key.clone(),
            primary_key.clone(),
        );
        Remittance {
            url,
            primary_key,
            secondary_key,
            environment,
            api_user,
            api_key,
            account,
            http_client,
        }
    }

    /// This operation is used to create an OAuth2 token
    ///
    /// # Parameters
    ///
    /// * 'auth_req_id', this is the auth request id
    ///
    /// # Returns
    ///
    /// * 'OAuth2TokenResponse'
    #[allow(dead_code)]
    async fn create_o_auth_2_token(
        &self,
        auth_req_id: String,
    ) -> Result<OAuth2TokenResponse, Box<dyn std::error::Error>> {
        let url = format!("{}/{}", self.url, "remittance");
        let auth = crate::products::auth::Authorization {};
        auth.create_o_auth_2_token(
            url,
            self.api_user.clone(),
            self.api_key.clone(),
            self.environment,
            self.primary_key.clone(),
            auth_req_id,
        )
        .await
    }

    /// This operation is used to authorize a user.
    ///
    /// # Parameters
    ///
    /// * 'msisdn', this is the phone number of the user
    /// * 'callback_url', this is the url that will be used to notify the client of the status of the transaction
    ///
    /// # Returns
    ///
    /// * 'BCAuthorizeResponse'
    #[allow(dead_code)]
    async fn bc_authorize(
        &self,
        msisdn: String,
        callback_url: Option<&str>,
    ) -> Result<BCAuthorizeResponse, Box<dyn std::error::Error>> {
        let url = format!("{}/{}", self.url, "remittance");
        let auth = crate::products::auth::Authorization {};
        let access_token: TokenResponse = self.http_client.get_or_create_token().await?;
        auth.bc_authorize(
            url,
            self.environment,
            self.primary_key.clone(),
            msisdn,
            callback_url,
            access_token,
        )
        .await
    }

    /// Cash transfer operation is used to transfer an amount from the owner’s account to a payee account.
    /// Status of the transaction can be validated by using GET /cashtransfer/{referenceId}
    ///
    /// # Parameters
    ///
    /// * 'callback_url', optional, the url to be called when the transaction is completed
    ///
    /// # Returns
    ///
    /// * ()
    pub async fn cash_transfer(
        &self,
        transfer: CashTransferRequest,
        callback_url: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.http_client.get_or_create_token().await?;
        let mut req = client
            .post(format!("{}/remittance/v2_0/cashtransfer", self.url))
            .bearer_auth(access_token.access_token)
            .header("X-Target-Environment", self.environment.to_string())
            .header("X-Reference-Id", &transfer.external_id)
            .header("Ocp-Apim-Subscription-Key", &self.primary_key)
            .header("Content-Type", "application/json")
            .body(transfer.clone());

        if let Some(callback_url) = callback_url {
            if !callback_url.is_empty() {
                req = req.header(
                    "X-Callback-Url",
                    format!("{}/remittance_cash_transfer", callback_url),
                );
            }
        }

        let res = req.send().await?;

        if res.status().is_success() {
            Ok(transfer.external_id)
        } else {
            Err(Box::new(std::io::Error::other(res.text().await?)))
        }
    }

    /// This operation is used to get the status of a transfer.
    /// X-Reference-Id that was passed in the post is used as reference to the request.
    ///
    /// # Parameters
    /// * 'transfer_id', the id of the transfer
    ///
    /// # Returns
    ///
    /// * 'CashTransferResult'
    pub async fn get_cash_transfer_status(
        &self,
        transfer_id: &str,
    ) -> Result<CashTransferResult, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.http_client.get_or_create_token().await?;
        let res = client
            .get(format!(
                "{}/remittance/v2_0/cashtransfer/{}",
                self.url, transfer_id
            ))
            .bearer_auth(access_token.access_token)
            .header("X-Target-Environment", self.environment.to_string())
            .header("Cache-Control", "no-cache")
            .send()
            .await?;

        if res.status().is_success() {
            let body = res.text().await?;
            let cash_transfer_result: CashTransferResult = serde_json::from_str(&body)?;
            Ok(cash_transfer_result)
        } else {
            Err(Box::new(std::io::Error::other(res.text().await?)))
        }
    }

    /// Transfer operation is used to transfer an amount from the own account to a payee account.
    /// Status of the transaction can validated by using the GET /transfer/{referenceId}
    ///
    ///
    /// # Parameters
    ///
    /// * 'transfer': TransferRequest,
    ///
    /// # Returns
    ///
    /// * 'TransferId', the transfer id (MTN Momo external id)
    pub async fn transfer(
        &self,
        transfer: TransferRequest,
        callback_url: Option<&str>,
    ) -> Result<TranserId, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.http_client.get_or_create_token().await?;
        let mut req = client
            .post(format!("{}/remittance/v1_0/transfer", self.url))
            .bearer_auth(access_token.access_token)
            .header("X-Target-Environment", self.environment.to_string())
            .header("X-Reference-Id", &transfer.external_id)
            .header("Cache-Control", "no-cache")
            .header("Ocp-Apim-Subscription-Key", &self.primary_key)
            .body(transfer.clone());

        if let Some(callback_url) = callback_url {
            if !callback_url.is_empty() {
                req = req.header(
                    "X-Callback-Url",
                    format!("{}/remittance_transfer", callback_url),
                );
            }
        }

        let res = req.send().await?;

        if res.status().is_success() {
            Ok(TranserId::new(transfer.external_id))
        } else {
            Err(Box::new(std::io::Error::other(res.text().await?)))
        }
    }

    /// This operation is used to get the status of a transfer.
    /// X-Reference-Id that was passed in the post is used as reference to the request.
    ///
    /// # Parameters
    ///
    /// * 'transfer_id', the id of the transfer
    ///
    /// # Returns
    ///
    /// * 'TransferResult'
    pub async fn get_transfer_status(
        &self,
        transfer_id: &str,
    ) -> Result<TransferResult, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.http_client.get_or_create_token().await?;
        let res = client
            .get(format!(
                "{}/remittance/v1_0/transfer/{}",
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
        } else {
            Err(Box::new(std::io::Error::other(res.text().await?)))
        }
    }

    /// This operation is used to get the balance of the account.
    /// # Returns
    ///
    /// * 'Balance', the balance
    pub async fn get_account_balance(&self) -> Result<Balance, Box<dyn std::error::Error>> {
        let url = format!("{}/remittance", self.url);
        let access_token = self.http_client.get_or_create_token().await?;
        self.account
            .get_account_balance(
                url,
                self.environment,
                self.primary_key.clone(),
                access_token,
            )
            .await
    }

    /// this operation is used to get the balance of an account in a specific currency
    ///
    /// # Parameters
    ///
    /// * 'currency', Currency of the account to get balance from
    /// # Returns
    ///
    /// * 'Balance', the balance
    pub async fn get_account_balance_in_specific_currency(
        &self,
        currency: Currency,
    ) -> Result<Balance, Box<dyn std::error::Error>> {
        let url = format!("{}/remittance", self.url);
        let access_token = self.http_client.get_or_create_token().await?;
        self.account
            .get_account_balance_in_specific_currency(
                url,
                self.environment,
                self.primary_key.clone(),
                currency,
                access_token,
            )
            .await
    }

    /// This operation is used to get the basic information of the account holder
    ///
    /// # Parameters
    /// * 'account_holder_msisdn', the MSISDN of the account holder
    ///
    /// # Returns
    ///
    /// * 'BasicUserInfoJsonResponse'
    pub async fn get_basic_user_info(
        &self,
        account_holder_msisdn: &str,
    ) -> Result<BasicUserInfoJsonResponse, Box<dyn std::error::Error>> {
        let url = format!("{}/remittance", self.url);
        let access_token = self.http_client.get_or_create_token().await?;
        self.account
            .get_basic_user_info(
                url,
                self.environment,
                self.primary_key.clone(),
                account_holder_msisdn,
                access_token,
            )
            .await
    }

    /// This operation is used to get the basic information of the account holder.
    ///
    /// # Parameters
    ///
    /// * 'access_token', the access token of the account holder
    ///
    /// # Returns
    ///
    /// * 'BasicUserInfoJsonResponse'
    pub async fn get_user_info_with_consent(
        &self,
        access_token: String,
    ) -> Result<BasicUserInfoJsonResponse, Box<dyn std::error::Error>> {
        let url = format!("{}/remittance", self.url);
        self.account
            .get_user_info_with_consent(
                url,
                self.environment,
                self.primary_key.clone(),
                access_token,
            )
            .await
    }

    /// this operation is used to validate the status of an account holder.
    ///
    /// # Parameters
    ///
    /// * 'account_holder_id', The MSISDN or email of the account holder
    /// * 'account_holder_type', The type of the account holder.
    ///
    ///
    /// # Returns
    ///
    /// * ()
    pub async fn validate_account_holder_status(
        &self,
        account_holder_id: &str,
        account_holder_type: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}/remittance", self.url);
        let access_token = self.http_client.get_or_create_token().await?;
        self.account
            .validate_account_holder_status(
                url,
                self.environment,
                self.primary_key.clone(),
                account_holder_id,
                account_holder_type,
                access_token,
            )
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;
    use std::env;

    use crate::{MomoRemittance, Party, PartyIdType};

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
    async fn test_transfer() {
        dotenv().ok();
        let url = env::var("MTN_URL").expect("MTN_URL not set");
        let primary_key =
            env::var("MTN_REMITTANCE_PRIMARY_KEY").expect("MTN_REMITTANCE_PRIMARY_KEY not set");
        let secondary_key =
            env::var("MTN_REMITTANCE_SECONDARY_KEY").expect("MTN_REMITTANCE_SECONDARY_KEY not set");
        let api_user = env::var("MTN_API_USER").expect("MTN_API_USER not set");
        let api_key = env::var("MTN_API_KEY").expect("MTN_API_KEY not set");
        let remittance = MomoRemittance::new(
            url,
            Environment::Sandbox,
            api_user,
            api_key,
            primary_key,
            secondary_key,
        );
        let transfer = TransferRequest::new(
            "100".to_string(),
            Currency::EUR,
            Party {
                party_id_type: PartyIdType::MSISDN,
                party_id: "256774290781".to_string(),
            },
            "payer_message".to_string(),
            "payee_note".to_string(),
        );

        let transer_result = remittance.transfer(transfer.clone(), None).await;
        assert!(transer_result.is_ok());
        assert_eq!(transer_result.unwrap().as_string(), transfer.external_id);
    }

    #[tokio::test]
    async fn test_get_transfer_status() {
        dotenv().ok();
        let url = env::var("MTN_URL").expect("MTN_URL not set");
        let primary_key =
            env::var("MTN_REMITTANCE_PRIMARY_KEY").expect("MTN_REMITTANCE_PRIMARY_KEY not set");
        let secondary_key =
            env::var("MTN_REMITTANCE_SECONDARY_KEY").expect("MTN_REMITTANCE_SECONDARY_KEY not set");
        let api_user = env::var("MTN_API_USER").expect("MTN_API_USER not set");
        let api_key = env::var("MTN_API_KEY").expect("MTN_API_KEY not set");
        let remittance = MomoRemittance::new(
            url,
            Environment::Sandbox,
            api_user,
            api_key,
            primary_key,
            secondary_key,
        );
        let transfer = TransferRequest::new(
            "100".to_string(),
            Currency::EUR,
            Party {
                party_id_type: PartyIdType::MSISDN,
                party_id: "256774290781".to_string(),
            },
            "payer_message".to_string(),
            "payee_note".to_string(),
        );
        let transfer_result = remittance.transfer(transfer.clone(), None).await;
        assert!(transfer_result.is_ok());

        let status_result = remittance
            .get_transfer_status(transfer_result.unwrap().as_str())
            .await;
        assert!(status_result.is_ok());
        assert_eq!(status_result.unwrap().status, "SUCCESSFUL");
    }

    #[tokio::test]
    async fn test_get_basic_user_info() {
        dotenv().ok();
        let url = env::var("MTN_URL").expect("MTN_URL not set");
        let primary_key =
            env::var("MTN_REMITTANCE_PRIMARY_KEY").expect("MTN_REMITTANCE_PRIMARY_KEY not set");
        let secondary_key =
            env::var("MTN_REMITTANCE_SECONDARY_KEY").expect("MTN_REMITTANCE_SECONDARY_KEY not set");
        let api_user = env::var("MTN_API_USER").expect("MTN_API_USER not set");
        let api_key = env::var("MTN_API_KEY").expect("MTN_API_KEY not set");
        let remittance = MomoRemittance::new(
            url,
            Environment::Sandbox,
            api_user,
            api_key,
            primary_key,
            secondary_key,
        );
        let basic_user_info = remittance
            .get_basic_user_info("256774290781")
            .await
            .unwrap();
        assert_ne!(basic_user_info.given_name.len(), 0);
    }

    #[tokio::test]
    async fn test_validate_account_holder_status() {
        dotenv().ok();
        let url = env::var("MTN_URL").expect("MTN_URL not set");
        let primary_key =
            env::var("MTN_REMITTANCE_PRIMARY_KEY").expect("MTN_REMITTANCE_PRIMARY_KEY not set");
        let secondary_key =
            env::var("MTN_REMITTANCE_SECONDARY_KEY").expect("MTN_REMITTANCE_SECONDARY_KEY not set");
        let api_user = env::var("MTN_API_USER").expect("MTN_API_USER not set");
        let api_key = env::var("MTN_API_KEY").expect("MTN_API_KEY not set");
        let remittance = MomoRemittance::new(
            url,
            Environment::Sandbox,
            api_user,
            api_key,
            primary_key,
            secondary_key,
        );
        let holder_status_result = remittance
            .validate_account_holder_status("256774290781", "msisdn")
            .await;
        assert!(holder_status_result.is_ok());
    }

    #[tokio::test]
    async fn test_get_account_balance() {
        dotenv().ok();
        let url = env::var("MTN_URL").expect("MTN_URL not set");
        let primary_key =
            env::var("MTN_REMITTANCE_PRIMARY_KEY").expect("MTN_REMITTANCE_PRIMARY_KEY not set");
        let secondary_key =
            env::var("MTN_REMITTANCE_SECONDARY_KEY").expect("MTN_REMITTANCE_SECONDARY_KEY not set");
        let api_user = env::var("MTN_API_USER").expect("MTN_API_USER not set");
        let api_key = env::var("MTN_API_KEY").expect("MTN_API_KEY not set");
        let remittance = MomoRemittance::new(
            url,
            Environment::Sandbox,
            api_user,
            api_key,
            primary_key,
            secondary_key,
        );
        let balance_result = remittance.get_account_balance().await;
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
    async fn test_bc_authorize() {
        dotenv().ok();
        let url = env::var("MTN_URL").expect("MTN_URL not set");
        let api_user = env::var("MTN_API_USER").expect("MTN_API_USER not set");
        let api_key = env::var("MTN_API_KEY").expect("MTN_API_KEY not set");

        let primary_key =
            env::var("MTN_REMITTANCE_PRIMARY_KEY").expect("MTN_REMITTANCE_PRIMARY_KEY not set");
        let secondary_key =
            env::var("MTN_REMITTANCE_SECONDARY_KEY").expect("MTN_REMITTANCE_SECONDARY_KEY not set");

        let remittance = MomoRemittance::new(
            url,
            Environment::Sandbox,
            api_user,
            api_key,
            primary_key,
            secondary_key,
        );
        let bc_authorize_result = remittance.bc_authorize("563607".to_string(), None).await;
        assert!(bc_authorize_result.is_ok());
        assert_ne!(bc_authorize_result.unwrap().auth_req_id.len(), 0);
    }

    #[tokio::test]
    async fn test_create_o_auth_2_token() {
        dotenv().ok();
        let url = env::var("MTN_URL").expect("MTN_URL not set");
        let api_user = env::var("MTN_API_USER").expect("MTN_API_USER not set");
        let api_key = env::var("MTN_API_KEY").expect("MTN_API_KEY not set");

        let primary_key =
            env::var("MTN_REMITTANCE_PRIMARY_KEY").expect("MTN_REMITTANCE_PRIMARY_KEY not set");
        let secondary_key =
            env::var("MTN_REMITTANCE_SECONDARY_KEY").expect("MTN_REMITTANCE_SECONDARY_KEY not set");

        let remittance = MomoRemittance::new(
            url,
            Environment::Sandbox,
            api_user,
            api_key,
            primary_key,
            secondary_key,
        );
        let bc_authorize_result = remittance.bc_authorize("563607".to_string(), None).await;
        assert!(bc_authorize_result.is_ok());
        let auth_req_id = bc_authorize_result.unwrap().auth_req_id;
        let res = remittance.create_o_auth_2_token(auth_req_id).await;
        assert!(res.is_ok());
        assert_ne!(res.unwrap().access_token.len(), 0);
    }

    #[tokio::test]
    async fn test_get_user_info_with_consent() {
        dotenv().ok();
        let url = env::var("MTN_URL").expect("MTN_URL not set");
        let primary_key =
            env::var("MTN_REMITTANCE_PRIMARY_KEY").expect("MTN_REMITTANCE_PRIMARY_KEY not set");
        let secondary_key =
            env::var("MTN_REMITTANCE_SECONDARY_KEY").expect("MTN_REMITTANCE_SECONDARY_KEY not set");
        let api_user = env::var("MTN_API_USER").expect("MTN_API_USER not set");
        let api_key = env::var("MTN_API_KEY").expect("MTN_API_KEY not set");
        let remittance = MomoRemittance::new(
            url,
            Environment::Sandbox,
            api_user,
            api_key,
            primary_key,
            secondary_key,
        );
        let bc_authorize_result = remittance.bc_authorize("563607".to_string(), None).await;
        assert!(bc_authorize_result.is_ok());
        let auth_req_id = bc_authorize_result.unwrap().auth_req_id;
        let res = remittance.create_o_auth_2_token(auth_req_id).await;
        assert!(res.is_ok());
        let user_info_with_consent = remittance
            .get_user_info_with_consent(res.unwrap().access_token)
            .await
            .unwrap();
        assert_ne!(user_info_with_consent.family_name.len(), 0);
    }
}
