//! Collection Product
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

use crate::{traits::{account::Account, auth::MOMOAuthorization}, InvoiceId, PaymentId, TransactionId, WithdrawId, Currency, TokenResponse, Environment, InvoiceDeleteRequest, InvoiceRequest, CreatePaymentRequest, InvoiceResult, PaymentResult, PreApprovalResult, PreApprovalRequest, RequestToPay, DeliveryNotificationRequest, RequestToPayResult, BasicUserInfoJsonResponse, OAuth2TokenResponse, BCAuthorizeResponse, AccessTokenRequest, AccessType, BcAuthorizeRequest, Balance};
use chrono::Utc;
use once_cell::sync::Lazy;
use tokio::sync::Mutex;
use tokio::task;


/// # Collection
/// This product provides a way to request payments from a customer.
/// # Example
pub struct Collection {
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


impl Collection {
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
        This operation is used to delete an invoice. The ReferenceId is associated with the invoice to be cancelled
        @param invoice_id The invoice id to be cancelled
        @param callback_url The callback url to be used by the MOMO Core API to notify the merchant of the invoice status
        @return Ok(())
    
     */
    pub async fn cancel_invoice(&self, invoice_id: &str, callback_url: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let mut req = client.delete(format!("{}/collection/v2_0/invoice/{}", self.url, invoice_id))
        .bearer_auth(access_token.access_token)
        .header("Content-Type", "application/json")
        .header("X-Target-Environment", self.environment.to_string())
        .header("X-Reference-Id", uuid::Uuid::new_v4().to_string())
        .header("Ocp-Apim-Subscription-Key", &self.primary_key)
        .body(InvoiceDeleteRequest{external_id: invoice_id.to_string()});

        if let Some(callback_url) = callback_url {
            if !callback_url.is_empty() {
                req = req.header("X-Callback-Url", callback_url);
            }
        }
        
        let res = req.send().await?;

        if res.status().is_success() {
            Ok(())
        }else {
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, res.text().await?)))
        }
    }

    /*
        A merchant may use this in order to create an invoice that can be paid by an intended payer via any channel at a later stage.
        @param invoice The invoice to be created on the MOMO Core API
        @param callback_url The callback url to be used by the MOMO Core API to notify the merchant of the invoice status
        @return InvoiceId, this is the external_id of the invoice
    
     */
    pub async fn create_invoice(&self, invoice: InvoiceRequest, callback_url: Option<&str>) -> Result<InvoiceId, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let mut req = client.post(format!("{}/collection/v2_0/invoice", self.url))
        .bearer_auth(access_token.access_token)
        //.header("X-Callback-Url", callback_url.unwrap_or(""))
        .header("X-Reference-Id", &invoice.external_id)
        .header("X-Target-Environment", self.environment.to_string())
        .header("Content-Type", "application/json")
        .header("Ocp-Apim-Subscription-Key", &self.primary_key)
        .body(invoice.clone());
        

        if let Some(callback_url) = callback_url {
            if !callback_url.is_empty() {
                req = req.header("X-Callback-Url", callback_url);
            }
        }
        
        let res = req.send().await?;


        if res.status().is_success() {
            Ok(InvoiceId(invoice.external_id))
        }else {
            let res_clone = res.text().await?;
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, res_clone)))
        }


    }


    /*
        Making it possible to perform payments via the partner gateway. This may be used to pay for external bills or to perform air-time top-ups.
        @param payment The payment to be created on the MOMO Core API
        @param callback_url The callback url to be used by the MOMO Core API to notify the merchant of the payment status
        @return PaymentId, this is the external_id of the payment
     */
    pub async fn create_payments(&self, payment: CreatePaymentRequest, callback_url: Option<&str>) -> Result<PaymentId, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let mut req = client.post(format!("{}/collection/v2_0/payment", self.url))
        .bearer_auth(access_token.access_token)
        .header("X-Target-Environment", self.environment.to_string())
        .header("X-Reference-Id", &payment.external_transaction_id)
        .header("Cache-Control", "no-cache")
        .header("Ocp-Apim-Subscription-Key", &self.primary_key)
        .body(payment.clone());

        if let Some(callback_url) = callback_url {
            if !callback_url.is_empty() {
                req = req.header("X-Callback-Url", callback_url);
            }
        }
        
        let res = req.send().await?;

        if res.status().is_success() {
            Ok(PaymentId(payment.external_transaction_id))
        }else {
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, res.text().await?)))
        }
    }


    /*
        This operation is used to get the status of an invoice. X-Reference-Id that was passed in the post is used as reference to the request
        @param invoice_id The invoice id to be cancelled
        @return InvoiceResult
     */
    #[allow(dead_code)]
    async fn get_invoice_status(&self, invoice_id: String) -> Result<InvoiceResult, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client.get(format!("{}/collection/v2_0/invoice/{}", self.url, invoice_id))
        .bearer_auth(access_token.access_token)
        .header("X-Target-Environment", self.environment.to_string())
        .header("Ocp-Apim-Subscription-Key", &self.primary_key)
        .send().await?;


        
        if res.status().is_success() {
            let body = res.text().await?;
            let invoice_status: InvoiceResult = serde_json::from_str(&body)?;
            Ok(invoice_status)
        }else {
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, res.text().await?)))
        }
    }


    /*
        This operation is used to get the status of a Payment. X-Reference-Id that was passed in the post is used as reference to the request
        @param payment_id The payment id to be cancelled
        @return PaymentResult
     */
    #[allow(dead_code)]
    async fn get_payment_status(&self, payment_id: String) -> Result<PaymentResult, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client.get(format!("{}/collection/v2_0/payment/{}", self.url, payment_id))
        .bearer_auth(access_token.access_token)
        .header("X-Target-Environment", self.environment.to_string())
        .header("Ocp-Apim-Subscription-Key", &self.primary_key)
        .send().await?;

        if res.status().is_success() {
            let body = res.text().await?;
            let payment_status: PaymentResult = serde_json::from_str(&body)?;
            Ok(payment_status)
        }else {
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, res.text().await?)))
        }
    }



    /*
    This operation is used to get the status of a pre-approval. X-Reference-Id that was passed in the post is used as reference to the request.
    @param pre_approval_id The pre-approval id to be cancelled
    @return PreApprovalResult
     */
    #[allow(dead_code)]
    async fn get_pre_approval_status(&self, pre_approval_id: String) -> Result<PreApprovalResult, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client.get(format!("{}/collection/v2_0/preapproval/{}", self.url, pre_approval_id))
        .bearer_auth(access_token.access_token)
        .header("X-Target-Environment", self.environment.to_string())
        .header("Cache-Control", "no-cache")
        .header("Ocp-Apim-Subscription-Key", &self.primary_key)
        .send().await?;

        if res.status().is_success() {
            let body = res.text().await?;
            let pre_approval_status: PreApprovalResult = serde_json::from_str(&body)?;
            Ok(pre_approval_status)
        }else {
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, res.text().await?)))
        }
    }


    /*
        Preapproval operation is used to create a pre-approval.
        @param preaproval The pre-approval to be created on the MOMO Core API
        @return external_id
     */
    pub async fn pre_approval(&self, preaproval: PreApprovalRequest) -> Result<String, Box<dyn std::error::Error>> {
        let external_id = uuid::Uuid::new_v4().to_string();
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client.post(format!("{}/collection/v2_0/preapproval", self.url))
        .bearer_auth(access_token.access_token)
        .header("X-Target-Environment", self.environment.to_string())
        .header("Cache-Control", "no-cache")
        .header("Content-Type", "application/json")
        .header("X-Reference-Id", &external_id)
        .header("Ocp-Apim-Subscription-Key", &self.primary_key)
        .body(preaproval)
        .send().await?;

        if res.status().is_success() {
            Ok(external_id)
        }else {
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, res.text().await?)))
        }
    }

    /*
        This operation is used to request a payment from a consumer (Payer). The payer will be asked to authorize the payment.
        The transaction will be executed once the payer has authorized the payment.
        The requesttopay will be in status PENDING until the transaction is authorized or declined by the payer or it is timed out by the system.
        Status of the transaction can be validated by using the GET /requesttopay/<resourceId>
        @param request
        @return TransactionId (external_id)
     */
    pub async fn request_to_pay(&self, request: RequestToPay) -> Result<TransactionId, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client.post(format!("{}/collection/v1_0/requesttopay", self.url))
        .bearer_auth(access_token.access_token)
        .header("X-Target-Environment", self.environment.to_string())
        .header("Cache-Control", "no-cache")
        .header("Content-Type", "application/json")
        .header("X-Reference-Id", &request.external_id)
        .header("Ocp-Apim-Subscription-Key", &self.primary_key)
        .body(request.clone())
        .send().await?;

        if res.status().is_success() {
            Ok(TransactionId(request.external_id))
        } else {
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, res.text().await?)))
        }

    }

    /*
        This operation is used to send additional Notification to an End User.
        @param external_id, this is the external_id of the request to pay
        @param notification, this is the notification message to be sent to the end user
        @return Ok(())
     */
    pub async fn request_to_pay_delivery_notification(&self, external_id: &str, notification: DeliveryNotificationRequest) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client.post(format!("{}/collection/v1_0/requesttopay/{}/deliverynotification", self.url, external_id))
        .bearer_auth(access_token.access_token)
        .header("X-Target-Environment", self.environment.to_string())
        .header("notificationMessage", &notification.notification_message)
        .header("Language", "")
        .header("Cache-Control", "no-cache")
        .header("Ocp-Apim-Subscription-Key", &self.primary_key)
        .body(notification)
        .send().await?;

        if res.status().is_success() {
            Ok(())
        } else {
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, res.text().await?)))
        }
    }

    /*
        This operation is used to get the status of a request to pay.
        X-Reference-Id that was passed in the post is used as reference to the request.
        @param payment_id The payment id to be cancelled, external_id of the request to pay
        @return RequestToPayResult, this is the result of the request to pay
     */
    pub async fn request_to_pay_transaction_status(&self, payment_id: &str) -> Result<RequestToPayResult, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client.get(format!("{}/collection/v1_0/requesttopay/{}", self.url, payment_id))
        .bearer_auth(access_token.access_token)
        .header("X-Target-Environment", self.environment.to_string())
        .header("Cache-Control", "no-cache")
        .header("Ocp-Apim-Subscription-Key", &self.primary_key)
        .send().await?;

        if res.status().is_success() {
            let body = res.text().await?;
            let request_to_pay_result: RequestToPayResult = serde_json::from_str(&body)?;
            Ok(request_to_pay_result)
        } else {
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, res.text().await?)))
        }
    }


    /*
        This operation is used to get the status of a request to withdraw.
        X-Reference-Id that was passed in the post is used as reference to the request.
        @param payment_id The payment id to be cancelled, external_id of the request to pay
        @return RequestToPayResult, this is the result of the request to pay
     */
    pub async fn request_to_withdraw_transaction_status(&self, payment_id: &str) -> Result<RequestToPayResult, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client.get(format!("{}/collection/v1_0/requesttowithdraw/{}", self.url, payment_id))
        .bearer_auth(access_token.access_token)
        .header("X-Target-Environment", self.environment.to_string())
        .header("Ocp-Apim-Subscription-Key", &self.primary_key)
        .send().await?;

        if res.status().is_success() {
            let body = res.text().await?;
            let request_to_pay_result: RequestToPayResult = serde_json::from_str(&body)?;
            Ok(request_to_pay_result)
        } else {
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, res.text().await?)))
        }
    }

    /*
    This operation is used to request a withdrawal (cash-out) from a consumer (Payer).
    The payer will be asked to authorize the withdrawal.
    The transaction will be executed once the payer has authorized the withdrawal
    @param request, this is the request to withdraw
    @param callback_url, this is the callback url to be used by the MOMO Core API to notify the merchant of the request to withdraw status

    @return WithdrawId, this is the external_id of the request to withdraw
     */
    pub async fn request_to_withdraw_v1(&self, request: RequestToPay, callback_url: Option<&str>) -> Result<WithdrawId, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let mut req = client.post(format!("{}/collection/v1_0/requesttowithdraw", self.url))
        .bearer_auth(access_token.access_token)
        .header("X-Target-Environment", self.environment.to_string())
        .header("X-Reference-Id", &request.external_id)
        .header("Ocp-Apim-Subscription-Key", &self.primary_key)
        .header("Content-Type", "application/json")
        .body(request.clone());


        if let Some(callback_url) = callback_url {
            if !callback_url.is_empty() {
                req = req.header("X-Callback-Url", callback_url);
            }
        }
        
        let res = req.send().await?;
    
        if res.status().is_success() {
            Ok(WithdrawId(request.external_id))
        } else {
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, res.text().await?)))
        }
    }

    /*
    This operation is used to request a withdrawal (cash-out) from a consumer (Payer).
    The payer will be asked to authorize the withdrawal.
    The transaction will be executed once the payer has authorized the withdrawal
    @param request, this is the request to withdraw
    @param callback_url, this is the callback url to be used by the MOMO Core API to notify the merchant of the request to withdraw status

    @return WithdrawId, this is the external_id of the request to withdraw
     */
    pub async fn request_to_withdraw_v2(&self, request: RequestToPay, callback_url: Option<&str>) -> Result<WithdrawId, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let mut req = client.post(format!("{}/collection/v2_0/requesttowithdraw", self.url))
        .bearer_auth(access_token.access_token)
        .header("X-Target-Environment", self.environment.to_string())
        .header("X-Reference-Id", &request.external_id) 
        .header("Ocp-Apim-Subscription-Key", &self.primary_key)
        .header("Content-Type", "application/json")
        .body(request.clone());

        if let Some(callback_url) = callback_url {
            if !callback_url.is_empty() {
                req = req.header("X-Callback-Url", callback_url);
            }
        }
        
        let res = req.send().await?;

        if res.status().is_success() {
            Ok(WithdrawId(request.external_id))
        } else {
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, res.text().await?)))
        }
    }
}


