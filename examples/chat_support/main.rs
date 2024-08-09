extern crate serde_derive;

use actix_admin::prelude::*;
use actix_web::{http::Error, middleware, web, App, HttpResponse, HttpServer};
use ollama_rs::generation::completion::{request::GenerationRequest, GenerationContext};
use ollama_rs::Ollama;
use sea_orm::ConnectOptions;
use std::time::Duration;
use tera::{Tera, Context};

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

#[derive(serde::Deserialize)]
struct SupportForm {
    question: String,
    context: String
}

async fn support_post(
    session: Session,
    tera: web::Data<Tera>,
    actix_admin: web::Data<ActixAdmin>,
    form: web::Form<SupportForm>, // Add this parameter to extract form data
) -> Result<HttpResponse, Error> {
    let ollama = Ollama::default();
    let model = "llama3.1".to_string();
    // naive context, better use GenerationContext
    let prompt = format!("Context: {} Question: {}", form.context, form.question);
    println!("{}", prompt);
    let request = GenerationRequest::new(model, prompt);
    let res = ollama.generate(request).await;
    
    if let Ok(res) = res {
        let mut ctx = Context::new();
        ctx.extend(get_admin_ctx(session, &actix_admin));
        ctx.insert("answer", res.response.as_str());
        let body = tera.into_inner().render("chat_answer.html", &ctx).unwrap();
        Ok(HttpResponse::Ok().content_type("text/html").body(body))
    } else {
        Ok(HttpResponse::InternalServerError().body("Failed generating answer"))
    }
}

async fn custom_index(
    session: Session,
    tera: web::Data<Tera>,
    actix_admin: web::Data<ActixAdmin>
) -> Result<HttpResponse, Error> {    
    let ctx = get_admin_ctx(session, &actix_admin);
    let body = tera.render("custom_index.html", &ctx).unwrap(); 
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
        base_path: "/absproxy/5000/admin",
    };

    let mut admin_builder = ActixAdminBuilder::new(configuration);

    let _support_route = admin_builder.add_support_handler("/support", web::get().to(support));
    let _support_route_post = admin_builder.add_support_handler("/support", web::post().to(support_post));
    let _custom_index = admin_builder.add_custom_handler_for_index(web::get().to(custom_index));

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

    println!("The admin interface is available at http://localhost:5000/absproxy/5000/admin");

    HttpServer::new(move || {
        let actix_admin_builder = create_actix_admin_builder();

        // create new tera instance and extend with actix admin templates
        let mut tera = Tera::parse(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/chat_support/templates/*.html")).unwrap();
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
