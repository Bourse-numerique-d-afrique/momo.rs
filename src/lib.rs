



use std::error::Error;

use enums::environment::Environment;
use products::{provisioning::Provisioning, collection::Collection, remittance::Remittance, disbursements::Disbursements};

pub mod traits;
pub mod structs;
pub mod responses;
pub mod errors;
pub mod requests;
pub mod products;
pub mod enums;


///# MTN Mobile Money API
/// This package provides for an easy way to connect to MTN MoMo API, it provides for the following products:
/// - Collection
/// - Disbursements
/// - Remittance
/// - Provisioning in case of sandbox environment
/// how to use:
/// ```
/// use mtnmomo::Momo;
/// use mtnmomo::enums::environment::Environment;
/// use uuid::Uuid;
/// 
/// #[tokio::main]
/// async fn main() {
///   let primary_subscription_key = Uuid::new_v4().to_string();
///   let secondary_subscription_key = Uuid::new_v4().to_string();
///   let mtn_url = "https://sandbox.momodeveloper.mtn.com";
///   let momo = Momo::new_with_provisioning(mtn_url.to_string(), primary_subscription_key.clone()).await.unwrap();
///   let collection = momo.collection(primary_subscription_key, secondary_subscription_key);
/// }
/// 
///```
/// After initializing the Momo struct, you can then use the collection, disbursement or remittance methods to initialize the respective products.
/// The products have methods that you can use to interact with the API.
/// For example, to request a payment from a customer, you can use the request_to_pay method of the Collection product.
/// ```
/// use mtnmomo::Momo;
/// use mtnmomo::enums::environment::Environment;
/// use uuid::Uuid;
/// use mtnmomo::structs::party::Party;
/// use mtnmomo::enums::party_id_type::PartyIdType;
/// use mtnmomo::enums::currency::Currency;
/// use mtnmomo::requests::request_to_pay::RequestToPay;
/// 
/// #[tokio::main]
/// async fn main() {
///   let primary_subscription_key = Uuid::new_v4().to_string();
///   let secondary_subscription_key = Uuid::new_v4().to_string();
///   let mtn_url = "https://sandbox.momodeveloper.mtn.com";
///   let momo = Momo::new_with_provisioning(mtn_url.to_string(), primary_subscription_key.clone()).await.unwrap();
///   let collection = momo.collection(primary_subscription_key, secondary_subscription_key);
/// 
///    let payer : Party = Party {
///           party_id_type: PartyIdType::MSISDN,
///          party_id: "msisdn".to_string(),
///      };
/// 
///   let request = RequestToPay::new("100".to_string(), Currency::EUR, payer, "test_payer_message".to_string(), "test_payee_note".to_string());
///   let result = collection.request_to_pay(request).await;
///   assert_eq!(result.is_ok(), true);
/// }
/// ```
/// The above code will request a payment of 100 EUR from the customer with the phone number "msisdn".
/// The customer will receive a prompt on their phone to confirm the payment.
/// If the customer confirms the payment, the payment will be processed and the customer will receive a confirmation message.
/// If the customer declines the payment, the payment will not be processed and the customer will receive a message informing them that the payment was declined.
/// The request_to_pay method returns a Result<RequestToPayResponse, Box<dyn Error>>.

pub struct Momo {
    pub url: String,
    pub environment: Environment,
    pub api_user: String,
    pub api_key: String,
}




impl Momo {
    pub async fn new(url: String, api_user: String, environment: Environment, api_key: Option<String>) -> Self {
        Momo{
            url,
            environment,
            api_user,
            api_key: api_key.unwrap(),
        }
       
    }

    pub async fn new_with_provisioning(url: String, subscription_key: String) -> Result<Momo, Box<dyn Error>> {
        let provisioning = Provisioning::new(url.clone(), subscription_key.clone());
        let _create_sandbox = provisioning.create_sandox(&subscription_key).await?;
        let api = provisioning.create_api_information(&subscription_key).await?;
        return 
            Ok(Momo{
                url,
                environment: Environment::Sandbox,
                api_user: subscription_key,
                api_key: api.api_key,
            })
        
    }


    /// # Collection
    /// This product provides a way to request payments from a customer.
    /// # Example
    /*
        create a new instance of Collection product
        @param primary_key
        @param secondary_key
        @return Collection
     */
    pub fn collection(&self, primary_key: String, secondary_key: String) -> Collection {
        Collection::new(self.url.clone(),
         self.environment.clone(),
          self.api_user.clone(),
           self.api_key.clone(),
            primary_key,
             secondary_key)
    }

    /*
        create a new instance of Disbursements product
        @param primary_key
        @param secondary_key
        @return Disbursements
     */
    pub fn disbursement(&self, primary_key: String, secondary_key: String) -> Disbursements {
        Disbursements::new(self.url.clone(),
         self.environment.clone(),
          self.api_user.clone(),
           self.api_key.clone(),
            primary_key,
             secondary_key)
    }

    /*
        create a new instance of Remittance product
        @param primary_key
        @param secondary_key
        @return Remittance
     */
    pub fn remittance(&self, primary_key: String, secondary_key: String) -> Remittance {
        Remittance::new(self.url.clone(),
         self.environment.clone(),
          self.api_user.clone(),
           self.api_key.clone(),
            primary_key,
             secondary_key)
    }

}