impl Account for Collection{
    /*
        This operation is used to get the balance of the account.
        @return Balance
     */
    async fn get_account_balance(&self) -> Result<Balance, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client.get(format!("{}/collection/v1_0/account/balance", self.url))
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
        This operation is used to get the balance of an account in a specific currency.
        @param currency The currency of the account balance to be retrieved
        @return Balance
     */
    async fn get_account_balance_in_specific_currency(&self, currency: Currency) -> Result<Balance, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client.get(format!("{}/collection/v1_0/account/balance/{}", self.url, currency.to_string().to_lowercase()))
        .bearer_auth(access_token.access_token)
        .header("X-Target-Environment", self.environment.to_string())
        .header("Cache-Control", "no-cache")
        .header("Ocp-Apim-Subscription-Key", &self.secondary_key)
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
        This operation is used to get the basic information of the account holder.
        @param account_holder_msisdn The MSISDN of the account holder
        @return BasicUserInfoJsonResponse
     */
    async fn get_basic_user_info(&self, account_holder_msisdn: &str) -> Result<BasicUserInfoJsonResponse, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client.get(format!("{}/collection/v1_0/accountholder/msisdn/{}/basicuserinfo", self.url, account_holder_msisdn))
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
        This operation is used to get the basic information of the account holder.
        @param access_token The access token of the account holder
        @return BasicUserInfoJsonResponse
     */
    async fn get_user_info_with_consent(&self, access_token: String) -> Result<BasicUserInfoJsonResponse, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client.get(format!("{}/collection/oauth2/v1_0/userinfo", self.url))
        .bearer_auth(access_token)
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
        This operation is used to validate the status of an account holder.
        @param account_holder_id The MSISDN or email of the account holder
        @param account_holder_type The type of the account holder
        @return Ok(())
     */
    async fn validate_account_holder_status(&self, account_holder_id: &str, account_holder_type: &str) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.get_valid_access_token().await?;
        let res = client.get(format!("{}/collection/v1_0/accountholder/{}/{}/active", self.url, account_holder_type.to_lowercase(), account_holder_id.to_lowercase()))
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


impl MOMOAuthorization for Collection {
    /*
        This operation is used to create an access token.
        @return TokenResponse
     */
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


