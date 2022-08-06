use actix_web::{error, web, Error, HttpResponse};
use actix_session::{Session};
use tera::{Context};

use crate::prelude::*;

use crate::TERA;

pub async fn index<T: ActixAdminAppDataTrait>(session: Session, data: web::Data<T>) -> Result<HttpResponse, Error> {
    let entity_names = &data.get_actix_admin().entity_names;
    let actix_admin = data.get_actix_admin();

    let mut ctx = Context::new();
    ctx.insert("entity_names", &entity_names);

    let enable_auth = &actix_admin.configuration.enable_auth;
    ctx.insert("enable_auth", &enable_auth);
    if *enable_auth {
        println!("auth enabled");
        let func = &actix_admin.configuration.user_is_logged_in.unwrap();
        ctx.insert("user_is_logged_in", &func(session));
        ctx.insert("login_link", &actix_admin.configuration.login_link);
        ctx.insert("logout_link", &actix_admin.configuration.logout_link);
    }

    let body = TERA
        .render("index.html", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}