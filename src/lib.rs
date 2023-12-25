



use std::error::Error;

use enums::environment::Environment;
use products::{provisioning::Provisioning, collection::Collection, remittance::Remittance, disbursements::Disbursements};

mod traits;
mod structs;
mod responses;
mod errors;
mod requests;
mod products;
mod enums;



pub struct Momo {
    pub url: String,
    environment: Environment,
    api_user: String,
    api_key: String,
}

impl Momo {
    /*
        create a new instance of MTNMoney
        @param url
        @param api_user UUID of the api user, must be created first using UUID::new_v4()
        @param environment
        @return MTNMoney
     */
    pub async fn new(url: String, api_user: String, environment: Environment) -> Result<Momo, Box<dyn Error>> {
        dotenv::dotenv().ok();
        let provisioning = Provisioning::new(url.clone());
        let _create_sandbox = provisioning.create_sandox(&api_user).await?;
        let api = provisioning.create_api_information(&api_user).await?;
       Ok(
            Momo{
                url,
                environment,
                api_user,
                api_key: api.api_key,
            }
       )
    }

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
