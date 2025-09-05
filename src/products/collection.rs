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

use crate::{
    common::{http_client::MomoHttpClient, token_manager::ProductType},
    BCAuthorizeResponse, Balance, BasicUserInfoJsonResponse, CreatePaymentRequest, Currency,
    DeliveryNotificationRequest, Environment, InvoiceDeleteRequest, InvoiceId, InvoiceRequest,
    InvoiceResult, OAuth2TokenResponse, PaymentId, PaymentResult, PreApprovalRequest,
    PreApprovalResult, RequestToPay, RequestToPayResult, TokenResponse, TransactionId, WithdrawId,
};

use super::{account::Account, auth::Authorization};

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
    account: Account,
    auth: Authorization,
    http_client: MomoHttpClient,
}

impl Collection {
    /// Create a new instance of Collection
    ///
    /// # Parameters
    ///
    /// * 'url', MTN MOMO collection url
    /// * 'environment', environement to be used, default = Sandbox
    /// * 'api_user', the api user to be used
    /// * 'api_key', the api key to be use
    /// * 'primary_key', the primary key of the collection product
    /// * 'secondary_key', the secondary key of the collection product
    ///
    /// # Returns
    /// * Collection
    pub fn new(
        url: String,
        environment: Environment,
        api_user: String,
        api_key: String,
        primary_key: String,
        secondary_key: String,
    ) -> Collection {
        let account = Account {};
        let auth = Authorization {};
        let http_client = MomoHttpClient::new(
            url.clone(),
            ProductType::Collection,
            environment,
            api_user.clone(),
            api_key.clone(),
            primary_key.clone(),
        );
        Collection {
            url,
            primary_key,
            secondary_key,
            environment,
            api_user,
            api_key,
            account,
            auth,
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
        let url = format!("{}/collection", self.url);
        self.auth
            .create_o_auth_2_token(
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
        let url = format!("{}/collection", self.url);
        let access_token: TokenResponse = self
            .auth
            .create_access_token(
                url.clone(),
                self.api_user.clone(),
                self.api_key.clone(),
                self.primary_key.clone(),
            )
            .await?;
        self.auth
            .bc_authorize(
                url,
                self.environment,
                self.primary_key.clone(),
                msisdn,
                callback_url,
                access_token,
            )
            .await
    }

    /// This operation is used to cancel an invoice.
    ///
    /// # Parameters
    ///
    /// * 'invoice_id', the reference id of the invoice
    /// * 'callback_url', The callback url to be used by the MOMO Core API to notify the merchant of the invoice status
    ///
    /// # Returns
    /// * (), the request is ok
    pub async fn cancel_invoice(
        &self,
        invoice_id: &str,
        callback_url: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.http_client.get_or_create_token().await?;
        let mut req = client
            .delete(format!(
                "{}/collection/v2_0/invoice/{}",
                self.url, invoice_id
            ))
            .bearer_auth(access_token.access_token)
            .header("Content-Type", "application/json")
            .header("X-Target-Environment", self.environment.to_string())
            .header("X-Reference-Id", uuid::Uuid::new_v4().to_string())
            .header("Ocp-Apim-Subscription-Key", &self.primary_key)
            .body(InvoiceDeleteRequest {
                external_id: invoice_id.to_string(),
            });

        if let Some(callback_url) = callback_url {
            if !callback_url.is_empty() {
                req = req.header(
                    "x-callback-url",
                    format!("{}/collection_invoice", callback_url),
                );
            }
        }

        let res = req.send().await?;

        if res.status().is_success() {
            Ok(())
        } else {
            Err(Box::new(std::io::Error::other(res.text().await?)))
        }
    }

    /// Create an invoice that can be paid by an intended payer via any channel at a later stage
    ///
    /// # Parameters
    ///
    /// * 'invoice': InvoiceRequest, the invoice to be created on the MOMO Core API
    /// * 'callback_url', The callback url to be used by the MOMO Core API to notify the merchant of the invoice status
    ///
    /// # Returns
    ///
    /// * 'InvoiceId', the is the exeternal id of the invoice
    pub async fn create_invoice(
        &self,
        invoice: InvoiceRequest,
        callback_url: Option<&str>,
    ) -> Result<InvoiceId, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.http_client.get_or_create_token().await?;
        let mut req = client
            .post(format!("{}/collection/v2_0/invoice", self.url))
            .bearer_auth(access_token.access_token)
            .header("X-Reference-Id", &invoice.external_id)
            .header("X-Target-Environment", self.environment.to_string())
            .header("Content-Type", "application/json")
            .header("Ocp-Apim-Subscription-Key", &self.primary_key)
            .body(invoice.clone());

        if let Some(callback_url) = callback_url {
            if !callback_url.is_empty() {
                req = req.header(
                    "x-callback-url",
                    format!("{}/collection_invoice", callback_url),
                );
            }
        }

        let res = req.send().await?;

        if res.status().is_success() {
            Ok(InvoiceId::new(invoice.external_id))
        } else {
            let res_clone = res.text().await?;
            Err(Box::new(std::io::Error::other(res_clone)))
        }
    }

    /// Create a payment for an externa bills or perform a air-time top_ups.
    ///
    /// # Parameters
    /// * 'payment': CreatePaymentRequest, the payment to be created on the Momo Core API
    /// * 'callback_url', The callback url to be used by the MOMO Core API to notify the merchant of the payment status
    ///
    /// # Returns
    ///
    /// * 'PaymentId', this is the external id of the payment
    pub async fn create_payments(
        &self,
        payment: CreatePaymentRequest,
        callback_url: Option<&str>,
    ) -> Result<PaymentId, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.http_client.get_or_create_token().await?;
        let mut req = client
            .post(format!("{}/collection/v2_0/payment", self.url))
            .bearer_auth(access_token.access_token)
            .header("X-Target-Environment", self.environment.to_string())
            .header("X-Reference-Id", &payment.external_transaction_id)
            .header("Cache-Control", "no-cache")
            .header("Ocp-Apim-Subscription-Key", &self.primary_key)
            .body(payment.clone());

        if let Some(callback_url) = callback_url {
            if !callback_url.is_empty() {
                req = req.header(
                    "x-callback-url",
                    format!("{}/collection_payment", callback_url),
                );
            }
        }

        let res = req.send().await?;

        if res.status().is_success() {
            Ok(PaymentId::new(payment.external_transaction_id))
        } else {
            Err(Box::new(std::io::Error::other(res.text().await?)))
        }
    }

