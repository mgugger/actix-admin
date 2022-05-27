use actix_web::{error, web, Error, HttpResponse};
use tera::{Context};

use crate::AppDataTrait;
use crate::TERA;

pub async fn index<T: AppDataTrait>(data: web::Data<T>) -> Result<HttpResponse, Error> {
    let entity_names = &data.get_actix_admin().entity_names;
    let mut ctx = Context::new();
    ctx.insert("entity_names", &entity_names);

    let body = TERA
        .render("index.html", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}