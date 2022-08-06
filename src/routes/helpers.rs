use actix_session::{Session};
use tera::{Context};

use crate::prelude::*;

pub fn add_auth_context(session: Session, actix_admin: &ActixAdmin, ctx: &mut Context) {
    let enable_auth = &actix_admin.configuration.enable_auth;
    ctx.insert("enable_auth", &enable_auth);
    if *enable_auth {
        let func = &actix_admin.configuration.user_is_logged_in.unwrap();
        ctx.insert("user_is_logged_in", &func(session));
        ctx.insert("login_link", &actix_admin.configuration.login_link);
        ctx.insert("logout_link", &actix_admin.configuration.logout_link);
    }
}