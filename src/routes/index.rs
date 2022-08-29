use actix_web::{error, web, Error, HttpResponse};
use actix_session::{Session};
use tera::{Context};

use crate::prelude::*;

use crate::TERA;

use super::{ add_auth_context };

pub fn get_admin_ctx<T: ActixAdminAppDataTrait>(session: Session, data: &web::Data<T>) -> Context {
    let actix_admin = data.get_actix_admin();

    let mut ctx = Context::new();
    ctx.insert("entity_names", &actix_admin.entity_names);

    add_auth_context(&session, actix_admin, &mut ctx);

    ctx
}

pub async fn index<T: ActixAdminAppDataTrait>(session: Session, data: web::Data<T>) -> Result<HttpResponse, Error> {
    let actix_admin = data.get_actix_admin();

    let mut ctx = Context::new();
    ctx.insert("entity_names", &actix_admin.entity_names);

    add_auth_context(&session, actix_admin, &mut ctx);

    let body = TERA
        .render("index.html", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}