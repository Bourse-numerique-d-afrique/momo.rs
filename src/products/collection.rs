use crate::{traits::{account::Account, auth::MOMOAuthorization},
     responses::{token_response::TokenResponse,
     bcauthorize_response::BCAuthorizeResponse,
     oauth2tokenresponse::OAuth2TokenResponse, account_info::BasicUserInfoJsonResponse,
     account_info_consent::UserInfoWithConsent, invoice::InvoiceResult,
     payment_result::PaymentResult, pre_approval::PreApprovalResult,
     request_to_pay_result::RequestToPayResult},
     requests::{invoice_delete::InvoiceDelete, invoice::InvoiceRequest,
         create_payment::CreatePayment,
          request_to_pay::RequestToPay,
           pre_approval::PreApproval,
            delivery_notification::DeliveryNotification},
             enums::environment::Environment};
use base64::{engine::general_purpose, Engine as _};

use crate::structs::{balance::Balance, party::Party};
use rusqlite::{params, Connection, Result};
use chrono::{DateTime, Utc};



pub struct Collection {
    pub url: String,
    pub primary_key: String,
    pub secondary_key: String,
    pub environment: Environment,
    pub api_user: String,
    pub api_key: String,
}


impl Collection {
    /*
        create a new instance of the collection product
        @param url
        @return Collection
    
     */
    pub fn new(url: String, environment: Environment, api_user: String, api_key: String, primary_key: String, secondary_key: String) -> Collection {
        Collection {
            url,
            primary_key,
            secondary_key,
            environment,
            api_user,
            api_key,
        }
    }


    /*
        create collection access tokens if they do not exist in the database    
        @return Ok(())
    */
    fn create_access_tokens_if_not_exists(&self) -> Result<(), Box<dyn std::error::Error>> {
        let conn = Connection::open("mtn_access_tokens.db")?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS access_tokens (
                id INTEGER PRIMARY KEY,
                access_token TEXT NOT NULL,
                token_type TEXT NOT NULL,
                expires_in INTEGER NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            params![],
        )?;

        Ok(())
    }


    /*
        This operation is used to insert an access token into the database
        @return Ok(())
     */
    fn insert_access_token(&self, access_token: &str, token_type: &str, expires_in: i32) -> Result<(), Box<dyn std::error::Error>> {
        let conn = Connection::open("mtn_access_tokens.db")?;
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
    fn get_valid_access_token(&self) -> Result<TokenResponse, Box<dyn std::error::Error>> {
        let conn = Connection::open("mtn_access_tokens.db")?;
        let mut stmt = conn.prepare("SELECT * FROM access_tokens ORDER BY created_at DESC LIMIT 1")?;
        let access_token_iter = stmt.query_map(params![], |row| {
            let created_at: String = row.get(4)?;
            let created_at = created_at.parse::<DateTime<Utc>>().map_err(|_| rusqlite::Error::QueryReturnedNoRows)?;
            let now = Utc::now();
            let duration = now.signed_duration_since(created_at);
            let duration = duration.num_seconds();
            if duration > 3600 {
                return Err(rusqlite::Error::QueryReturnedNoRows);
            }
            Ok(TokenResponse {
                access_token: row.get(1)?,
                token_type: row.get(2)?,
                expires_in: row.get(3)?,
            })
        })?;

        let access_token = access_token_iter.map(|x| x.unwrap()).collect::<Vec<TokenResponse>>();
        Ok(access_token[0].clone())
    }



    /*
        This operation is used to delete an invoice. The ReferenceId is associated with the invoice to be cancelled
        @return InvoiceDelete
    
     */
    pub async fn cancel_invoice(&self, external_id: &str, access_token: &str) -> Result<InvoiceDelete, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client.delete(format!("{}/collection/v2_0/invoice/{}", self.url, "invoice_id"))
        .header("Authorization", format!("Basic {}", ""))
        .header("X-Target-Environment", self.environment.to_string())
        .header("X-Reference-Id", "")
        .header("X-Callback-Url", "")
        .header("Cache-Control", "no-cache")
        .body(InvoiceDelete{external_id: "external_id".to_string()})
        .send().await?;
        let body = res.text().await?;
        let response: InvoiceDelete = serde_json::from_str(&body)?;
        Ok(response)
    }

