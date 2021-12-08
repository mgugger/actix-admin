#[macro_use]
extern crate serde_derive;

use actix_session::{Session, CookieSession};
use actix_web::{web, App, HttpResponse, HttpServer};
use tera::{ Tera, Context};
use oauth2::basic::BasicClient;
use oauth2::{
    AuthUrl, ClientId, ClientSecret,
    RedirectUrl, TokenUrl,
};
use std::time::{Duration};
use std::env;
use sea_orm::{{ DatabaseConnection, ConnectOptions }};

mod web_auth;
mod entity;
mod actix_admin;

#[derive(Debug, Clone)]
pub struct AppState {
    pub oauth: BasicClient,
    pub api_base_url: String,
    pub tmpl: Tera,
    pub db: DatabaseConnection
}

fn index(session: Session, data: web::Data<AppState>) -> HttpResponse {
    let login = session.get::<web_auth::UserInfo>("user_info").unwrap();
    let web_auth_link = if login.is_some() { "logout" } else { "login" };

    let mut ctx = Context::new();
    ctx.insert("web_auth_link", web_auth_link);
    let rendered = data.tmpl.render("index.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[actix_rt::main]
async fn main() {
    dotenv::dotenv().ok();
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

    let tera =
    Tera::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")
    ).unwrap();

    dotenv::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let mut opt = ConnectOptions::new(db_url);
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .sqlx_logging(true);

    let conn = sea_orm::Database::connect(opt).await.unwrap();
    let _ = entity::create_post_table(&conn).await;

    let app_state = AppState {
        oauth: client,
        api_base_url,
        tmpl: tera,
        db: conn
    };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(CookieSession::signed(&[0; 32]).secure(false))
            .service(actix_admin::admin_scope())
            .route("/", web::get().to(index))
            .route("/login", web::get().to(web_auth::login))
            .route("/logout", web::get().to(web_auth::logout))
            .route("/auth", web::get().to(web_auth::auth))
    })
    .bind("127.0.0.1:5000")
    .expect("Can not bind to port 5000")
    .run()
    .await
    .unwrap();
}