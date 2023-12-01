use actix_session::Session;
use tera::Context;

use crate::prelude::*;
use actix_web::{error, Error, HttpResponse};

pub fn add_auth_context(session: &Session, actix_admin: &ActixAdmin, ctx: &mut Context) {
    let enable_auth = &actix_admin.configuration.enable_auth;
    ctx.insert("enable_auth", &enable_auth);
    ctx.insert("navbar_title", &actix_admin.configuration.navbar_title);
    ctx.insert("base_path", &actix_admin.configuration.base_path);
    if *enable_auth {
        let func = &actix_admin.configuration.user_is_logged_in.unwrap();
        ctx.insert("user_is_logged_in", &func(session));
        ctx.insert("login_link", &actix_admin.configuration.login_link.as_ref().unwrap_or(&String::new()));
        ctx.insert("logout_link", &actix_admin.configuration.logout_link.as_ref().unwrap_or(&String::new()));
    }
}

pub fn user_can_access_page(session: &Session, actix_admin: &ActixAdmin, view_model: &ActixAdminViewModel) -> bool {
    let auth_is_enabled = &actix_admin.configuration.enable_auth;
    let user_is_logged_in = &actix_admin.configuration.user_is_logged_in;
    let user_can_access_view_model = &view_model.user_can_access;

    match (auth_is_enabled, user_is_logged_in, user_can_access_view_model) {
        (true, Some(auth_func), Some(view_model_access_func)) => auth_func(session) && view_model_access_func(session),
        (true, Some(auth_func), _) => auth_func(session),
        (_, _, _) => !auth_is_enabled,
    }
}

pub fn render_unauthorized(ctx: &Context, actix_admin: &ActixAdmin) -> Result<HttpResponse, Error> {
    let body = actix_admin.tera
            .render("unauthorized.html", &ctx)
            .map_err(|err| error::ErrorInternalServerError(err))?;
    Ok(HttpResponse::Unauthorized().content_type("text/html").body(body))
}