extern crate serde_derive;

use actix_admin::prelude::*;
use actix_web::{web, App, HttpServer, middleware};
use sea_orm::ConnectOptions;
use std::time::Duration;
mod entity;
use entity::{Post, Comment, User};

fn create_actix_admin_builder() -> ActixAdminBuilder {
    let configuration = ActixAdminConfiguration {
        enable_auth: false,
        user_is_logged_in: None,
        login_link: None,
        logout_link: None,
        file_upload_directory: "./file_uploads",
        navbar_title: "ActixAdmin Example",
        user_tenant_ref: None,
        base_path: "/admin/"
    };

    let mut admin_builder = ActixAdminBuilder::new(configuration);
    
    let post_view_model = ActixAdminViewModel::from(Post);
    admin_builder.add_entity::<Post>(&post_view_model);

    let some_category = "Group";
    let comment_view_model = ActixAdminViewModel::from(Comment);
    admin_builder.add_entity_to_category::<Comment>(&comment_view_model, some_category);
    let user_view_model = ActixAdminViewModel::from(User);
    admin_builder.add_entity_to_category::<User>(&user_view_model, some_category);

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
    let conn: sea_orm::DatabaseConnection = sea_orm::Database::connect(opt).await.unwrap();
    let _ = entity::create_post_table(&conn).await;

    println!("The admin interface is available at http://localhost:5000/admin/");

    HttpServer::new(move || {

        let actix_admin_builder = create_actix_admin_builder();

        App::new()
            .app_data(web::Data::new(actix_admin_builder.get_actix_admin()))
            .app_data(web::Data::new(conn.clone()))
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