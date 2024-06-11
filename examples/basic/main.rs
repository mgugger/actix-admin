extern crate serde_derive;

use actix_admin::prelude::*;
use actix_web::{http::Error, middleware, web, App, HttpResponse, HttpServer};
use sea_orm::ConnectOptions;
use std::time::Duration;
use tera::{Tera, Context};
mod entity;
use entity::{Comment, Post, User};

async fn profile(
    session: Session,
    tera: web::Data<Tera>,
    actix_admin: web::Data<ActixAdmin>,
) -> Result<HttpResponse, Error> {
    let mut ctx = Context::new();
    ctx.extend(get_admin_ctx(session, &actix_admin));
    let body = tera.into_inner().render("profile.html", &ctx).unwrap();
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

async fn support(
    session: Session,
    tera: web::Data<Tera>,
    actix_admin: web::Data<ActixAdmin>,
) -> Result<HttpResponse, Error> {
    let mut ctx = Context::new();
    ctx.extend(get_admin_ctx(session, &actix_admin));
    let body = tera.into_inner().render("support.html", &ctx).unwrap();
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

async fn card(
    session: Session,
    tera: web::Data<Tera>,
    actix_admin: web::Data<ActixAdmin>,
    id: web::Path<i32>
) -> Result<HttpResponse, Error> {
    let mut ctx = Context::new();
    ctx.extend(get_admin_ctx(session, &actix_admin));
    ctx.insert("id", &(id.into_inner()));
    let body = tera.into_inner().render("card.html", &ctx).unwrap();
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

fn create_actix_admin_builder() -> ActixAdminBuilder {
    let configuration = ActixAdminConfiguration {
        enable_auth: true,
        user_is_logged_in: Some(|_session: &Session| -> bool { true }),
        login_link: None,
        logout_link: None,
        file_upload_directory: "./file_uploads",
        navbar_title: "ActixAdmin Example",
        user_tenant_ref: None,
        base_path: "/admin",
    };

    let mut admin_builder = ActixAdminBuilder::new(configuration);

    let post_view_model = ActixAdminViewModel::from(Post);
    admin_builder.add_entity::<Post>(&post_view_model);

    let some_category = "Group";
    let comment_view_model = ActixAdminViewModel::from(Comment);
    admin_builder.add_entity_to_category::<Comment>(&comment_view_model, some_category);
    let user_view_model = ActixAdminViewModel::from(User);
    admin_builder.add_entity_to_category::<User>(&user_view_model, some_category);

    let navbar_end_category = "navbar-end";
    admin_builder.add_custom_handler_to_category(
        "Profile",
        "/profile",
        web::get().to(profile),
        true,
        navbar_end_category,
    );

    let _support_route = admin_builder.add_support_handler("/support", web::get().to(support));
    let _card_route = admin_builder.add_custom_handler("card", "/card/{id}", web::get().to(card), false);

    let card_grid: Vec<Vec<String>> = vec![
        vec!["admin/card/1".to_string(), "admin/card/2".to_string()],
        vec!["admin/card/3".to_string()],
    ];
    admin_builder.add_card_grid("Card Grid", "/my_card_grid", card_grid, true);

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

        // create new tera instance and extend with actix admin templates
        let mut tera = Tera::parse(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/basic/templates/*.html")).unwrap();
        tera.extend(&actix_admin_builder.get_actix_admin().tera)
            .unwrap();
        let _tera_res = tera.build_inheritance_chains();

        App::new()
            .app_data(web::Data::new(tera))
            .app_data(web::Data::new(actix_admin_builder.get_actix_admin()))
            .app_data(web::Data::new(conn.clone()))
            .service(actix_admin_builder.get_scope())
            .wrap(middleware::Logger::default())
    })
    .bind("127.0.0.1:5000")
    .expect("Can not bind to port 5000")
    .run()
    .await
    .unwrap();
}
