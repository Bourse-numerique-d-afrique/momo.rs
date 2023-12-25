use crate::{
    enums::environment::Environment,
    requests::{refund::Refund, transfer::Transfer},
    responses::{
        account_info::BasicUserInfoJsonResponse, account_info_consent::UserInfoWithConsent,
        bcauthorize_response::BCAuthorizeResponse, oauth2tokenresponse::OAuth2TokenResponse,
        token_response::TokenResponse,
    },
    traits::{account::Account, auth::MOMOAuthorization},
};
use base64::{engine::general_purpose, Engine as _};

use crate::structs::balance::Balance;

pub struct Disbursements {
    pub url: String,
    pub primary_key: String,
    pub secondary_key: String,
    pub environment: Environment,
    pub api_user: String,
    pub api_key: String,
}

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
       deposit operation is used to deposit an amount from the owner’s account to a payee account.
       Status of the transaction can be validated by using the GET /deposit/{referenceId}
       @return Ok(())
    */
    pub async fn deposit_v1(&self) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client
            .post(format!(
                "{}/disbursement/v1_0/deposit",
                self.url
            ))
            .header("Authorization", format!("Basic {}", ""))
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
        let res = client
            .post(format!(
                "{}/disbursement/v2_0/deposit",
                self.url
            ))
            .header("Authorization", format!("Basic {}", ""))
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
        let res = client
            .post(format!(
                "{}/disbursement/v1_0/deposit/{}",
                self.url, "referenceId"
            ))
            .header("Authorization", format!("Basic {}", ""))
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
    pub async fn get_refund_status(&self) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client
            .post(format!(
                "{}/disbursement/v1_0/refund/{}",
                self.url, "referenceId"
            ))
            .header("Authorization", format!("Basic {}", ""))
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
        let res = client
            .post(format!(
                "{}/disbursement/v1_0/transfer/{}",
                self.url, "referenceId"
            ))
            .header("Authorization", format!("Basic {}", ""))
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
    pub async fn refund_v1(&self) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client
            .post(format!(
                "{}/disbursement/v1_0/refund",
                self.url
            ))
            .header("Authorization", format!("Basic {}", ""))
            .header("X-Target-Environment", self.environment.to_string())
            .header("X-Reference-Id", "")
            .header("Cache-Control", "no-cache")
            .body(Refund {
                amount: todo!(),
                currency: todo!(),
                external_id: todo!(),
                payer_message: todo!(),
                payee_note: todo!(),
                reference_id_to_refund: todo!(),
            })
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
    pub async fn refund_v2(&self) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client
            .post(format!(
                "{}/disbursement/v2_0/refund",
                self.url
            ))
            .header("Authorization", format!("Basic {}", ""))
            .header("X-Target-Environment", self.environment.to_string())
            .header("X-Reference-Id", "")
            .header("Cache-Control", "no-cache")
            .body(Refund {
                amount: todo!(),
                currency: todo!(),
                external_id: todo!(),
                payer_message: todo!(),
                payee_note: todo!(),
                reference_id_to_refund: todo!(),
            })
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
    pub async fn transfer(&self) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client
            .post(format!(
                "{}/disbursement/v1_0/transfer",
                self.url
            ))
            .header("Authorization", format!("Basic {}", ""))
            .header("X-Target-Environment", self.environment.to_string())
            .header("X-Reference-Id", "")
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
        &self,
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
        account_holder_id: String,
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
        let authorization = self.encode(&self.primary_key, &self.secondary_key);
        let client = reqwest::Client::new();
        let res = client
            .post(format!("{}/disbursement/token/", self.url))
            .header("Authorization", format!("Basic {}", authorization))
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
        let authorization = self.encode(&self.primary_key, &self.secondary_key);
        let client = reqwest::Client::new();
        let res = client
            .post(format!(
                "{}/disbursement/oauth2/token/",
                self.url
            ))
            .header("Authorization", format!("Basic {}", authorization))
            .header("X-Target-Environment", self.environment.to_string())
            .header("Content-type", "application/x-www-form-urlencoded")
            .header("Cache-Control", "no-cache")
            .send()
            .await?;

        let body = res.text().await?;
        let token_response: OAuth2TokenResponse = serde_json::from_str(&body)?;
        Ok(token_response)
    }

    async fn bc_authorize(&self) -> Result<BCAuthorizeResponse, Box<dyn std::error::Error>> {
        let authorization = self.encode(&self.primary_key, &self.secondary_key);
        let client = reqwest::Client::new();
        let res = client
            .post(format!(
                "{}/disbursement/v1_0/bc-authorize",
                self.url
            ))
            .header("Authorization", format!("Basic {}", authorization))
            .header("X-Target-Environment", self.environment.to_string())
            .header("X-Callback-Url", "callback")
            .header("Content-type", "application/x-www-form-urlencoded")
            .header("Cache-Control", "no-cache")
            .send()
            .await?;

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
