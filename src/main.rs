#[macro_use]
extern crate serde_derive;
use std::str;
use actix_session::{Session, CookieSession};
use actix_web::http::header;
use actix_web::{web, App, HttpResponse, HttpServer};
use http::{HeaderMap, Method};
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use oauth2::{
    AccessToken, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use std::env;
use url::Url;

struct AppState {
    oauth: BasicClient,
    api_base_url: String,
}

fn index(session: Session) -> HttpResponse {
    let login = session.get::<String>("login").unwrap();
    let link = if login.is_some() { "logout" } else { "login" };

    let html = format!(
        r#"<html>
        <head><title>OAuth2 Test</title></head>
        <body>
            {} <a href="/{}">{}</a>
        </body>
    </html>"#,
        login.unwrap_or("".to_string()),
        link,
        link
    );

    HttpResponse::Ok().body(html)
}

fn login(data: web::Data<AppState>) -> HttpResponse {
    // Create a PKCE code verifier and SHA-256 encode it as a code challenge.
    let (pkce_code_challenge, _pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();
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
        .header(header::LOCATION, auth_url.to_string())
        .finish()
}

fn logout(session: Session) -> HttpResponse {
    session.remove("login");
    HttpResponse::Found()
        .header(header::LOCATION, "/".to_string())
        .finish()
}

#[derive(Deserialize, Debug)]
pub struct UserInfo {
    mail: String,
    userPrincipalName: String,
    displayName: String,
    givenName: String,
    surname: String,
    id: String
}

async fn read_user(api_base_url: &str, access_token: &AccessToken) -> UserInfo {
    let url = Url::parse(
        format!(
            "{}/me",
            api_base_url
        )
        .as_str(),
    )
    .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Authorization", format!("Bearer {}", access_token.secret()).parse().unwrap());
    
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

    println!("{} {}", &resp.status_code , s);
    serde_json::from_slice(&resp.body).unwrap()
}

#[derive(Deserialize)]
struct AuthRequest {
    code: String,
    state: String,
}

async fn auth(
    session: Session,
    data: web::Data<AppState>,
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

    //session.insert("login", user_info.username.clone()).unwrap();

    let html = format!(
        r#"<html>
        <head><title>OAuth2 Test</title></head>
        <body>
            User info:
            <pre>{:?}</pre>
            <a href="/">Home</a>
        </body>
    </html>"#,
        user_info
    );
    HttpResponse::Ok().body(html)
}

#[actix_rt::main]
async fn main() {
    HttpServer::new(|| {
        let oauth2_client_id = ClientId::new(
            env::var("OAUTH2_CLIENT_ID")
                .expect("Missing the OAUTH2_CLIENT_ID environment variable."),
        );
        let oauth2_client_secret = ClientSecret::new(
            env::var("OAUTH2_CLIENT_SECRET")
                .expect("Missing the OAUTH2_CLIENT_SECRET environment variable."),
        );
        let oauth2_server =
            env::var("OAUTH2_SERVER").expect("Missing the OAUTH2_SERVER environment variable.");
        
        let auth_url = AuthUrl::new(format!("https://{}/oauth2/v2.0/authorize", oauth2_server))
            .expect("Invalid authorization endpoint URL");
        let token_url = TokenUrl::new(format!("https://{}/oauth2/v2.0/token", oauth2_server))
            .expect("Invalid token endpoint URL");
        
        let api_base_url = "https://graph.microsoft.com/v1.0".to_string();

        // Set up the config for the OAuth2 process.
        let client = BasicClient::new(
            oauth2_client_id,
            Some(oauth2_client_secret),
            auth_url,
            Some(token_url),
        )
        // This example will be running its own server at 127.0.0.1:5000.
        .set_redirect_uri(
            RedirectUrl::new("http://localhost:5000/auth".to_string())
                .expect("Invalid redirect URL"),
        );

        let app_state = web::Data::new(
            AppState {
                oauth: client,
                api_base_url,
            }
        );

        App::new()
            .app_data(app_state)
            .wrap(CookieSession::signed(&[0; 32]).secure(false))
            .route("/", web::get().to(index))
            .route("/login", web::get().to(login))
            .route("/logout", web::get().to(logout))
            .route("/auth", web::get().to(auth))
    })
    .bind("127.0.0.1:5000")
    .expect("Can not bind to port 5000")
    .run()
    .await
    .unwrap();
}