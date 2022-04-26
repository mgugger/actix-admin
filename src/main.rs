extern crate serde_derive;

use actix_session::{Session, CookieSession};
use actix_web::{web, App, HttpResponse, HttpServer, HttpRequest, Error};
use tera::{ Tera, Context};
use oauth2::basic::BasicClient;
use oauth2::{ RedirectUrl };
use std::time::{Duration};
use std::env;
use sea_orm::{{ DatabaseConnection, ConnectOptions }};
use std::any::Any;
use actix_admin::{ AppDataTrait as ActixAdminAppDataTrait, ActixAdminViewModelTrait };
use azure_auth::{ AzureAuth, UserInfo, AppDataTrait as AzureAuthAppDataTrait };

mod entity;
use entity::{ Post, Comment };

#[derive(Debug, Clone)]
pub struct AppState {
    pub oauth: BasicClient,
    pub tmpl: Tera,
    pub db: DatabaseConnection,
}

impl ActixAdminAppDataTrait for AppState {
    fn get_db(&self) -> &DatabaseConnection {
        &self.db
    }
}

impl AzureAuthAppDataTrait for AppState {
    fn get_oauth(&self) -> &BasicClient {
        &self.oauth
    }
}

async fn index(session: Session, data: web::Data<AppState>) -> HttpResponse {
    let login = session.get::<UserInfo>("user_info").unwrap();
    let web_auth_link = if login.is_some() { "/auth/logout" } else { "/auth/login" };

    let mut ctx = Context::new();
    ctx.insert("web_auth_link", web_auth_link);
    let rendered = data.tmpl.render("index.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[actix_rt::main]
async fn main() {
    dotenv::dotenv().ok();
    let oauth2_client_id = env::var("OAUTH2_CLIENT_ID").expect("Missing the OAUTH2_CLIENT_ID environment variable.");
    let oauth2_client_secret = env::var("OAUTH2_CLIENT_SECRET").expect("Missing the OAUTH2_CLIENT_SECRET environment variable.");
    let oauth2_server = env::var("OAUTH2_SERVER").expect("Missing the OAUTH2_SERVER environment variable.");
    let azure_auth = AzureAuth::new(&oauth2_server, &oauth2_client_id, &oauth2_client_secret);

    // Set up the config for the OAuth2 process.
    let client = azure_auth.clone().get_oauth_client()
    // This example will be running its own server at 127.0.0.1:5000.
    .set_redirect_uri(
        RedirectUrl::new("http://localhost:5000/auth".to_string())
            .expect("Invalid redirect URL"),
    );

    let tera =
    Tera::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")
    ).unwrap();

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
        tmpl: tera,
        db: conn,
    };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(CookieSession::signed(&[0; 32]).secure(false))
            .route("/", web::get().to(index))
            .service(azure_auth.clone().create_scope(&app_state))
            .service(
                actix_admin::ActixAdmin::new()
                    .create_scope(&app_state)
                    .service(
                        web::scope(&format!("/{}", "posts")).route("/list", web::get().to(Post::list::<AppState>)),
                    )
            )
    })
    .bind("127.0.0.1:5000")
    .expect("Can not bind to port 5000")
    .run()
    .await
    .unwrap();
}