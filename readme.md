### MOMO.rs is a Rust library for the MOMO payment gateway.
[![.github/workflows/deployment.yml](https://github.com/Bourse-numerique-d-afrique/momo.rs/actions/workflows/deployment.yml/badge.svg)](https://github.com/Bourse-numerique-d-afrique/momo.rs/actions/workflows/deployment.yml)
<p align="center">
  <img src="https://github.com/Bourse-numerique-d-afrique/momo.rs/blob/master/images/BrandGuid-mtnmomo.svg" alt="MOMO logo">
</p>


### Installation
#
#
```toml
[dependencies]
momo = "0.1.0"
```

```cli
cargo add momo
```


### Usage
### Create a new instance of momo
To create an instance of momo you need to have an api_user, url and environment (sandbox or production).
 ```rust
    use momo::Momo;

    fn main() -> () {
        let url = "https://sandbox.momodeveloper.mtn.com"; // the url of the api you are using for sandbox please use https://sandbox.momodeveloper.mtn.com
        let api_user = "api_user"; // api_user UUID of the api user, must be created first using UUID::new_v4()
        let momo = Momo::new("url", "api_user", Environment::Sandbox); // create a new instance of momo
    }
 ```
### Usage for Collections

 ```rust
    use momo::Momo;

    fn main() -> () {
        let url = "https://sandbox.momodeveloper.mtn.com"; // the url of the api you are using for sandbox please use https://sandbox.momodeveloper.mtn.com
        let api_user = "api_user"; // api_user UUID of the api user, must be created first using UUID::new_v4()
        let momo = Momo::new("url", "api_user", Environment::Sandbox); // create a new instance of momo
        // @param primary_key: the primary key of the collection given by https://momodeveloper.mtn.com when you create a collection product
        // @param secondary_key: the secondary key of the collection given by https://momodeveloper.mtn.com when you create a collection product
        let collection = momo.collection(primary_key, secondary_key); // create a new instance of collection
    }
 ```


### Usage for Disbursements

 ```rust
    use momo::Momo;

    fn main() -> () {
        let url = "https://sandbox.momodeveloper.mtn.com"; // the url of the api you are using for sandbox please use https://sandbox.momodeveloper.mtn.com
        let api_user = "api_user"; // api_user UUID of the api user, must be created first using UUID::new_v4()
        let momo = Momo::new("url", "api_user", Environment::Sandbox); // create a new instance of momo
        // @param primary_key: the primary key of the disbursement given by https://momodeveloper.mtn.com when you create a disbursement product
        // @param secondary_key: the secondary key of the disbursement given by https://momodeveloper.mtn.com when you create a disbursement product
        let disbursement = momo.disbursement(primary_key, secondary_key); // create a new instance of disbursement
    }
 ```

### Usage for Remittances

 ```rust
    use momo::Momo;

    fn main() -> () {
        let url = "https://sandbox.momodeveloper.mtn.com"; // the url of the api you are using for sandbox please use https://sandbox.momodeveloper.mtn.com
        let api_user = "api_user"; // api_user UUID of the api user, must be created first using UUID::new_v4()
        let momo = Momo::new("url", "api_user", Environment::Sandbox); // create a new instance of momo
        // @param primary_key: the primary key of the remittance given by https://momodeveloper.mtn.com when you create a remittance product
        // @param secondary_key: the secondary key of the remittance given by https://momodeveloper.mtn.com when you create a remittance product
        let remittance = momo.remittance(primary_key, secondary_key); // create a new instance of remittance
    }
 ```
