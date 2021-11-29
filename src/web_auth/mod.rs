use actix_session::{Session};
use actix_web::http::header;
use actix_web::{web, HttpResponse};
use http::{HeaderMap, Method};
use oauth2::reqwest::async_http_client;
use oauth2::{
    AccessToken, AuthorizationCode, CsrfToken, //PkceCodeChallenge,
    Scope, TokenResponse
};
use std::str;
use url::Url;

pub fn login(data: web::Data<super::AppState>) -> HttpResponse {
    // Create a PKCE code verifier and SHA-256 encode it as a code challenge.
    // let (_pkce_code_challenge, _pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();
    // Generate the authorization URL to which we'll redirect the user.
    let (auth_url, _csrf_token) = &data
        .oauth
        .authorize_url(CsrfToken::new_random)
        // Set the desired scopes.
        .add_scope(Scope::new("openid".to_string()))
        // Set the PKCE code challenge, need to pass verifier to /auth.
        //.set_pkce_challenge(pkce_code_challenge)
        .url();

    HttpResponse::Found()
        .append_header((header::LOCATION, auth_url.to_string()))
        .finish()
}

pub fn logout(session: Session) -> HttpResponse {
    session.remove("user_info");
    HttpResponse::Found()
        .append_header((header::LOCATION, "/".to_string()))
        .finish()
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
pub struct UserInfo {
    mail: String,
    userPrincipalName: String,
    displayName: String,
    givenName: String,
    surname: String,
    id: String,
}

async fn read_user(api_base_url: &str, access_token: &AccessToken) -> UserInfo {
    let url = Url::parse(format!("{}/me", api_base_url).as_str()).unwrap();

    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        format!("Bearer {}", access_token.secret()).parse().unwrap(),
    );

    let resp = async_http_client(oauth2::HttpRequest {
        url,
        method: Method::GET,
        headers: headers,
        body: Vec::new(),
    })
    .await
    .expect("Request failed");

    let s: &str = match str::from_utf8(&resp.body) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };

    println!("{} {}", &resp.status_code, s);
    serde_json::from_slice(&resp.body).unwrap()
}

#[derive(Deserialize)]
pub struct AuthRequest {
    code: String,
    state: String,
}

pub async fn auth(
    session: Session,
    data: web::Data<super::AppState>,
    params: web::Query<AuthRequest>,
) -> HttpResponse {
    let code = AuthorizationCode::new(params.code.clone());
    let _state = CsrfToken::new(params.state.clone());

    // Exchange the code with a token.
    let token = &data
        .oauth
        .exchange_code(code)
        //.set_pkce_verifier()
        .request_async(async_http_client)
        .await
        .expect("exchange_code failed");

    let user_info = read_user(&data.api_base_url, token.access_token()).await;

    session.insert("user_info", &user_info).unwrap();

    HttpResponse::Found().append_header(("location", "/")).finish()
}
