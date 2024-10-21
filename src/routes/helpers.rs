use actix_session::Session;
use serde_derive::Deserialize;
use tera::Context;

use crate::{prelude::*, ActixAdminNotification};
use actix_web::{error, web::Query, Error, HttpRequest, HttpResponse};

use super::{Params, DEFAULT_ENTITIES_PER_PAGE};

pub fn add_auth_context(session: &Session, actix_admin: &ActixAdmin, ctx: &mut Context) {
    let enable_auth = &actix_admin.configuration.enable_auth;
    ctx.insert("enable_auth", &enable_auth);
    ctx.insert("custom_css_paths", &actix_admin.configuration.custom_css_paths);
    ctx.insert("navbar_title", &actix_admin.configuration.navbar_title);
    ctx.insert("base_path", &actix_admin.configuration.base_path);
    ctx.insert("support_path", &actix_admin.support_path.as_ref());
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

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub page: u64,
    pub entities_per_page: u64,
    pub search: String,
    pub sort_by: String,
    pub sort_order: SortOrder,
}

impl SearchParams {
    pub fn to_query_string(&self) -> String {
        format!(
            "page={0}&search={1}&sort_by={2}&sort_order={3}&entities_per_page={4}",
            self.page,
            self.search,
            self.sort_by,
            self.sort_order,
            self.entities_per_page,
        )
    }

    pub fn from_params(params: &Query<Params>, view_model: &ActixAdminViewModel) -> Self {
        SearchParams {
            page: params.page.unwrap_or(1),
            entities_per_page: params.entities_per_page.unwrap_or(DEFAULT_ENTITIES_PER_PAGE),
            search: params.search.clone().unwrap_or(String::new()),
            sort_by: params.sort_by.clone().unwrap_or(view_model.primary_key.to_string()),
            sort_order: params.sort_order.as_ref().unwrap_or(&SortOrder::Asc).clone(),
        }
    }
}

pub fn add_default_context(
    ctx: &mut Context,
    req: HttpRequest,
    view_model: &ActixAdminViewModel,
    entity_name: String,
    actix_admin: &ActixAdmin,
    notifications: Vec<ActixAdminNotification>,
    search_params: &SearchParams,
) {
    let render_partial = req.headers().contains_key("HX-Target");

    ctx.insert(
        "view_model",
        &ActixAdminViewModelSerializable::from(view_model.clone()),
    );
    ctx.insert("entity_name", &entity_name);
    ctx.insert("entity_names", &actix_admin.entity_names);
    ctx.insert("notifications", &notifications);
    ctx.insert("entities_per_page", &search_params.entities_per_page);
    ctx.insert("render_partial", &render_partial);
    ctx.insert("search", &search_params.search);
    ctx.insert("sort_by", &search_params.sort_by);
    ctx.insert("sort_order", &search_params.sort_order);
    ctx.insert("page", &search_params.page);
}