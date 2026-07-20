#[macro_use]
extern crate serde_derive;

use actix_session::Session;
use actix_web::http::header;
use actix_web::{web, HttpResponse};
use oauth2::basic::{BasicClient, BasicTokenType};
use oauth2::{
    AccessToken, AuthorizationCode, CsrfToken, EmptyExtraTokenFields, EndpointNotSet, EndpointSet,
    Scope, StandardTokenResponse, TokenResponse,
};
use oauth2::{AuthUrl, ClientId, ClientSecret, TokenUrl};

/// Fully-configured `BasicClient` type used throughout the example (auth URL,
/// token URL and redirect URL set; other endpoints unset).
pub type AzureBasicClient = BasicClient<
    EndpointSet,    // HasAuthUrl
    EndpointNotSet, // HasDeviceAuthUrl
    EndpointNotSet, // HasIntrospectionUrl
    EndpointNotSet, // HasRevocationUrl
    EndpointSet,    // HasTokenUrl
>;

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
pub struct UserInfo {
    userPrincipalName: String,
}

// AppDataTrait
pub trait AppDataTrait {
    fn get_oauth(&self) -> &AzureBasicClient;
    fn get_http_client(&self) -> &reqwest::Client;
}

#[derive(Clone, Debug)]
pub struct AzureAuth {
    auth_url: AuthUrl,
    token_url: TokenUrl,
    client_id: ClientId,
    client_secret: ClientSecret,
}

impl AzureAuth {
    pub fn new(oauth2_server: &String, client_id: &String, client_secret: &String) -> Self {
        AzureAuth {
            auth_url: AuthUrl::new(format!("https://{}/oauth2/v2.0/authorize", oauth2_server))
                .expect("Invalid authorization endpoint URL"),
            token_url: TokenUrl::new(format!("https://{}/oauth2/v2.0/token", oauth2_server))
                .expect("Invalid token endpoint URL"),
            client_id: ClientId::new(client_id.clone()),
            client_secret: ClientSecret::new(client_secret.clone()),
        }
    }

    pub fn get_api_base_url() -> &'static str {
        "https://graph.microsoft.com/v1.0"
    }

    /// Build the fully configured `BasicClient` (auth + token URLs set).
    pub fn get_oauth_client(self) -> AzureBasicClient {
        BasicClient::new(self.client_id)
            .set_client_secret(self.client_secret)
            .set_auth_uri(self.auth_url)
            .set_token_uri(self.token_url)
    }

    /// A reqwest client suitable for use with `oauth2::request_async`.
    /// Redirects are disabled to prevent SSRF.
    pub fn build_http_client() -> reqwest::Client {
        reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("failed to build reqwest client")
    }

    pub fn create_scope<T: AppDataTrait + 'static>(self) -> actix_web::Scope {
        web::scope("/azure-auth")
            .route("/login", web::get().to(login::<T>))
            .route("/logout", web::get().to(logout))
            .route("/auth", web::get().to(auth::<T>))
    }
}

pub async fn login<T: AppDataTrait>(data: web::Data<T>) -> HttpResponse {
    // Generate the authorization URL to which we'll redirect the user.
    let (auth_url, _csrf_token) = data
        .get_oauth()
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .add_scope(Scope::new("offline_access".to_string()))
        .url();

    HttpResponse::Found()
        .append_header((header::LOCATION, auth_url.to_string()))
        .finish()
}

pub async fn logout(session: Session) -> HttpResponse {
    session.remove("user_info");
    HttpResponse::Found()
        .append_header((header::LOCATION, "/admin/".to_string()))
        .finish()
}

async fn read_user(api_base_url: &str, access_token: &AccessToken) -> UserInfo {
    // With oauth2 v5 the caller supplies their own HTTP client, so we use
    // reqwest directly for arbitrary Graph API calls too.
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/me", api_base_url))
        .bearer_auth(access_token.secret())
        .send()
        .await
        .expect("Request failed")
        .bytes()
        .await
        .expect("Failed to read response body");

    serde_json::from_slice(&resp).unwrap()
}

#[derive(Deserialize)]
pub struct AuthRequest {
    code: String,
    state: String,
}

/// The token response returned by `BasicClient::exchange_code` in oauth2 v5.
type AzureTokenResponse = StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>;

pub async fn auth<T: AppDataTrait>(
    session: Session,
    data: web::Data<T>,
    params: web::Query<AuthRequest>,
) -> HttpResponse {
    let code = AuthorizationCode::new(params.code.clone());
    let _state = CsrfToken::new(params.state.clone());
    let api_base_url = AzureAuth::get_api_base_url();

    // Exchange the code with a token.
    let token: AzureTokenResponse = data
        .get_oauth()
        .exchange_code(code)
        .request_async(data.get_http_client())
        .await
        .expect("exchange_code failed");

    let user_info = read_user(api_base_url, token.access_token()).await;

    session.insert("user_info", &user_info).unwrap();

    HttpResponse::Found()
        .append_header(("location", "/admin/"))
        .finish()
}