    /*
        A merchant may use this in order to create an invoice that can be paid by an intended payer via any channel at a later stage.
        @return Ok(())
    
     */
    pub async fn create_invoice(&self, external_id: &str, access_token: &str) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client.post(format!("{}/collection/v2_0/invoice", self.url))
        .header("Authorization", format!("Basic {}", ""))
        .header("X-Target-Environment", self.environment.to_string())
        .header("X-Reference-Id", "")
        .header("X-Callback-Url", "")
        .header("Cache-Control", "no-cache")
        .body(
            InvoiceRequest {
                amount: "amount".to_string(),
                currency: "currency".to_string(),
                external_id: "external_id".to_string(),
                validity_duration: "validity_duration".to_string(),
                description: "description".to_string(),
                intended_payer: Party {
                    party_id_type: todo!(),
                    party_id: todo!(),
                },
                payee: Party {
                    party_id_type: todo!(),
                    party_id: todo!(),
                },
            }
        )
        .send().await?;

        let response = res.text().await?;
        Ok(())
    }


    /*
        Making it possible to perform payments via the partner gateway. This may be used to pay for external bills or to perform air-time top-ups.
        @return Ok(())
     */
    pub async fn create_payments(&self, external_id: &str, access_token: &str) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client.post(format!("{}/collection/v2_0/payment", self.url))
        .header("Authorization", format!("Basic {}", ""))
        .header("X-Target-Environment", self.environment.to_string())
        .header("X-Reference-Id", "")
        .header("X-Callback-Url", "")
        .header("Cache-Control", "no-cache")
        .body(CreatePayment{
            external_transaction_id: todo!(),
            money: todo!(),
            customer_reference: todo!(),
            service_provider_user_name: todo!(),
            coupon_id: todo!(),
            product_id: todo!(),
            product_offering_id: todo!(),
            receiver_message: todo!(),
            sender_note: todo!(),
            max_number_of_retries: todo!(),
            include_sender_charges: todo!(),
        })
        .send().await?;

        let response = res.text().await?;

        
        Ok(())
    }


    /*
        This operation is used to get the status of an invoice. X-Reference-Id that was passed in the post is used as reference to the request
        @return InvoiceResult
     */
    async fn get_invoice_status(&self, invoice_id: String, external_id: &str, access_token: &str) -> Result<InvoiceResult, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client.get(format!("{}/collection/v2_0/invoice/{}", self.url, invoice_id))
        .header("Authorization", format!("Basic {}", ""))
        .header("X-Target-Environment", self.environment.to_string())
        .header("Cache-Control", "no-cache")
        .send().await?;

        let body = res.text().await?;
        let invoice_status: InvoiceResult = serde_json::from_str(&body)?;
        Ok(invoice_status)
    }


    /*
        This operation is used to get the status of a Payment. X-Reference-Id that was passed in the post is used as reference to the request
        @return PaymentResult
     */
    async fn get_payment_status(&self, payment_id: String, external_id: &str, access_token: &str) -> Result<PaymentResult, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client.get(format!("{}/collection/v2_0/payment/{}", self.url, payment_id))
        .header("Authorization", format!("Basic {}", ""))
        .header("X-Target-Environment", self.environment.to_string())
        .header("Cache-Control", "no-cache")
        .send().await?;

        let body = res.text().await?;
        let payment_status: PaymentResult = serde_json::from_str(&body)?;
        Ok(payment_status)
    }


    /*
    This operation is used to get the status of a pre-approval. X-Reference-Id that was passed in the post is used as reference to the request.
     */
    async fn get_pre_approval_status(&self, pre_approval_id: String, external_id: &str, access_token: &str) -> Result<PreApprovalResult, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client.get(format!("{}/collection/v2_0/preapproval/{}", self.url, pre_approval_id))
        .header("Authorization", format!("Basic {}", ""))
        .header("X-Target-Environment", self.environment.to_string())
        .header("Cache-Control", "no-cache")
        .send().await?;

        let body = res.text().await?;
        let pre_approval_status: PreApprovalResult = serde_json::from_str(&body)?;
        Ok(pre_approval_status)
    }


    /*
        Preapproval operation is used to create a pre-approval.
        @return Ok(())
     */
    pub async fn pre_approval(&self, external_id: &str, access_token: &str) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client.post(format!("{}/collection/v2_0/preapproval", self.url))
        .header("Authorization", format!("Basic {}", ""))
        .header("X-Target-Environment", self.environment.to_string())
        .header("Cache-Control", "no-cache")
        .body(PreApproval{
            payer: todo!(),
            payer_currency: todo!(),
            payer_message: todo!(),
            validity_time: todo!(),
        })
        .send().await?;

        
        let response = res.text().await?;
        Ok(())
    }

    /*
        This operation is used to request a payment from a consumer (Payer). The payer will be asked to authorize the payment.
        The transaction will be executed once the payer has authorized the payment.
        The requesttopay will be in status PENDING until the transaction is authorized or declined by the payer or it is timed out by the system.
        Status of the transaction can be validated by using the GET /requesttopay/<resourceId>
        @param request
        @param external_id
        @param access_token access token obtained from the create_access_token method
        @return Ok(())
     */
    pub async fn request_to_pay(&self, request: RequestToPay, external_id: &str, access_token: &str) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client.post(format!("{}/collection/v1_0/requesttopay", self.url))
        .bearer_auth(access_token)
        .header("X-Target-Environment", self.environment.to_string())
        .header("Cache-Control", "no-cache")
        .header("Content-Type", "application/json")
        .header("X-Reference-Id", external_id)
        .header("Ocp-Apim-Subscription-Key", &self.primary_key)
        .body(request)
        .send().await?;

        if res.status().is_success() {
            Ok(())
        } else {
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, res.text().await?)))
        }

    }

    /*
        This operation is used to send additional Notification to an End User.
        @return Ok(())
     */
    pub async fn request_to_pay_delivery_notification(&self, external_id: &str, access_token: &str) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client.post(format!("{}/collection/v1_0/requesttopay/{}/deliverynotification", self.url, ""))
        .header("Authorization", format!("Basic {}", ""))
        .header("X-Target-Environment", self.environment.to_string())
        .header("notificationMessage", "")
        .header("Language", "")
        .header("Cache-Control", "no-cache")
        .body(
            DeliveryNotification{
                notification_message: todo!(),
            }
        )
        .send().await?;

        let response = res.text().await?;
        Ok(())
    }

    /*
        This operation is used to get the status of a request to pay.
        X-Reference-Id that was passed in the post is used as reference to the request.
     */
    pub async fn request_to_pay_transaction_status(&self, external_id: &str, access_token: &str) -> Result<RequestToPayResult, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client.get(format!("{}/collection/v1_0/requesttopay/{}", self.url, "payment_id"))
        .header("Authorization", format!("Basic {}", ""))
        .header("X-Target-Environment", self.environment.to_string())
        .header("Cache-Control", "no-cache")
        .send().await?;

        let response = res.text().await?;
        let request_to_pay_result: RequestToPayResult = serde_json::from_str(&response)?;

        Ok(request_to_pay_result)
    }


    /*
        This operation is used to get the status of a request to withdraw.
        X-Reference-Id that was passed in the post is used as reference to the request.
     */
    pub async fn request_to_withdraw_transaction_status(&self, external_id: &str, access_token: &str) -> Result<RequestToPayResult, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client.get(format!("{}/collection/v1_0/requesttowithdraw/{}", self.url, "payment_id"))
        .header("Authorization", format!("Basic {}", ""))
        .header("X-Target-Environment", self.environment.to_string())
        .header("Cache-Control", "no-cache")
        .send().await?;

        let response = res.text().await?;
        let request_to_pay_result: RequestToPayResult = serde_json::from_str(&response)?;

        Ok(request_to_pay_result)
    }

    /*
    This operation is used to request a withdrawal (cash-out) from a consumer (Payer).
    The payer will be asked to authorize the withdrawal.
    The transaction will be executed once the payer has authorized the withdrawal

    @return Ok(())
     */
    pub async fn request_to_withdraw_v1(&self, external_id: &str, access_token: &str) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client.post(format!("{}/collection/v1_0/requesttowithdraw", self.url))
        .header("Authorization", format!("Basic {}", ""))
        .header("X-Target-Environment", self.environment.to_string())
        .header("X-Callback-Url", "")
        .header("Cache-Control", "no-cache")
        .body(
            RequestToPay{
                amount: todo!(),
                currency: todo!(),
                external_id: todo!(),
                payer: todo!(),
                payer_message: todo!(),
                payee_note: todo!(),
            }
        )
        .send().await?;

        let response = res.text().await?;
        Ok(())
    }

    /*
    This operation is used to request a withdrawal (cash-out) from a consumer (Payer).
    The payer will be asked to authorize the withdrawal.
    The transaction will be executed once the payer has authorized the withdrawal

    @return Ok(())
    
     */
    pub async fn request_to_withdraw_v2(&self, external_id: &str, access_token: &str) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client.post(format!("{}/collection/v2_0/requesttowithdraw", self.url))
        .header("Authorization", format!("Basic {}", ""))
        .header("X-Target-Environment", self.environment.to_string())
        .header("X-Callback-Url", "")
        .header("Cache-Control", "no-cache")
        .body(
            RequestToPay{
                amount: todo!(),
                currency: todo!(),
                external_id: todo!(),
                payer: todo!(),
                payer_message: todo!(),
                payee_note: todo!(),
            }
        )
        .send().await?;

        let response = res.text().await?;
        Ok(())
    }
}


