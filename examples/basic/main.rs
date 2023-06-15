extern crate serde_derive;

use actix_admin::prelude::*;
use actix_web::{web, App, HttpServer, middleware};
use sea_orm::{ConnectOptions, DatabaseConnection};
use std::time::Duration;
mod entity;
use entity::{Post, Comment, User};

#[derive(Clone)]
pub struct AppState {
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

fn create_actix_admin_builder() -> ActixAdminBuilder {
    let configuration = ActixAdminConfiguration {
        enable_auth: false,
        user_is_logged_in: None,
        login_link: None,
        logout_link: None,
        file_upload_directory: "./file_uploads",
        navbar_title: "ActixAdmin Example"
    };

    let mut admin_builder = ActixAdminBuilder::new(configuration);
    
    let post_view_model = ActixAdminViewModel::from(Post);
    admin_builder.add_entity::<AppState, Post>(&post_view_model);

    let some_category = "Group";
    let comment_view_model = ActixAdminViewModel::from(Comment);
    admin_builder.add_entity_to_category::<AppState, Comment>(&comment_view_model, some_category);
    let user_view_model = ActixAdminViewModel::from(User);
    admin_builder.add_entity_to_category::<AppState, User>(&user_view_model, some_category);

    admin_builder
}

fn get_db_options() -> ConnectOptions {
    let db_url = "sqlite::memory:".to_string();
    let mut opt = ConnectOptions::new(db_url);
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .sqlx_logging(true);
    opt
}

#[actix_rt::main]
async fn main() {
    let opt = get_db_options();
    let conn = sea_orm::Database::connect(opt).await.unwrap();
    let _ = entity::create_post_table(&conn).await;

    println!("The admin interface is available at http://localhost:5000/admin/");

    HttpServer::new(move || {

        let actix_admin_builder = create_actix_admin_builder();

        let app_state = AppState {
            db: conn.clone(),
            actix_admin: actix_admin_builder.get_actix_admin(),
        };

        App::new()
            .app_data(web::Data::new(app_state))
            .service(
                actix_admin_builder.get_scope::<AppState>()
            )
            .wrap(middleware::Logger::default())
    })
    .bind("127.0.0.1:5000")
    .expect("Can not bind to port 5000")
    .run()
    .await
    .unwrap();
}