extern crate serde_derive;

use actix_admin::prelude::*;
use actix_session::{Session, SessionMiddleware, storage::CookieSessionStore};
use actix_web::{cookie::Key, web, App, HttpResponse, HttpServer, middleware};
use azure_auth::{AppDataTrait as AzureAuthAppDataTrait, AzureAuth, UserInfo};
use oauth2::basic::BasicClient;
use oauth2::RedirectUrl;
use sea_orm::ConnectOptions;
use std::env;
use std::time::Duration;
use tera::{Context, Tera};
use actix_web::Error;

mod entity;
use entity::{Post, Comment};

#[derive(Clone)]
pub struct AppState {
    pub oauth: BasicClient,
    pub tmpl: Tera
}

impl AzureAuthAppDataTrait for AppState {
    fn get_oauth(&self) -> &BasicClient {
        &self.oauth
    }
}

async fn custom_handler(
    session: Session,
    data: web::Data<AppState>,
    actix_admin: web::Data<ActixAdmin>,
    _text: String
) -> Result<HttpResponse, Error> {
    
    let mut ctx = Context::new();
    ctx.extend(get_admin_ctx(session, &actix_admin));

    let body = data.tmpl.render("custom_handler.html", &ctx).unwrap();
    
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

async fn custom_index(
    session: Session,
    data: web::Data<AppState>,
    actix_admin: web::Data<ActixAdmin>,
    _text: String
) -> Result<HttpResponse, Error> {
    
    let mut ctx = Context::new();
    ctx.extend(get_admin_ctx(session, &actix_admin));

    let body = data.tmpl.render("custom_index.html", &ctx).unwrap();
    
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
        enable_auth: true,
        user_is_logged_in: Some(|session: &Session| -> bool { 
             let user_info = session.get::<UserInfo>("user_info").unwrap();
             user_info.is_some()
        }),
        login_link: Some("/azure-auth/login".to_string()),
        logout_link: Some("/azure-auth/logout".to_string()),
        file_upload_directory: "./file_uploads",
        navbar_title: "ActixAdmin Example",
        user_tenant_ref: None,
        base_path: "/admin",
        custom_css_paths: None,
        custom_js_paths: None
    };

    let mut admin_builder = ActixAdminBuilder::new(configuration);
    admin_builder.add_custom_handler_for_index(
         web::get().to(custom_index)
    );
    admin_builder.add_entity::<Post>(&post_view_model);
    admin_builder.add_custom_handler("Custom Route in Menu", "/custom_route_in_menu", web::get().to(custom_index), true);
    admin_builder.add_custom_handler("Custom Route not in Menu", "/custom_route_not_in_menu", web::get().to(custom_index), false);

    let some_category = "Some Category";
    admin_builder.add_entity_to_category::<Comment>(&comment_view_model, some_category);
    admin_builder.add_custom_handler_for_entity_in_category::<Comment>(
        "My custom handler",
        "/custom_handler", 
        web::get().to(custom_handler),
        some_category,
        true
    );

    admin_builder
}

#[actix_rt::main]
async fn main() {
    dotenv::from_filename("./examples/azure_auth/.env.example").ok();
    dotenv::from_filename("./examples/azure_auth/.env").ok();

    let oauth2_client_id = env::var("OAUTH2_CLIENT_ID").expect("Missing the OAUTH2_CLIENT_ID environment variable.");
    let oauth2_client_secret = env::var("OAUTH2_CLIENT_SECRET").expect("Missing the OAUTH2_CLIENT_SECRET environment variable.");
    let oauth2_server= env::var("OAUTH2_SERVER").expect("Missing the OAUTH2_SERVER environment variable.");
        
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


    let db_url = "sqlite::memory:".to_string();
    let mut opt = ConnectOptions::new(db_url);
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .sqlx_logging(true);

    let conn = sea_orm::Database::connect(opt).await.unwrap();
    let _ = entity::create_post_table(&conn).await;

    let cookie_secret_key = Key::generate();
    HttpServer::new(move || {
        let actix_admin_builder = create_actix_admin_builder();

        let actix_admin = actix_admin_builder.get_actix_admin();
        let mut tera = Tera::parse(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();
        tera.extend(&actix_admin.tera).unwrap();
        let _tera_res = tera.build_inheritance_chains();
        
        let app_state = AppState {
            oauth: client.clone(),
            tmpl: tera.clone()
        };

        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .app_data(web::Data::new(conn.clone()))
            .app_data(web::Data::new(actix_admin.clone()))
            .wrap(SessionMiddleware::new(CookieSessionStore::default(), cookie_secret_key.clone()))
            .route("/", web::get().to(index))
            .service(azure_auth.clone().create_scope::<AppState>())
            .service(
                actix_admin_builder.get_scope()
            )
            .wrap(middleware::Logger::default())
    })
    .bind("127.0.0.1:5000")
    .expect("Can not bind to port 5000")
    .run()
    .await
    .unwrap();
}