        if res.status().is_success() {
            let body = res.text().await?;
            let token_response: TokenResponse = serde_json::from_str(&body)?;
            let cloned = token_response.clone();
            let _t = task::spawn(async move {
                let mut token = ACCESS_TOKEN.lock().await;
                *token = Some(token_response.clone());
            });
            Ok(cloned)
        }else {
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
        let res: reqwest::Response = client.post(format!("{}/collection/oauth2/token/", self.url))
        .basic_auth(self.api_user.to_string(), Some(self.api_key.to_string()))
        .header("X-Target-Environment", self.environment.to_string())
        .header("Content-type", "application/x-www-form-urlencoded")
        .header("Ocp-Apim-Subscription-Key", &self.primary_key)
        .body(AccessTokenRequest{grant_type: "urn:openid:params:grant-type:ciba".to_string(), auth_req_id})
        .send().await?;

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
        let mut req = client.post(format!("{}/collection/v1_0/bc-authorize", self.url))
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
    use crate::{Party, PartyIdType, Money};
    use dotenv::dotenv;
    use std::env;


    #[tokio::test] 
    async fn test_create_and_cancel_invoice(){
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");
        let collection = Collection::new(mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key);
        
        let payer : Party = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "+242064818006".to_string(),
        };
        let payee : Party = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "+242074818007".to_string(),
        };
        let invoice = InvoiceRequest::new("100".to_string(), Currency::EUR.to_string(), "360".to_string(), payer, payee, "test invoice".to_string());
        let invoice_id = collection.create_invoice(invoice, None).await.expect("Error creating invoice");
        let res = collection.cancel_invoice(invoice_id.as_str(), None).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_request_payment(){
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");
        let collection = Collection::new(mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key);
        
        let payer : Party = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "+242064818006".to_string(),
        };
        let request = RequestToPay::new("100".to_string(), Currency::EUR, payer, "test_payer_message".to_string(), "test_payee_note".to_string());
        let res = collection.request_to_pay(request).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_request_payment_status(){
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");
        let collection = Collection::new(mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key);
        
        let payer : Party = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "+242064818006".to_string(),
        };
        let request = RequestToPay::new("100".to_string(), Currency::EUR, payer, "test_payer_message".to_string(), "test_payee_note".to_string());
        let res = collection.request_to_pay(request).await.expect("Error requesting payment");