impl Account for Collection{
    async fn get_account_balance(&self, external_id: &str, access_token: &str) -> Result<Balance, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client.get(format!("{}/collection/v1_0/account/balance", self.url))
        .header("Authorization", format!("Basic {}", ""))
        .header("X-Target-Environment", self.environment.to_string())
        .header("Cache-Control", "no-cache").send().await?;

        let body = res.text().await?;
        let balance: Balance = serde_json::from_str(&body)?;
        Ok(balance)

    }

    async fn get_account_balance_in_specific_currency(&self, currency: String, external_id: &str, access_token: &str) -> Result<Balance, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client.get(format!("{}/collection/v1_0/account/balance/currency", self.url))
        .header("Authorization", format!("Basic {}", ""))
        .header("X-Target-Environment", self.environment.to_string())
        .header("Cache-Control", "no-cache").send().await?;
        let body = res.text().await?;
        let balance: Balance = serde_json::from_str(&body)?;
        Ok(balance)
    }

    async fn get_basic_user_info(&self, external_id: &str, access_token: &str) -> Result<BasicUserInfoJsonResponse, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client.get(format!("{}/collection/v1_0/accountholder/msisdn/{}/basicuserinfo", self.url, "accountHolderMSISDN"))
        .header("Authorization", format!("Basic {}", ""))
        .header("X-Target-Environment", self.environment.to_string())
        .header("Cache-Control", "no-cache").send().await?;
        let body = res.text().await?;
        let basic_user_info: BasicUserInfoJsonResponse = serde_json::from_str(&body)?;
        Ok(basic_user_info)
    }

    async fn get_user_info_with_consent(&self, external_id: &str, access_token: &str) -> Result<UserInfoWithConsent, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client.get(format!("{}/collection/oauth2/v1_0/userinfo", self.url))
        .header("Authorization", format!("Basic {}", ""))
        .header("X-Target-Environment", self.environment.to_string())
        .header("Cache-Control", "no-cache").send().await?;
        let body = res.text().await?;
        let basic_user_info: UserInfoWithConsent = serde_json::from_str(&body)?;
        Ok(basic_user_info)

    }

    async fn validate_account_holder_status(&self, account_holder_id: String, external_id: &str, access_token: &str) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client.get(format!("{}/collection/v1_0/accountholder/msisdn/{}/basicuserinfo", self.url, "accountHolderMSISDN"))
        .header("Authorization", format!("Basic {}", ""))
        .header("X-Target-Environment", self.environment.to_string())
        .header("Cache-Control", "no-cache").send().await?;
        let body = res.text().await?;
        let basic_user_info: BasicUserInfoJsonResponse = serde_json::from_str(&body)?;
        Ok(())


    }
}


