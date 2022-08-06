use actix_web::{error, web, Error, HttpResponse};
use actix_session::{Session};
use tera::{Context};

use crate::prelude::*;

use crate::TERA;

use super::add_auth_context;

pub async fn index<T: ActixAdminAppDataTrait>(session: Session, data: web::Data<T>) -> Result<HttpResponse, Error> {
    let entity_names = &data.get_actix_admin().entity_names;
    let actix_admin = data.get_actix_admin();

    let mut ctx = Context::new();
    ctx.insert("entity_names", &entity_names);

    add_auth_context(session, actix_admin, &mut ctx);
    // TODO: show 404 if user is not logged in but auth enabled

    let body = TERA
        .render("index.html", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}