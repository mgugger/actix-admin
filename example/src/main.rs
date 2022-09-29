extern crate serde_derive;

use actix_admin::prelude::*;
use actix_session::{Session, SessionMiddleware, storage::CookieSessionStore};
use actix_web::{cookie::Key, web, App, HttpResponse, HttpServer, middleware};
use azure_auth::{AppDataTrait as AzureAuthAppDataTrait, AzureAuth, UserInfo};
use oauth2::basic::BasicClient;
use oauth2::RedirectUrl;
use sea_orm::{ConnectOptions, DatabaseConnection};
use std::env;
use std::time::Duration;
use tera::{Context, Tera};
use actix_web::Error;

mod entity;
use entity::{Post, Comment};

#[derive(Clone)]
pub struct AppState {
    pub oauth: BasicClient,
    pub tmpl: Tera,
    pub db: DatabaseConnection,
    pub actix_admin: ActixAdmin,
}

impl ActixAdminAppDataTrait for AppState {
    fn get_db(&self) -> &DatabaseConnection {
        &self.db
    }
    fn get_actix_admin(&self) -> &ActixAdmin {
        &self.actix_admin
    }
}

trait AppDataTrait {
    fn get_tmpl(&self) -> &Tera;
}

impl AppDataTrait for AppState {
    fn get_tmpl(&self) -> &Tera {
        &self.tmpl
    }
}

impl AzureAuthAppDataTrait for AppState {
    fn get_oauth(&self) -> &BasicClient {
        &self.oauth
    }
}

async fn custom_handler<
    T: ActixAdminAppDataTrait + AppDataTrait,
    E: ActixAdminViewModelTrait,
>(
    session: Session,
    data: web::Data<T>,
    _text: String
) -> Result<HttpResponse, Error> {
    
    let mut ctx = Context::new();
    ctx.extend(get_admin_ctx(session, &data));

    let body = data.get_tmpl()
    .render("custom_handler.html", &ctx).unwrap();
    
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

async fn custom_index<
    T: ActixAdminAppDataTrait + AppDataTrait
>(
    session: Session,
    data: web::Data<T>,
    _text: String
) -> Result<HttpResponse, Error> {
    
    let mut ctx = Context::new();
    ctx.extend(get_admin_ctx(session, &data));

    let body = data.get_tmpl()
    .render("custom_index.html", &ctx).unwrap();
    
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}


async fn index(session: Session, data: web::Data<AppState>) -> HttpResponse {
    let login = session.get::<UserInfo>("user_info").unwrap();
    let web_auth_link = if login.is_some() {
        "azure-auth/logout"
    } else {
        "azure-auth/login"
    };

    let mut ctx = Context::new();
    ctx.insert("web_auth_link", web_auth_link);
    let rendered = data.tmpl.render("index.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

fn create_actix_admin_builder() -> ActixAdminBuilder {
    let post_view_model = ActixAdminViewModel::from(Post);
    let comment_view_model = ActixAdminViewModel::from(Comment);

    let configuration = ActixAdminConfiguration {
        enable_auth: false,
        user_is_logged_in: Some(|session: &Session| -> bool { 
             let user_info = session.get::<UserInfo>("user_info").unwrap();
             user_info.is_some()
        }),
        login_link: Some("/azure-auth/login".to_string()),
        logout_link: Some("/azure-auth/logout".to_string())
    };

    let mut admin_builder = ActixAdminBuilder::new(configuration);
    admin_builder.add_custom_handler_for_index::<AppState>(
        web::get().to(custom_index::<AppState>)
    );
    admin_builder.add_entity::<AppState, Post>(&post_view_model);

    let some_category = "Some Category";
    admin_builder.add_entity_to_category::<AppState, Comment>(&comment_view_model, some_category);
    admin_builder.add_custom_handler_for_entity_in_category::<AppState, Comment>(
        "My custom handler",
        "/custom_handler", 
        web::get().to(custom_handler::<AppState, Comment>),
        some_category
    );

    admin_builder
}

#[actix_rt::main]
async fn main() {
    dotenv::dotenv().ok();

    let actix_admin = create_actix_admin_builder().get_actix_admin();

    let oauth2_client_id;
    let oauth2_client_secret;
    let oauth2_server;

    match actix_admin.configuration.enable_auth {
        true => {
            oauth2_client_id = env::var("OAUTH2_CLIENT_ID").expect("Missing the OAUTH2_CLIENT_ID environment variable.");
            oauth2_client_secret = env::var("OAUTH2_CLIENT_SECRET").expect("Missing the OAUTH2_CLIENT_SECRET environment variable.");
            oauth2_server= env::var("OAUTH2_SERVER").expect("Missing the OAUTH2_SERVER environment variable.");
        },
        false => {
            oauth2_client_id = String::new();
            oauth2_client_secret = String::new();
            oauth2_server = String::new();
        }
    }
        
    let azure_auth = AzureAuth::new(&oauth2_server, &oauth2_client_id, &oauth2_client_secret);

    // Set up the config for the OAuth2 process.
    let client = azure_auth
        .clone()
        .get_oauth_client()
        // This example will be running its own server at 127.0.0.1:5000.
        .set_redirect_uri(
            RedirectUrl::new("http://localhost:5000/azure-auth/auth".to_string())
                .expect("Invalid redirect URL"),
        );

    let mut tera = Tera::parse(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();
    tera.extend(&TERA).unwrap();
    let _tera_res = tera.build_inheritance_chains();

    let db_url = "sqlite::memory:".to_string();
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
        actix_admin: actix_admin,
    };

    let cookie_secret_key = Key::generate();
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(SessionMiddleware::new(CookieSessionStore::default(), cookie_secret_key.clone()))
            .route("/", web::get().to(index))
            .service(azure_auth.clone().create_scope::<AppState>())
            .service(
                create_actix_admin_builder().get_scope::<AppState>()
            )
            .wrap(middleware::Logger::default())
    })
    .bind("127.0.0.1:5000")
    .expect("Can not bind to port 5000")
    .run()
    .await
    .unwrap();
}