        assert_ne!(res.as_str().len(), 0);

        let status = collection.request_to_pay_transaction_status(res.as_str()).await.expect("Error getting payment status");
        assert_eq!(status.status, "SUCCESSFUL");
    }

    #[tokio::test]
    async fn test_request_payment_with_delivery_notification(){
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");
        let collection = Collection::new(mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key);
        
        let payer : Party = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "+242064818006".to_string(),
        };
        let request = RequestToPay::new("100".to_string(), Currency::EUR, payer, "test_payer_message".to_string(), "test_payee_note".to_string());
        let res = collection.request_to_pay(request).await.expect("Error requesting payment");

        assert_ne!(res.0.len(), 0);

        let notifcation_result = collection.request_to_pay_delivery_notification(&res.0, DeliveryNotificationRequest{notification_message: "test_notification_message".to_string()}).await;
        assert!(notifcation_result.is_ok());

    }



    #[tokio::test]
    async fn test_bc_authorize(){
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");
        let collection = Collection::new(mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key);
        let bc_authorize_res = collection.bc_authorize("563607".to_string(), None).await;
        assert!(bc_authorize_res.is_ok());
        assert_ne!(bc_authorize_res.unwrap().auth_req_id.len(), 0);
    }

    #[tokio::test]
    async fn test_create_o_auth_2_token(){
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");
        let collection = Collection::new(mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key);
        let bc_authorize_res = collection.bc_authorize("563607".to_string(), None).await;
        assert!(bc_authorize_res.is_ok());
        let res = collection.create_o_auth_2_token(bc_authorize_res.unwrap().auth_req_id).await.expect("Error creating o auth 2 token");
        assert_ne!(res.access_token.len(), 0);
    }

    #[tokio::test]
    async fn test_get_user_info_with_consent(){
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");
        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");

        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");

        let collection = Collection::new(mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key);
        let bc_authorize_res = collection.bc_authorize("563607".to_string(), None).await;
        assert!(bc_authorize_res.is_ok());
        let res = collection.create_o_auth_2_token(bc_authorize_res.unwrap().auth_req_id).await.expect("Error creating o auth 2 token");
        assert_ne!(res.access_token.len(), 0);
        let res = collection.get_user_info_with_consent(res.access_token).await.expect("Error getting user info with consent");
        assert_ne!(res.family_name.len(), 0);
    }


    #[tokio::test]
    async fn test_get_account_balance(){
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");
        let collection = Collection::new(mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key);
        let res = collection.get_account_balance().await;
        if res.is_ok() {
            assert_ne!(res.unwrap().available_balance.len(), 0);
        }

    }


    // #[tokio::test]
    // async fn test_get_account_balance_in_specific_currency() {
    //     dotenv().ok();
    //     let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

    //     let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
    //     let secondary_key = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
    //     let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
    //     let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");
    //     let collection = Collection::new(mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key);
    //     let res = collection.get_account_balance_in_specific_currency(Currency::XAF.to_string()).await.expect("Error getting account balance");
    //     assert_ne!(res.available_balance.len(), 0);
    // }

    #[tokio::test]
    async fn test_get_basic_user_info(){
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");
        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key: String = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");

        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");

        let collection = Collection::new(mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key);
        let res = collection.get_basic_user_info("256774290781").await.expect("Error getting basic user info");
        assert_ne!(res.given_name.len(), 0);
    }



    #[tokio::test]
    async fn test_get_invoice_status(){
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");
        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");

        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");

        let collection = Collection::new(mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key);
        let payer : Party = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "+242064818006".to_string(),
        };
        let payee : Party = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "+242074818007".to_string(),
        };
        let invoice = InvoiceRequest::new("100".to_string(), Currency::EUR.to_string(), "360".to_string(), payer, payee, "test invoice".to_string());
        let invoice_id = collection.create_invoice(invoice, None).await.expect("Error creating invoice");

        let res = collection.get_invoice_status(invoice_id.0).await.expect("Error getting invoice status");
        assert_eq!(res.status, "SUCCESSFUL".to_string());
    }



    #[tokio::test]
    async fn test_pre_approval(){
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");
        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");

        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");

        let collection = Collection::new(mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key);

        let user : Party = Party {
            party_id_type:PartyIdType::MSISDN,
            party_id: "+242064818006".to_string(),
        };
        let preapproval = PreApprovalRequest { payer: user, payer_currency: Currency::EUR.to_string(), payer_message: "".to_string(), validity_time: 3600};
        let res = collection.pre_approval(preapproval).await;
        if res.is_ok() {
            assert!(true);
        }
    }

    #[tokio::test]
    async fn test_get_pre_approval_status(){
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");
        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");

        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");

        let collection = Collection::new(mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key);

        let user : Party = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "+242064818006".to_string(),
        };
        let preapproval = PreApprovalRequest { payer: user, payer_currency: Currency::EUR.to_string(), payer_message: "".to_string(), validity_time: 3600};
        let res = collection.pre_approval(preapproval).await;

        if res.is_ok() {
            let res = collection.get_pre_approval_status(res.unwrap()).await.expect("Error getting pre approval status");
            assert_ne!(res.status.len(), 0);
        }
    }

    #[tokio::test]
    async fn test_create_payment(){
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");
        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");
        let collection = Collection::new(mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key);

        let payment = CreatePaymentRequest::new(
            Money{
                amount: "100".to_string(),
                currency: Currency::EUR.to_string(),
            },
            "561551442".to_string(),
            "WaterProvider".to_string(), 
            "203".to_string(), 
            "Monthly Payments".to_string(), 
            "788".to_string(), 
            "Thank You ".to_string(), 
            "Thank You".to_string(), 
            2, 
            true
        );
        let res = collection.create_payments(payment, None).await.expect("Error creating payment");
        assert_ne!(res.as_str().len(), 0);
    }

    #[tokio::test]
    async fn test_payment_status(){
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");

        
        let collection = Collection::new(mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key);
        let payment = CreatePaymentRequest::new(
            Money{
                amount: "100".to_string(),
                currency: Currency::EUR.to_string(),
            },
            "561551442".to_string(),
            "WaterProvider".to_string(), 
            "203".to_string(), 
            "Monthly Payments".to_string(), 
            "788".to_string(), 
            "Thank You ".to_string(), 
            "Thank You".to_string(), 
            2, 
            true
        );
        let payment_id = collection.create_payments(payment, None).await.expect("Error creating payment");
        let res = collection.get_payment_status(payment_id.as_string()).await.expect("Error getting payment status");
        assert_eq!(res.status, "SUCCESSFUL");
    }

    #[tokio::test]
    async fn test_request_to_withdraw_v1(){
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");
        
        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");

        
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");

        
        let collection = Collection::new(mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key);
        let payer : Party = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "467331234534".to_string(),
        };
        let request = RequestToPay::new("100.0".to_string(), Currency::EUR, payer, "test_payer_message".to_string(), "test_payee_note".to_string());
        let res = collection.request_to_withdraw_v1(request, None).await.expect("Error requesting to withdraw");
        assert_ne!(res.as_str().len(), 0);
    }

    #[tokio::test]
    async fn test_request_to_withdraw_v2(){
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");
        
        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");

        
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");

        
        let collection = Collection::new(mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key);
        let payer : Party = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "+242064818006".to_string(),
        };
        let request = RequestToPay::new("100".to_string(), Currency::EUR, payer, "test_payer_message".to_string(), "test_payee_note".to_string());
        let res = collection.request_to_withdraw_v2(request, None).await.expect("Error requesting to withdraw");
        assert_ne!(res.as_str().len(), 0);
    }

    #[tokio::test]
    async fn test_request_withdraw_status(){
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");
        
        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");

        
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");

        
        let collection = Collection::new(mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key);
        let payer : Party = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "+242064818006".to_string(),
        };
        let request = RequestToPay::new("100".to_string(), Currency::EUR, payer, "test_payer_message".to_string(), "test_payee_note".to_string());
        let withdraw_id = collection.request_to_withdraw_v2(request, None).await.expect("Error requesting to withdraw");
        let res = collection.request_to_withdraw_transaction_status(&withdraw_id.0).await.expect("Error getting request to withdraw status");
        assert_eq!(res.status, "SUCCESSFUL");
    }

    #[tokio::test] 
    async fn test_validate_account_holder_status(){
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");
        
        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");

        
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");

        
        let collection = Collection::new(mtn_url, Environment::Sandbox, api_user, api_key, primary_key, secondary_key);
        let res = collection.validate_account_holder_status("256774290781", "MSISDN").await;
        assert!(res.is_ok());
    }
}
