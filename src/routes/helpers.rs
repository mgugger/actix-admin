use actix_session::Session;
use serde_derive::Deserialize;
use tera::Context;

use crate::{prelude::*, ActixAdminNotification};
use actix_web::{error, Error, HttpRequest, HttpResponse};

use super::{Params, DEFAULT_ENTITIES_PER_PAGE};

pub fn add_auth_context(session: &Session, actix_admin: &ActixAdmin, ctx: &mut Context) {
    let cfg = &actix_admin.configuration;
    ctx.insert("enable_auth", &cfg.enable_auth);
    ctx.insert("custom_css_paths", &cfg.custom_css_paths);
    ctx.insert("custom_js_paths", &cfg.custom_js_paths);
    ctx.insert("navbar_title", &cfg.navbar_title);
    ctx.insert("base_path", &cfg.base_path);
    ctx.insert("support_path", &actix_admin.support_path.as_ref());
    if cfg.enable_auth {
        let func = cfg.user_is_logged_in.unwrap();
        ctx.insert("user_is_logged_in", &func(session));
        ctx.insert("login_link", cfg.login_link.as_deref().unwrap_or(""));
        ctx.insert("logout_link", cfg.logout_link.as_deref().unwrap_or(""));
    }
}

pub fn user_can_access_page(session: &Session, actix_admin: &ActixAdmin, view_model: &ActixAdminViewModel) -> bool {
    let cfg = &actix_admin.configuration;
    match (cfg.enable_auth, cfg.user_is_logged_in, view_model.user_can_access) {
        (true, Some(auth), Some(vm_access)) => auth(session) && vm_access(session),
        (true, Some(auth), None) => auth(session),
        _ => !cfg.enable_auth,
    }
}

pub fn render_unauthorized(ctx: &Context, actix_admin: &ActixAdmin) -> Result<HttpResponse, Error> {
    let body = actix_admin.tera
            .render("unauthorized.html", ctx)
            .map_err(error::ErrorInternalServerError)?;
    Ok(HttpResponse::Unauthorized().content_type("text/html").body(body))
}

/// Render `template_name` with `ctx`, falling back to rendering only the
/// `content` block when the context has `render_partial == true`.
///
/// Tera 2 no longer allows `{% block %}` inside `{% if %}`, so the partial
/// vs. full page decision is made here in Rust instead of inside `base.html`.
pub fn render_template(
    tera: &tera::Tera,
    template_name: &str,
    ctx: &Context,
) -> Result<String, tera::Error> {
    let render_partial = ctx
        .get("render_partial")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if render_partial {
        tera.render_block(template_name, "content", ctx)
    } else {
        tera.render(template_name, ctx)
    }
}

/// Look up the view model for an entity name. Returns 500 rather than panicking
/// if it is missing (should be impossible in normal operation).
pub fn view_model_or_500<'a>(
    actix_admin: &'a ActixAdmin,
    entity_name: &str,
) -> Result<&'a ActixAdminViewModel, Error> {
    actix_admin
        .view_models
        .get(entity_name)
        .ok_or_else(|| error::ErrorInternalServerError(
            format!("View model for entity '{entity_name}' is not registered")
        ))
}

/// Validate that `sort_by` refers to a real, non-hidden field on the view model.
/// Returns Ok(sort_by) or a 400 error.
pub fn validate_sort_by(
    view_model: &ActixAdminViewModel,
    sort_by: &str,
) -> Result<(), Error> {
    if sort_by == view_model.primary_key {
        return Ok(());
    }
    if view_model.fields.iter().any(|f| f.field_name == sort_by) {
        Ok(())
    } else {
        Err(error::ErrorBadRequest(format!(
            "Unknown sort column: {sort_by}"
        )))
    }
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
        use urlencoding::encode;
        format!(
            "page={0}&search={1}&sort_by={2}&sort_order={3}&entities_per_page={4}",
            self.page,
            encode(&self.search),
            encode(&self.sort_by),
            self.sort_order,
            self.entities_per_page,
        )
    }

    pub fn from_params(params: &Params, view_model: &ActixAdminViewModel) -> Self {
        SearchParams {
            page: params.page.unwrap_or(1),
            entities_per_page: params.entities_per_page.unwrap_or(DEFAULT_ENTITIES_PER_PAGE),
            search: params.search.clone().unwrap_or_default(),
            sort_by: params
                .sort_by
                .clone()
                .unwrap_or_else(|| view_model.primary_key.clone()),
            sort_order: params.sort_order.clone().unwrap_or(SortOrder::Asc),
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