impl MOMOAuthorization for Collection {
    async fn create_access_token(&self) -> Result<TokenResponse, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client.post(format!("{}/collection/token/", self.url))
        .basic_auth(self.api_user.to_string(), Some(self.api_key.to_string()))
        .header("Cache-Control", "no-cache")
        .header("Content-type", "application/x-www-form-urlencoded")
        .header("Ocp-Apim-Subscription-Key", &self.primary_key)
        .header("Content-Length", "0")
        .body("")
        .send().await?;

        let body = res.text().await?;
        let token_response: TokenResponse = serde_json::from_str(&body)?;
        Ok(token_response)
    }

    async fn create_o_auth_2_token(&self, access_token: &str) -> Result<OAuth2TokenResponse, Box<dyn std::error::Error>> {
        let authorization = self.encode(&self.primary_key, &self.secondary_key);
        let client = reqwest::Client::new();
        let res = client.post(format!("{}/collection/oauth2/token/", self.url))
        .bearer_auth(authorization)
        .header("X-Target-Environment", self.environment.to_string())
        .header("Content-type", "application/x-www-form-urlencoded")
        .header("Cache-Control", "no-cache")
        .header("Ocp-Apim-Subscription-Key", &self.primary_key)
        .body("")
        .send().await?;

        let body = res.text().await?;
        let token_response: OAuth2TokenResponse = serde_json::from_str(&body)?;
        Ok(token_response)
    }

    async fn bc_authorize(&self, access_token: &str) -> Result<BCAuthorizeResponse, Box<dyn std::error::Error>> {
        let authorization = self.encode(&self.primary_key, &self.secondary_key);
        let client = reqwest::Client::new();
        let res = client.post(format!("{}/collection/v1_0/bc-authorize", self.url))
        .basic_auth(self.api_user.to_string(), Some(self.api_key.to_string()))
        .header("X-Target-Environment", self.environment.to_string())
        .header("X-Callback-Url", "callback")
        .header("Content-type", "application/x-www-form-urlencoded")
        .header("Content-Length", "0")
        .header("Cache-Control", "no-cache")
        .send().await?;


        let body: String = res.text().await?;
        let token_response: BCAuthorizeResponse = serde_json::from_str(&body)?;
        Ok(token_response)
    }

    fn encode(&self, user_id: &str, user_api_key: &str) -> String {
        let concatenated_str = format!("{}:{}", user_id, user_api_key);
        let encoded_str = general_purpose::STANDARD.encode(&concatenated_str);
        encoded_str
    }
}