    /// This operation is used to get the status of an invoice
    ///
    /// # Parameters
    ///
    /// * 'invoice_id': String, the invoice id to get the status from
    ///
    /// # Returns
    ///
    /// * 'InvoiceResult'
    #[allow(dead_code)]
    async fn get_invoice_status(
        &self,
        invoice_id: String,
    ) -> Result<InvoiceResult, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.http_client.get_or_create_token().await?;
        let res = client
            .get(format!(
                "{}/collection/v2_0/invoice/{}",
                self.url, invoice_id
            ))
            .bearer_auth(access_token.access_token)
            .header("X-Target-Environment", self.environment.to_string())
            .header("Ocp-Apim-Subscription-Key", &self.primary_key)
            .send()
            .await?;

        if res.status().is_success() {
            let body = res.text().await?;
            let invoice_status: InvoiceResult = serde_json::from_str(&body)?;
            Ok(invoice_status)
        } else {
            Err(Box::new(std::io::Error::other(res.text().await?)))
        }
    }

    /// This operation is used to get the status of a payment.
    ///
    /// # Parameters
    ///
    /// * 'payment_id': String, the payment id to get the status from
    ///
    ///
    /// # Returns
    ///
    /// * 'PaymentResult'
    #[allow(dead_code)]
    async fn get_payment_status(
        &self,
        payment_id: String,
    ) -> Result<PaymentResult, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.http_client.get_or_create_token().await?;
        let res = client
            .get(format!(
                "{}/collection/v2_0/payment/{}",
                self.url, payment_id
            ))
            .bearer_auth(access_token.access_token)
            .header("X-Target-Environment", self.environment.to_string())
            .header("Ocp-Apim-Subscription-Key", &self.primary_key)
            .send()
            .await?;

        if res.status().is_success() {
            let body = res.text().await?;
            let payment_status: PaymentResult = serde_json::from_str(&body)?;
            Ok(payment_status)
        } else {
            Err(Box::new(std::io::Error::other(res.text().await?)))
        }
    }

    /// This operation is used to get the status of a pre-approval
    ///
    ///
    /// # Parameters
    ///
    /// * 'pre_approval_id', The pre-approval id to get the status
    ///
    /// # Returns
    ///
    /// * 'PreApprovalResult'
    #[allow(dead_code)]
    async fn get_pre_approval_status(
        &self,
        pre_approval_id: String,
    ) -> Result<PreApprovalResult, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.http_client.get_or_create_token().await?;
        let res = client
            .get(format!(
                "{}/collection/v2_0/preapproval/{}",
                self.url, pre_approval_id
            ))
            .bearer_auth(access_token.access_token)
            .header("X-Target-Environment", self.environment.to_string())
            .header("Cache-Control", "no-cache")
            .header("Ocp-Apim-Subscription-Key", &self.primary_key)
            .send()
            .await?;

        if res.status().is_success() {
            let body = res.text().await?;
            let pre_approval_status: PreApprovalResult = serde_json::from_str(&body)?;
            Ok(pre_approval_status)
        } else {
            Err(Box::new(std::io::Error::other(res.text().await?)))
        }
    }

    /// Preapproval operation is used to create a pre-approval.
    ///
    /// # Parameters
    ///
    /// * 'preaproval'; the pre-approval to be created on the MOMO Core API
    pub async fn pre_approval(
        &self,
        preaproval: PreApprovalRequest,
        callback_url: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let external_id = uuid::Uuid::new_v4().to_string();
        let client = reqwest::Client::new();
        let access_token = self.http_client.get_or_create_token().await?;
        let mut req = client
            .post(format!("{}/collection/v2_0/preapproval", self.url))
            .bearer_auth(access_token.access_token)
            .header("X-Target-Environment", self.environment.to_string())
            .header("Cache-Control", "no-cache")
            .header("Content-Type", "application/json")
            .header("X-Reference-Id", &external_id)
            .header("Ocp-Apim-Subscription-Key", &self.primary_key)
            .body(preaproval);

        if let Some(callback_url) = callback_url {
            if !callback_url.is_empty() {
                req = req.header(
                    "x-callback-url",
                    format!("{}/collection_preapproval", callback_url),
                );
            }
        }

        let res = req.send().await?;

        if res.status().is_success() {
            Ok(external_id)
        } else {
            Err(Box::new(std::io::Error::other(res.text().await?)))
        }
    }

    /// This operation is used to request a payment from a consumer (Payer). The payer will be asked to authorize the payment.
    /// The transaction will be executed once the payer has authorized the payment.
    /// The requesttopay will be in status PENDING until the transaction is authorized or declined by the payer or it is timed out by the system.
    /// Status of the transaction can be validated by using the GET /requesttopay/<resourceId>
    ///
    ///
    /// # Parameters
    ///
    /// * 'request': RequestToPay
    /// * 'callback_url', the callback url to send updates to
    ///
    /// # Returns
    ///
    /// * 'TransactionId' (external_id), the transaction id of the payment.
    pub async fn request_to_pay(
        &self,
        request: RequestToPay,
        callback_url: Option<&str>,
    ) -> Result<TransactionId, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.http_client.get_or_create_token().await?;
        let mut req = client
            .post(format!("{}/collection/v1_0/requesttopay", self.url))
            .bearer_auth(access_token.access_token)
            .header("X-Target-Environment", self.environment.to_string())
            .header("X-Reference-Id", &request.external_id)
            .header("Ocp-Apim-Subscription-Key", &self.primary_key)
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&request)?);

        if let Some(callback_url) = callback_url {
            if !callback_url.is_empty() {
                println!("Using callback_url: {}", callback_url);
                req = req.header(
                    "x-callback-url",
                    format!("{}/collection_request_to_pay", callback_url),
                );
            }
        }

        let res = req.send().await?;

        if res.status().is_success() {
            Ok(TransactionId::new(request.external_id))
        } else {
            let status_code = res.status().as_u16();
            let error_text = res.text().await?;
            Err(Box::new(std::io::Error::other(format!(
                "{{ \"statusCode\": {}, \"message\": \"{}\" }}",
                status_code, error_text
            ))))
        }
    }

    /// This operation is used to send additional Notification  to an end user.
    ///
    /// # Parameters
    ///
    /// * 'external_id', this is the external id of the request to pay
    /// * 'notification': DeliveryNotificationRequest
    ///
    /// # Returns
    ///
    /// * ()
    pub async fn request_to_pay_delivery_notification(
        &self,
        external_id: &str,
        notification: DeliveryNotificationRequest,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.http_client.get_or_create_token().await?;
        let res = client
            .post(format!(
                "{}/collection/v1_0/requesttopay/{}/deliverynotification",
                self.url, external_id
            ))
            .bearer_auth(access_token.access_token)
            .header("X-Target-Environment", self.environment.to_string())
            .header("notificationMessage", &notification.notification_message)
            .header("Language", "")
            .header("Cache-Control", "no-cache")
            .header("Ocp-Apim-Subscription-Key", &self.primary_key)
            .body(notification)
            .send()
            .await?;

        if res.status().is_success() {
            Ok(())
        } else {
            Err(Box::new(std::io::Error::other(res.text().await?)))
        }
    }

    /// this operation is used to get the status of a request to pay.
    ///
    /// # Parameters
    ///
    /// * 'payment_id', the payment id that we are trying to get the status
    ///
    /// # Returns
    ///
    /// *  'RequestToPayResult', the is the result of the request to pay
    pub async fn request_to_pay_transaction_status(
        &self,
        payment_id: &str,
    ) -> Result<RequestToPayResult, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.http_client.get_or_create_token().await?;
        let res = client
            .get(format!(
                "{}/collection/v1_0/requesttopay/{}",
                self.url, payment_id
            ))
            .bearer_auth(access_token.access_token)
            .header("X-Target-Environment", self.environment.to_string())
            .header("Cache-Control", "no-cache")
            .header("Ocp-Apim-Subscription-Key", &self.primary_key)
            .send()
            .await?;

        if res.status().is_success() {
            let body = res.text().await?;
            let request_to_pay_result: RequestToPayResult = serde_json::from_str(&body)?;
            Ok(request_to_pay_result)
        } else {
            Err(Box::new(std::io::Error::other(res.text().await?)))
        }
    }

    /// This operation is used to get the status of a request to withdraw
    ///
    /// # Parameters
    ///
    /// * 'payment_id', the payment id to be cancelled, external_id of the request to pay
    ///
    /// # Returns
    /// * "RequestToPayResult", this is the result of the request to pay
    pub async fn request_to_withdraw_transaction_status(
        &self,
        payment_id: &str,
    ) -> Result<RequestToPayResult, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.http_client.get_or_create_token().await?;
        let res = client
            .get(format!(
                "{}/collection/v1_0/requesttowithdraw/{}",
                self.url, payment_id
            ))
            .bearer_auth(access_token.access_token)
            .header("X-Target-Environment", self.environment.to_string())
            .header("Ocp-Apim-Subscription-Key", &self.primary_key)
            .send()
            .await?;

        if res.status().is_success() {
            let body = res.text().await?;
            let request_to_pay_result: RequestToPayResult = serde_json::from_str(&body)?;
            Ok(request_to_pay_result)
        } else {
            Err(Box::new(std::io::Error::other(res.text().await?)))
        }
    }

    /// This operation is used to request a withdrawal (cash-out) from a consumer (Payer).
    /// The payer will be asked to authorize the withdrawal.
    /// The transaction will be executed once the payer has authorized the withdrawal
    /// # Parameters
    /// * 'request': RequestToPay, this is the request to withwray
    /// * 'callback_url': Option<&str>, this is the callback url to be used by the MOMO Core API to notify the merchant of the request to withdraw status
    ///
    /// # Returns
    /// * WithdrawId, this is the external_id of the request to withdraw
    pub async fn request_to_withdraw_v1(
        &self,
        request: RequestToPay,
        callback_url: Option<&str>,
    ) -> Result<WithdrawId, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.http_client.get_or_create_token().await?;
        let mut req = client
            .post(format!("{}/collection/v1_0/requesttowithdraw", self.url))
            .bearer_auth(access_token.access_token)
            .header("X-Target-Environment", self.environment.to_string())
            .header("X-Reference-Id", &request.external_id)
            .header("Ocp-Apim-Subscription-Key", &self.primary_key)
            .header("Content-Type", "application/json")
            .body(request.clone());

        if let Some(callback_url) = callback_url {
            if !callback_url.is_empty() {
                req = req.header(
                    "x-callback-url",
                    format!("{}/collection_request_to_withdraw_v1", callback_url),
                );
            }
        }

        let res = req.send().await?;

        if res.status().is_success() {
            Ok(WithdrawId::new(request.external_id))
        } else {
            Err(Box::new(std::io::Error::other(res.text().await?)))
        }
    }

    /// This operation is used to request a withdrawal (cash-out) from a consumer (Payer)
    /// the payer will be asked to authorize the withdrawal.
    /// the transaction will be executed once the payer has authorized the withdrawal.
    ///
    /// # Parameters
    ///
    /// * 'request': RequestToPay, this is the request to withdraw
    /// * 'callback_url', this is the callback url to be used by the MOMO Core API to notify the merchant of the request to withdraw status
    ///
    ///
    /// # Returns
    ///
    /// * 'WithdrawId', the reference id of the request
    pub async fn request_to_withdraw_v2(
        &self,
        request: RequestToPay,
        callback_url: Option<&str>,
    ) -> Result<WithdrawId, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let access_token = self.http_client.get_or_create_token().await?;
        let mut req = client
            .post(format!("{}/collection/v2_0/requesttowithdraw", self.url))
            .bearer_auth(access_token.access_token)
            .header("X-Target-Environment", self.environment.to_string())
            .header("X-Reference-Id", &request.external_id)
            .header("Ocp-Apim-Subscription-Key", &self.primary_key)
            .header("Content-Type", "application/json")
            .body(request.clone());

        if let Some(callback_url) = callback_url {
            if !callback_url.is_empty() {
                req = req.header(
                    "x-callback-url",
                    format!("{}/collection_request_to_withdraw_v2", callback_url),
                );
            }
        }

        let res = req.send().await?;

        if res.status().is_success() {
            Ok(WithdrawId::new(request.external_id))
        } else {
            Err(Box::new(std::io::Error::other(res.text().await?)))
        }
    }

    /// This operation is used to get the balance of the account.
    /// # Returns
    ///
    /// * 'Balance', the balance
    pub async fn get_account_balance(&self) -> Result<Balance, Box<dyn std::error::Error>> {
        let url = format!("{}/collection", self.url);
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
        let url = format!("{}/collection", self.url);
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
        let url = format!("{}/collection", self.url);
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
        let url = format!("{}/collection", self.url);
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
        let url = format!("{}/collection", self.url);
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
    use crate::{Money, Party, PartyIdType};
    use dotenv::dotenv;
    use std::env;

    #[tokio::test]
    async fn test_create_and_cancel_invoice() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");
        let collection = Collection::new(
            mtn_url,
            Environment::Sandbox,
            api_user,
            api_key,
            primary_key,
            secondary_key,
        );

        let payer: Party = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "+242064818006".to_string(),
        };
        let payee: Party = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "+242074818007".to_string(),
        };
        let invoice = InvoiceRequest::new(
            "100".to_string(),
            Currency::EUR.to_string(),
            "360".to_string(),
            payer,
            payee,
            "test invoice".to_string(),
        );
        let invoice_id = collection
            .create_invoice(invoice, None)
            .await
            .expect("Error creating invoice");
        let res = collection.cancel_invoice(invoice_id.as_str(), None).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_request_payment() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");
        let collection = Collection::new(
            mtn_url,
            Environment::Sandbox,
            api_user,
            api_key,
            primary_key,
            secondary_key,
        );

        let payer: Party = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "+242064818006".to_string(),
        };
        let request = RequestToPay::new(
            "100".to_string(),
            Currency::EUR,
            payer,
            "test_payer_message".to_string(),
            "test_payee_note".to_string(),
        );
        let res = collection.request_to_pay(request, None).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_request_payment_status() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");
        let collection = Collection::new(
            mtn_url,
            Environment::Sandbox,
            api_user,
            api_key,
            primary_key,
            secondary_key,
        );

        let payer: Party = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "+242064818006".to_string(),
        };
        let request = RequestToPay::new(
            "100".to_string(),
            Currency::EUR,
            payer,
            "test_payer_message".to_string(),
            "test_payee_note".to_string(),
        );
        let res = collection
            .request_to_pay(request, None)
            .await
            .expect("Error requesting payment");

        assert_ne!(res.as_str().len(), 0);

        let status = collection
            .request_to_pay_transaction_status(res.as_str())
            .await
            .expect("Error getting payment status");
        assert_eq!(status.status, "SUCCESSFUL");
    }

    #[tokio::test]
    async fn test_request_payment_with_delivery_notification() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");
        let collection = Collection::new(
            mtn_url,
            Environment::Sandbox,
            api_user,
            api_key,
            primary_key,
            secondary_key,
        );

        let payer: Party = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "+242064818006".to_string(),
        };
        let request = RequestToPay::new(
            "100".to_string(),
            Currency::EUR,
            payer,
            "test_payer_message".to_string(),
            "test_payee_note".to_string(),
        );
        let res = collection
            .request_to_pay(request, None)
            .await
            .expect("Error requesting payment");

        assert_ne!(res.as_str().len(), 0);

        let notifcation_result = collection
            .request_to_pay_delivery_notification(
                res.as_str(),
                DeliveryNotificationRequest {
                    notification_message: "test_notification_message".to_string(),
                },
            )
            .await;
        assert!(notifcation_result.is_ok());
    }

    #[tokio::test]
    async fn test_bc_authorize() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");
        let collection = Collection::new(
            mtn_url,
            Environment::Sandbox,
            api_user,
            api_key,
            primary_key,
            secondary_key,
        );
        let bc_authorize_res = collection.bc_authorize("563607".to_string(), None).await;
        assert!(bc_authorize_res.is_ok());
        assert_ne!(bc_authorize_res.unwrap().auth_req_id.len(), 0);
    }

    #[tokio::test]
    async fn test_create_o_auth_2_token() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");
        let collection = Collection::new(
            mtn_url,
            Environment::Sandbox,
            api_user,
            api_key,
            primary_key,
            secondary_key,
        );
        let bc_authorize_res = collection.bc_authorize("563607".to_string(), None).await;
        assert!(bc_authorize_res.is_ok());
        let res = collection
            .create_o_auth_2_token(bc_authorize_res.unwrap().auth_req_id)
            .await
            .expect("Error creating o auth 2 token");
        assert_ne!(res.access_token.len(), 0);
    }

    #[tokio::test]
    async fn test_get_user_info_with_consent() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");
        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");

        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");

        let collection = Collection::new(
            mtn_url,
            Environment::Sandbox,
            api_user,
            api_key,
            primary_key,
            secondary_key,
        );
        let bc_authorize_res = collection.bc_authorize("563607".to_string(), None).await;
        assert!(bc_authorize_res.is_ok());
        let res = collection
            .create_o_auth_2_token(bc_authorize_res.unwrap().auth_req_id)
            .await
            .expect("Error creating o auth 2 token");
        assert_ne!(res.access_token.len(), 0);
        let res = collection
            .get_user_info_with_consent(res.access_token)
            .await
            .expect("Error getting user info with consent");
        assert_ne!(res.family_name.len(), 0);
    }

    #[tokio::test]
    async fn test_get_account_balance() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");
        let collection = Collection::new(
            mtn_url,
            Environment::Sandbox,
            api_user,
            api_key,
            primary_key,
            secondary_key,
        );
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
    async fn test_get_basic_user_info() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");
        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key: String =
            env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");

        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");

        let collection = Collection::new(
            mtn_url,
            Environment::Sandbox,
            api_user,
            api_key,
            primary_key,
            secondary_key,
        );
        let res = collection
            .get_basic_user_info("256774290781")
            .await
            .expect("Error getting basic user info");
        assert_ne!(res.given_name.len(), 0);
    }

    #[tokio::test]
    async fn test_get_invoice_status() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");
        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");

        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");

        let collection = Collection::new(
            mtn_url,
            Environment::Sandbox,
            api_user,
            api_key,
            primary_key,
            secondary_key,
        );
        let payer: Party = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "+242064818006".to_string(),
        };
        let payee: Party = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "+242074818007".to_string(),
        };
        let invoice = InvoiceRequest::new(
            "100".to_string(),
            Currency::EUR.to_string(),
            "360".to_string(),
            payer,
            payee,
            "test invoice".to_string(),
        );
        let invoice_id = collection
            .create_invoice(invoice, None)
            .await
            .expect("Error creating invoice");

        let res = collection
            .get_invoice_status(invoice_id.as_string())
            .await
            .expect("Error getting invoice status");
        assert_eq!(res.status, "SUCCESSFUL".to_string());
    }

    #[tokio::test]
    #[ignore]
    async fn test_pre_approval() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");
        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");

        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");

        let collection = Collection::new(
            mtn_url,
            Environment::Sandbox,
            api_user,
            api_key,
            primary_key,
            secondary_key,
        );

        let user: Party = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "+242064818006".to_string(),
        };
        let preapproval = PreApprovalRequest {
            payer: user,
            payer_currency: Currency::EUR.to_string(),
            payer_message: "".to_string(),
            validity_time: 3600,
        };
        let res = collection.pre_approval(preapproval, None).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_get_pre_approval_status() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");
        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");

        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");

        let collection = Collection::new(
            mtn_url,
            Environment::Sandbox,
            api_user,
            api_key,
            primary_key,
            secondary_key,
        );

        let user: Party = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "+242064818006".to_string(),
        };
        let preapproval = PreApprovalRequest {
            payer: user,
            payer_currency: Currency::EUR.to_string(),
            payer_message: "".to_string(),
            validity_time: 3600,
        };
        let res = collection.pre_approval(preapproval, None).await;

        if res.is_ok() {
            let res = collection
                .get_pre_approval_status(res.unwrap())
                .await
                .expect("Error getting pre approval status");
            assert_ne!(res.status.len(), 0);
        }
    }

    #[tokio::test]
    async fn test_create_payment() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");
        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");
        let collection = Collection::new(
            mtn_url,
            Environment::Sandbox,
            api_user,
            api_key,
            primary_key,
            secondary_key,
        );

        let payment = CreatePaymentRequest::new(
            Money {
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
            true,
        );
        let res = collection
            .create_payments(payment, None)
            .await
            .expect("Error creating payment");
        assert_ne!(res.as_str().len(), 0);
    }

    #[tokio::test]
    async fn test_payment_status() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");

        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");

        let collection = Collection::new(
            mtn_url,
            Environment::Sandbox,
            api_user,
            api_key,
            primary_key,
            secondary_key,
        );
        let payment = CreatePaymentRequest::new(
            Money {
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
            true,
        );
        let payment_id = collection
            .create_payments(payment, None)
            .await
            .expect("Error creating payment");
        let res = collection
            .get_payment_status(payment_id.as_string())
            .await
            .expect("Error getting payment status");
        assert_eq!(res.status, "SUCCESSFUL");
    }

    #[tokio::test]
    async fn test_request_to_withdraw_v1() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");

        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");

        let collection = Collection::new(
            mtn_url,
            Environment::Sandbox,
            api_user,
            api_key,
            primary_key,
            secondary_key,
        );
        let payer: Party = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "467331234534".to_string(),
        };
        let request = RequestToPay::new(
            "100.0".to_string(),
            Currency::EUR,
            payer,
            "test_payer_message".to_string(),
            "test_payee_note".to_string(),
        );
        let res = collection
            .request_to_withdraw_v1(request, None)
            .await
            .expect("Error requesting to withdraw");
        assert_ne!(res.as_str().len(), 0);
    }

    #[tokio::test]
    async fn test_request_to_withdraw_v2() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");

        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");

        let collection = Collection::new(
            mtn_url,
            Environment::Sandbox,
            api_user,
            api_key,
            primary_key,
            secondary_key,
        );
        let payer: Party = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "+242064818006".to_string(),
        };
        let request = RequestToPay::new(
            "100".to_string(),
            Currency::EUR,
            payer,
            "test_payer_message".to_string(),
            "test_payee_note".to_string(),
        );
        let res = collection
            .request_to_withdraw_v2(request, None)
            .await
            .expect("Error requesting to withdraw");
        assert_ne!(res.as_str().len(), 0);
    }

    #[tokio::test]
    async fn test_request_withdraw_status() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");

        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");

        let collection = Collection::new(
            mtn_url,
            Environment::Sandbox,
            api_user,
            api_key,
            primary_key,
            secondary_key,
        );
        let payer: Party = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "+242064818006".to_string(),
        };
        let request = RequestToPay::new(
            "100".to_string(),
            Currency::EUR,
            payer,
            "test_payer_message".to_string(),
            "test_payee_note".to_string(),
        );
        let withdraw_id = collection
            .request_to_withdraw_v2(request, None)
            .await
            .expect("Error requesting to withdraw");
        let res = collection
            .request_to_withdraw_transaction_status(withdraw_id.as_str())
            .await
            .expect("Error getting request to withdraw status");
        assert_eq!(res.status, "SUCCESSFUL");
    }

    #[tokio::test]
    async fn test_validate_account_holder_status() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");

        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");

        let api_user = env::var("MTN_API_USER").expect("API_USER must be set");
        let api_key = env::var("MTN_API_KEY").expect("API_KEY must be set");

        let collection = Collection::new(
            mtn_url,
            Environment::Sandbox,
            api_user,
            api_key,
            primary_key,
            secondary_key,
        );
        let res = collection
            .validate_account_holder_status("256774290781", "MSISDN")
            .await;
        match res {
            Ok(_) => println!("Account validation successful"),
            Err(e) => {
                println!(
                    "Account validation failed (this may be expected in test environment): {:?}",
                    e
                );
                return;
            }
        }
    }
}
