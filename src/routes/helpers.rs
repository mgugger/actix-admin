use actix_session::{Session};
use tera::{Context};

use crate::prelude::*;
use crate::TERA;
use actix_web::{error, Error, HttpResponse};


pub fn add_auth_context(session: &Session, actix_admin: &ActixAdmin, ctx: &mut Context) {
    let enable_auth = &actix_admin.configuration.enable_auth;
    ctx.insert("enable_auth", &enable_auth);
    if *enable_auth {
        let func = &actix_admin.configuration.user_is_logged_in.unwrap();
        ctx.insert("user_is_logged_in", &func(session));
        ctx.insert("login_link", &actix_admin.configuration.login_link);
        ctx.insert("logout_link", &actix_admin.configuration.logout_link);
    }
}

pub fn user_can_access_page<E: ActixAdminViewModelAccessTrait>(session: &Session, actix_admin: &ActixAdmin) -> bool {
    let auth_is_enabled = &actix_admin.configuration.enable_auth;
    let user_is_logged_in = &actix_admin.configuration.user_is_logged_in;
    let user_can_access_viewmodel = E::user_can_access(session);

    match (auth_is_enabled, user_can_access_viewmodel, user_is_logged_in) {
        (true, true, Some(auth_func)) => auth_func(session),
        (true, false, _) => false,
        (true, _, None) => false,
        (false, _, _) => true
    }
}

pub fn render_unauthorized(ctx: &Context) -> Result<HttpResponse, Error> {
    let body = TERA
            .render("unauthorized.html", &ctx)
            .map_err(|err| error::ErrorInternalServerError(err))?;
    Ok(HttpResponse::Unauthorized().content_type("text/html").body(body))
}