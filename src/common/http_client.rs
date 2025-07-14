use std::collections::HashMap;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::{Environment, errors::error::MtnMomoError};
use super::token_manager::{TokenManager, ProductType};
use crate::products::auth::Authorization;

pub struct MomoHttpClient {
    client: Client,
    base_url: String,
    product_type: ProductType,
    environment: Environment,
    api_user: String,
    api_key: String,
    primary_key: String,
}

impl MomoHttpClient {
    pub fn new(
        base_url: String,
        product_type: ProductType,
        environment: Environment,
        api_user: String,
        api_key: String,
        primary_key: String,
    ) -> Self {
        Self {
            client: Client::new(),
            base_url,
            product_type,
            environment,
            api_user,
            api_key,
            primary_key,
        }
    }

    pub async fn post<T, R>(&self, endpoint: &str, body: &T, callback_url: Option<String>) -> Result<R, Box<dyn std::error::Error>>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        let token = self.get_or_create_token().await?;
        let product_path = match self.product_type {
            ProductType::Collection => "collection/v1_0",
            ProductType::Disbursement => "disbursement/v1_0",
            ProductType::Remittance => "remittance/v1_0",
        };
        let url = format!("{}/{}/{}", self.base_url, product_path, endpoint);
        
        let mut headers = self.build_common_headers(&token.access_token);
        
        if let Some(callback) = callback_url {
            headers.insert("X-Callback-Url".to_string(), callback);
        }

        let response = self.client
            .post(&url)
            .headers(self.headers_to_reqwest(&headers)?)
            .body(serde_json::to_string(body)?)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            let text = response.text().await?;
            let result: R = serde_json::from_str(&text)?;
            Ok(result)
        } else {
            let error_text = response.text().await?;
            Err(Box::new(MtnMomoError::HttpError(format!(
                "HTTP {}: {}",
                status,
                error_text
            ))))
        }
    }

    pub async fn get<R>(&self, endpoint: &str) -> Result<R, Box<dyn std::error::Error>>
    where
        R: for<'de> Deserialize<'de>,
    {
        let token = self.get_or_create_token().await?;
        let product_path = match self.product_type {
            ProductType::Collection => "collection/v1_0",
            ProductType::Disbursement => "disbursement/v1_0",
            ProductType::Remittance => "remittance/v1_0",
        };
        let url = format!("{}/{}/{}", self.base_url, product_path, endpoint);
        
        let headers = self.build_common_headers(&token.access_token);

        let response = self.client
            .get(&url)
            .headers(self.headers_to_reqwest(&headers)?)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            let text = response.text().await?;
            let result: R = serde_json::from_str(&text)?;
            Ok(result)
        } else {
            let error_text = response.text().await?;
            Err(Box::new(MtnMomoError::HttpError(format!(
                "HTTP {}: {}",
                status,
                error_text
            ))))
        }
    }

    pub async fn get_or_create_token(&self) -> Result<crate::TokenResponse, Box<dyn std::error::Error>> {
        if let Some(token) = TokenManager::get_valid_token(self.product_type).await {
            return Ok(token);
        }

        let product_name = match self.product_type {
            ProductType::Collection => "collection",
            ProductType::Disbursement => "disbursement", 
            ProductType::Remittance => "remittance",
        };

        let url = format!("{}/{}", self.base_url, product_name);
        let auth = Authorization {};
        let token = auth.create_access_token(
            url,
            self.api_user.clone(),
            self.api_key.clone(),
            self.primary_key.clone(),
        ).await?;

        TokenManager::store_token(self.product_type, token.clone()).await;
        Ok(token)
    }

    fn build_common_headers(&self, access_token: &str) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), format!("Bearer {}", access_token));
        headers.insert("X-Target-Environment".to_string(), self.environment.to_string());
        headers.insert("Ocp-Apim-Subscription-Key".to_string(), self.primary_key.clone());
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers
    }

    fn headers_to_reqwest(&self, headers: &HashMap<String, String>) -> Result<reqwest::header::HeaderMap, Box<dyn std::error::Error>> {
        let mut header_map = reqwest::header::HeaderMap::new();
        
        for (key, value) in headers {
            let header_name = reqwest::header::HeaderName::from_bytes(key.as_bytes())?;
            let header_value = reqwest::header::HeaderValue::from_str(value)?;
            header_map.insert(header_name, header_value);
        }
        
        Ok(header_map)
    }
}