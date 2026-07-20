use actix_session::Session;
use serde_derive::Deserialize;
use tera::Context;

use crate::{prelude::*, ActixAdminNotification};
use actix_web::{error, Error, HttpRequest, HttpResponse};

use super::{Params, DEFAULT_ENTITIES_PER_PAGE};

/// The set of gated actions on an entity view. Used by [`user_can_perform`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdminAction {
    /// Access the list, show and (read-only) detail routes.
    View,
    Create,
    Edit,
    Delete,
    /// Export the list as CSV / other formats.
    Export,
    /// Trigger a custom bulk action.
    BulkAction,
}

pub fn add_auth_context(session: &Session, actix_admin: &ActixAdmin, ctx: &mut Context) {
    let cfg = &actix_admin.configuration;
    ctx.insert("enable_auth", &cfg.enable_auth);
    ctx.insert("custom_css_paths", &cfg.custom_css_paths);
    ctx.insert("custom_js_paths", &cfg.custom_js_paths);
    ctx.insert("navbar_title", &cfg.navbar_title);
    ctx.insert("base_path", &cfg.base_path);
    ctx.insert("support_path", &actix_admin.support_path.as_ref());
    ctx.insert("enable_csrf", &cfg.enable_csrf);
    // Always insert a (possibly empty) csrf_token so templates can reference
    // it unconditionally without checking `enable_csrf`.
    let mut token_value = String::new();
    if cfg.enable_csrf {
        token_value = csrf_token_for(session).unwrap_or_default();
    }
    ctx.insert("csrf_token", &token_value);
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

/// True iff the user can perform `action` on `view_model`. Always requires
/// top-level page access via [`user_can_access_page`] first.
pub fn user_can_perform(
    session: &Session,
    actix_admin: &ActixAdmin,
    view_model: &ActixAdminViewModel,
    action: AdminAction,
) -> bool {
    if !user_can_access_page(session, actix_admin, view_model) {
        return false;
    }
    let hook = match action {
        AdminAction::View => view_model.user_can_view_details,
        AdminAction::Create => view_model.user_can_create,
        AdminAction::Edit => view_model.user_can_edit,
        AdminAction::Delete => view_model.user_can_delete,
        AdminAction::Export => view_model.user_can_export,
        // Bulk actions inherit the top-level page permission by default;
        // fine-grained gating happens inside individual action handlers.
        AdminAction::BulkAction => return true,
    };
    match hook {
        Some(f) => f(session),
        None => true,
    }
}

/// Same as [`user_can_perform`] but returns a ready-made 403 response when
/// the user is denied. Convenience for route handlers.
pub fn forbid_if_denied(
    session: &Session,
    actix_admin: &ActixAdmin,
    view_model: &ActixAdminViewModel,
    action: AdminAction,
) -> Option<HttpResponse> {
    if user_can_perform(session, actix_admin, view_model, action) {
        None
    } else {
        Some(HttpResponse::Forbidden().finish())
    }
}

pub fn render_unauthorized(ctx: &Context, actix_admin: &ActixAdmin) -> Result<HttpResponse, Error> {
    // Fall back to a short plain-text body if the template render fails
    // (e.g. the caller only supplied a partial context). Returning 500
    // here would leak an internal error to a user who simply lacks a
    // permission.
    let body = actix_admin
        .tera
        .render("unauthorized.html", ctx)
        .unwrap_or_else(|_| String::from("Forbidden"));
    Ok(HttpResponse::Forbidden().content_type("text/html").body(body))
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

#[allow(dead_code)]
pub fn add_default_context(
    ctx: &mut Context,
    req: HttpRequest,
    view_model: &ActixAdminViewModel,
    entity_name: String,
    actix_admin: &ActixAdmin,
    notifications: Vec<ActixAdminNotification>,
    search_params: &SearchParams,
) {
    add_default_context_with_session(
        ctx, req, view_model, entity_name, actix_admin, notifications, search_params, None,
    )
}

/// Variant that also resolves per-view permission hooks against `session`
/// and pushes them into the template context as `view_model.can_*` booleans.
pub fn add_default_context_with_session(
    ctx: &mut Context,
    req: HttpRequest,
    view_model: &ActixAdminViewModel,
    entity_name: String,
    actix_admin: &ActixAdmin,
    notifications: Vec<ActixAdminNotification>,
    search_params: &SearchParams,
    session: Option<&Session>,
) {
    let render_partial = req.headers().contains_key("HX-Target");

    let mut serializable = ActixAdminViewModelSerializable::from(view_model.clone());
    if let Some(session) = session {
        serializable.can_create = user_can_perform(session, actix_admin, view_model, AdminAction::Create);
        serializable.can_edit = user_can_perform(session, actix_admin, view_model, AdminAction::Edit);
        serializable.can_delete = user_can_perform(session, actix_admin, view_model, AdminAction::Delete);
        serializable.can_view_details =
            user_can_perform(session, actix_admin, view_model, AdminAction::View);
        serializable.can_export =
            user_can_perform(session, actix_admin, view_model, AdminAction::Export);
    }

    ctx.insert("view_model", &serializable);
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