use crate::{Balance, BasicUserInfoJsonResponse, Currency, Environment, TokenResponse};

pub struct Account {}

impl Account {
    /// This operation is used to get the balance of the account.
    /// # Parameters
    ///
    /// * 'url', the url of the product to get balance from
    /// * 'environment', the environment of the installation
    /// * 'primary_key', the primary key of the installation
    /// * 'access_token', the access token to be used to make the request
    ///
    /// # Returns
    ///
    /// * 'Balance', the balance
    pub async fn get_account_balance(
        &self,
        url: String,
        environment: Environment,
        primary_key: String,
        access_token: TokenResponse,
    ) -> Result<Balance, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client
            .get(format!("{}/v1_0/account/balance", url))
            .bearer_auth(access_token.access_token)
            .header("X-Target-Environment", environment.to_string())
            .header("Cache-Control", "no-cache")
            .header("Ocp-Apim-Subscription-Key", &primary_key)
            .send()
            .await?;

        if res.status().is_success() {
            let body = res.text().await?;
            let balance: Balance = serde_json::from_str(&body)?;
            Ok(balance)
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                res.text().await?,
            )))
        }
    }

    /// this operation is used to get the balance of an account in a specific currency
    ///
    /// # Parameters
    ///
    /// * 'url', the url of the product to get balance from
    /// * 'currency', Currency of the account to get balance from
    /// * 'environment', the environment of the installation
    /// * 'primary_key', the primary key of the installation
    /// * 'access_token', the access token to be used to make the request
    ///
    /// # Returns
    ///
    /// * 'Balance', the balance
    pub async fn get_account_balance_in_specific_currency(
        &self,
        url: String,
        environment: Environment,
        primary_key: String,
        currency: Currency,
        access_token: TokenResponse,
    ) -> Result<Balance, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client
            .get(format!(
                "{}/v1_0/account/balance/{}",
                url,
                currency.to_string().to_lowercase()
            ))
            .bearer_auth(access_token.access_token)
            .header("X-Target-Environment", environment.to_string())
            .header("Ocp-Apim-Subscription-Key", &primary_key)
            .send()
            .await?;

        if res.status().is_success() {
            let body = res.text().await?;
            let balance: Balance = serde_json::from_str(&body)?;
            Ok(balance)
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                res.text().await?,
            )))
        }
    }

    /// This operation is used to get the basic information of the account holder
    ///
    /// # Parameters
    /// * 'url', the url of the product to get balance from
    /// * 'environment', the environment of the installation
    /// * 'primary_key', the primary key of the installation
    /// * 'account_holder_msisdn', the MSISDN of the account holder
    /// * 'access_token', the access token to be used to make the request
    ///
    /// # Returns
    ///
    /// * 'BasicUserInfoJsonResponse'
    pub async fn get_basic_user_info(
        &self,
        url: String,
        environment: Environment,
        primary_key: String,
        account_holder_msisdn: &str,
        access_token: TokenResponse,
    ) -> Result<BasicUserInfoJsonResponse, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client
            .get(format!(
                "{}/v1_0/accountholder/msisdn/{}/basicuserinfo",
                url, account_holder_msisdn
            ))
            .bearer_auth(access_token.access_token)
            .header("Content-Type", "application/json")
            .header("X-Target-Environment", environment.to_string())
            .header("Ocp-Apim-Subscription-Key", &primary_key)
            .header("Cache-Control", "no-cache")
            .send()
            .await?;

        if res.status().is_success() {
            let body = res.text().await?;
            let basic_user_info: BasicUserInfoJsonResponse = serde_json::from_str(&body)?;
            Ok(basic_user_info)
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                res.text().await?,
            )))
        }
    }

    /// This operation is used to get the basic information of the account holder.
    ///
    /// # Parameters
    ///
    /// * 'url', the url of the product to get balance from
    /// * 'environment', the environment of the installation
    /// * 'primary_key', the primary key of the installation
    /// * 'access_token', the access token of the account holder
    ///
    /// # Returns
    ///
    /// * 'BasicUserInfoJsonResponse'
    pub async fn get_user_info_with_consent(
        &self,
        url: String,
        environment: Environment,
        primary_key: String,
        access_token: String,
    ) -> Result<BasicUserInfoJsonResponse, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client
            .get(format!("{}/oauth2/v1_0/userinfo", url))
            .bearer_auth(access_token)
            .header("X-Target-Environment", environment.to_string())
            .header("Ocp-Apim-Subscription-Key", &primary_key)
            .header("Cache-Control", "no-cache")
            .send()
            .await?;

        if res.status().is_success() {
            let body = res.text().await?;
            let basic_user_info: BasicUserInfoJsonResponse = serde_json::from_str(&body)?;
            Ok(basic_user_info)
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                res.text().await?,
            )))
        }
    }

    /// this operation is used to validate the status of an account holder.
    ///
    /// # Parameters
    ///
    /// * 'url', the url of the product to get balance from
    /// * 'environment', the environment of the installation
    /// * 'primary_key', the primary key of the installation
    /// * 'access_token', the access token of the account holder
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
        url: String,
        environment: Environment,
        primary_key: String,
        account_holder_id: &str,
        account_holder_type: &str,
        access_token: TokenResponse,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client
            .get(format!(
                "{}/v1_0/accountholder/{}/{}/active",
                url, account_holder_type, account_holder_id
            ))
            .bearer_auth(access_token.access_token)
            .header("X-Target-Environment", environment.to_string())
            .header("Ocp-Apim-Subscription-Key", &primary_key)
            .send()
            .await?;

        if res.status().is_success() {
            Ok(())
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                res.text().await?,
            )))
        }
    }
}
