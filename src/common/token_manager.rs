use std::sync::Arc;
use chrono::Utc;
use once_cell::sync::Lazy;
use tokio::sync::RwLock;
use crate::responses::token_response::TokenResponse;

static TOKEN_STORAGE: Lazy<Arc<RwLock<TokenStorage>>> = 
    Lazy::new(|| Arc::new(RwLock::new(TokenStorage::new())));

#[derive(Debug, Clone)]
struct TokenStorage {
    collection_token: Option<TokenResponse>,
    disbursement_token: Option<TokenResponse>,
    remittance_token: Option<TokenResponse>,
}

impl TokenStorage {
    fn new() -> Self {
        TokenStorage {
            collection_token: None,
            disbursement_token: None,
            remittance_token: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ProductType {
    Collection,
    Disbursement,
    Remittance,
}

pub struct TokenManager;

impl TokenManager {
    pub async fn get_valid_token(product_type: ProductType) -> Option<TokenResponse> {
        let storage = TOKEN_STORAGE.read().await;
        let token = match product_type {
            ProductType::Collection => &storage.collection_token,
            ProductType::Disbursement => &storage.disbursement_token,
            ProductType::Remittance => &storage.remittance_token,
        };

        if let Some(token) = token {
            if Self::is_token_valid(token) {
                return Some(token.clone());
            }
        }
        None
    }

    pub async fn store_token(product_type: ProductType, token: TokenResponse) {
        let mut storage = TOKEN_STORAGE.write().await;
        match product_type {
            ProductType::Collection => storage.collection_token = Some(token),
            ProductType::Disbursement => storage.disbursement_token = Some(token),
            ProductType::Remittance => storage.remittance_token = Some(token),
        }
    }

    pub async fn clear_token(product_type: ProductType) {
        let mut storage = TOKEN_STORAGE.write().await;
        match product_type {
            ProductType::Collection => storage.collection_token = None,
            ProductType::Disbursement => storage.disbursement_token = None,
            ProductType::Remittance => storage.remittance_token = None,
        }
    }

    fn is_token_valid(token: &TokenResponse) -> bool {
        let now = Utc::now();
        let expiry_time = token.created_at.unwrap() + chrono::Duration::seconds(token.expires_in as i64);
        now < expiry_time
    }
}