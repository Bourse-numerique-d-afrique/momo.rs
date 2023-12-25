use crate::responses::{token_response::TokenResponse, oauth2tokenresponse::OAuth2TokenResponse, bcauthorize_response::BCAuthorizeResponse};


pub trait MOMOAuthorization {
    /*
    This operation is used to create an access token which can then be used to authorize and authenticate towards the other end-points of the API.
    */
    fn encode(&self, user_id: &str, user_api_key: &str) -> String;
    /*
    This operation is used to create an access token which can then be used to authorize and authenticate towards the other end-points of the API.
    @return TokenResponse
     */
    async fn create_access_token(&self) -> Result<TokenResponse, Box<dyn std::error::Error>>;
    /*
        This operation is used to claim a consent by the account holder for the requested scopes.
        @return OAuth2TokenResponse
     */
    async fn create_o_auth_2_token(&self) -> Result<OAuth2TokenResponse, Box<dyn std::error::Error>>;
    /*
    This operation is used to claim a consent by the account holder for the requested scopes.
    @return BCAuthorizeResponse
     */
    async fn bc_authorize(&self) -> Result<BCAuthorizeResponse, Box<dyn std::error::Error>>;
}