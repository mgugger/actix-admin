use actix_web::{web, guard, HttpRequest, HttpResponse};

async fn index(data: web::Data<super::AppState>) -> &'static str {
    "Welcome!"
}

async fn list(data: web::Data<super::AppState>) -> &'static str {
    "List!"
}

fn entity_scope() -> actix_web::Scope {
    let scope = web::scope("/entity")
    .route("/list", web::get().to(list));
    scope
}

pub fn admin_scope() -> actix_web::Scope {
    let scope = web::scope("/admin")
        .route("/", web::get().to(index))
        .service(entity_scope());
    scope
}