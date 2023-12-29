use crate::responses::{token_response::TokenResponse, oauth2tokenresponse::OAuth2TokenResponse, bcauthorize_response::BCAuthorizeResponse};


pub trait MOMOAuthorization {
    /*
    This operation is used to create an access token which can then be used to authorize and authenticate towards the other end-points of the API.
    @return TokenResponse
     */
    fn create_access_token(&self) ->impl std::future::Future<Output = Result<TokenResponse, Box<dyn std::error::Error>>> + Send;
    /*
        This operation is used to claim a consent by the account holder for the requested scopes.
        @return OAuth2TokenResponse
     */
    fn create_o_auth_2_token(&self, auth_req_id: String) ->impl std::future::Future<Output = Result<OAuth2TokenResponse, Box<dyn std::error::Error>>> + Send;
    /*
    This operation is used to claim a consent by the account holder for the requested scopes.
    @return BCAuthorizeResponse
     */
    fn bc_authorize(&self, msisdn: String, callback_url: Option<&str>) ->impl std::future::Future<Output = Result<BCAuthorizeResponse, Box<dyn std::error::Error>>> + Send;
}