#[cfg(test)]  
mod tests {
    use super::*;
    use crate::enums::currency::Currency;
    use crate::products::collection::Collection;
    use crate::traits::{account::Account, auth::MOMOAuthorization};
    use crate::responses::{token_response::TokenResponse, bcauthorize_response::BCAuthorizeResponse, oauth2tokenresponse::OAuth2TokenResponse, account_info::BasicUserInfoJsonResponse, account_info_consent::UserInfoWithConsent};
    use crate::requests::{invoice_delete::InvoiceDelete, invoice::InvoiceRequest, create_payment::CreatePayment, request_to_pay::RequestToPay, pre_approval::PreApproval, delivery_notification::DeliveryNotification};
    use crate::structs::{balance::Balance, party::Party};
    use dotenv::dotenv;
    use std::env;

    #[tokio::test]
    async fn test_create_token() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");
        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let collection = Collection::new(mtn_url, Environment::Sandbox, "33e32162-3ca5-43fa-a21c-db44d0c704f4".to_string(), "a56bed3d90b440409a838adc39049d51".to_string(), primary_key, secondary_key);
        let token: TokenResponse = collection.create_access_token().await.expect("Error creating token");
        assert!(token.access_token.len() > 0);

    }

    #[tokio::test]
    async fn test_request_payment(){
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let collection = Collection::new(mtn_url, Environment::Sandbox, "33e32162-3ca5-43fa-a21c-db44d0c704f4".to_string(), "a56bed3d90b440409a838adc39049d51".to_string(), primary_key, secondary_key);
        let external = uuid::Uuid::new_v4().to_string();
        let token: TokenResponse = collection.create_access_token().await.expect("Error creating token");
        assert!(token.access_token.len() > 0);
        
        

        let res = collection.request_to_pay(RequestToPay{
            amount: "100".to_string(),
            currency: Currency::EUR, // The currency used in Sandbox is EUR
            external_id: external.clone(),
            payer: Party {
                party_id_type: "MSISDN".to_string(),
                party_id: "+242064818006".to_string(),
            },
            payer_message: "test_payer_message".to_string(),
            payee_note: "test_payee_note".to_string(),
        }, &external, &token.access_token).await.expect("Error requesting payment");

        assert_eq!(res, ());

    }

    

}
