#!/usr/bin/env rust-script

//! Test script for MTN MoMo callback server
//! 
//! This script demonstrates how to:
//! 1. Start the callback server
//! 2. Make MTN MoMo API requests with callback URLs
//! 3. Receive and process callbacks

use std::env;
use mtnmomo::{Momo, RequestToPay, Party, PartyIdType, Currency};
use tokio;
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    let mtn_url = env::var("MTN_URL").expect("MTN_URL must be set");
    let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
    let secondary_key = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
    let callback_host = env::var("MTN_CALLBACK_HOST").expect("MTN_CALLBACK_HOST must be set");
    
    // Your public callback server URL (replace with your actual URL)
    let callback_server_url = env::var("CALLBACK_SERVER_URL")
        .unwrap_or_else(|_| "https://your-domain.com".to_string());
    
    println!("🚀 Starting MTN MoMo callback test...");
    
    // Initialize MTN MoMo
    let momo = Momo::new_with_provisioning(mtn_url, primary_key.clone(), &callback_host).await?;
    let collection = momo.collection(primary_key, secondary_key);
    
    // Create a test payment request
    let payer = Party {
        party_id_type: PartyIdType::MSISDN,
        party_id: "+256123456789".to_string(), // Test phone number
    };
    
    let request = RequestToPay::new(
        "100".to_string(),
        Currency::UGX,
        payer,
        "Test payment from callback server".to_string(),
        "Testing callback functionality".to_string(),
    );
    
    // Set callback URL to your server
    let callback_url = format!("{}/collection_request_to_pay/REQUEST_TO_PAY", callback_server_url);
    
    println!("📞 Making request to pay with callback URL: {}", callback_url);
    
    // Make the payment request
    match collection.request_to_pay(request, Some(callback_url)).await {
        Ok(result) => {
            println!("✅ Payment request successful!");
            println!("📋 Result: {:?}", result);
            println!("⏳ Now waiting for callback on your server...");
            println!("💡 Check your callback server logs for incoming callbacks");
        }
        Err(e) => {
            println!("❌ Payment request failed: {}", e);
            return Err(e);
        }
    }
    
    Ok(())
}