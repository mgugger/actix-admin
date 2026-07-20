use actix_web::{error, web, Error, HttpRequest, HttpResponse};
use sea_orm::DatabaseConnection;
use tera::Context;
use actix_session::Session;
use crate::ActixAdminError;
use crate::ActixAdminNotification;
use crate::prelude::*;

use super::helpers::add_default_context_with_session;
use super::helpers::SearchParams;
use super::Params;
use super::{add_auth_context, render_template, render_unauthorized, user_can_perform, view_model_or_500, AdminAction};

pub async fn create_get<E: ActixAdminViewModelTrait>(
    session: Session,
    req: HttpRequest,
    data: web::Data<ActixAdmin>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, Error> {
    let db = db.get_ref();
    let model = ActixAdminModel::create_empty();

    create_or_edit_get::<E>(&session, req, &data, db, Ok(model), false).await
}

pub async fn edit_get<E: ActixAdminViewModelTrait>(
    session: Session,
    req: HttpRequest,
    data: web::Data<ActixAdmin>,
    db: web::Data<DatabaseConnection>,
    id: web::Path<E::Id>,
) -> Result<HttpResponse, Error> {
    let db = db.get_ref();
    let actix_admin = data.get_ref();
    let tenant_ref = actix_admin
        .configuration
        .user_tenant_ref
        .and_then(|f| f(&session));

    let model = E::get_entity(db, id.into_inner(), tenant_ref).await;
    if let Err(ref e) = model {
        if e.ty == crate::ActixAdminErrorType::EntityDoesNotExistError {
            let body = actix_admin
                .tera
                .render("not_found.html", &tera::Context::new())
                .unwrap_or_else(|_| String::from("Not Found"));
            return Ok(HttpResponse::NotFound().content_type("text/html").body(body));
        }
    }
    let entity_name = E::get_entity_name();
    let view_model = view_model_or_500(actix_admin, &entity_name)?;

    create_or_edit_get::<E>(&session, req, &data, db, model, view_model.inline_edit).await
}

async fn create_or_edit_get<E: ActixAdminViewModelTrait>(
    session: &Session,
    req: HttpRequest,
    data: &web::Data<ActixAdmin>,
    db: &sea_orm::DatabaseConnection,
    model_result: Result<ActixAdminModel, ActixAdminError>,
    is_inline: bool,
) -> Result<HttpResponse, Error> {
    let actix_admin = data.get_ref();
    let mut ctx = Context::new();
    add_auth_context(session, actix_admin, &mut ctx);
    let entity_name = E::get_entity_name();
    let mut errors: Vec<crate::ActixAdminError> = Vec::new();

    let view_model = view_model_or_500(actix_admin, &entity_name)?;

    let is_edit = model_result
        .as_ref()
        .ok()
        .and_then(|m| m.primary_key.clone())
        .is_some();
    let required_action = if is_edit { AdminAction::Edit } else { AdminAction::Create };

    if !user_can_perform(session, actix_admin, view_model, required_action) {
        return render_unauthorized(&ctx, actix_admin);
    }

    let model = match model_result {
        Ok(res) => res,
        Err(e) => {
            errors.push(e);
            ActixAdminModel::create_empty()
        }
    };

    let mut http_response_code = match errors.first() {
        None => HttpResponse::Ok(),
        Some(e) if e.ty == crate::ActixAdminErrorType::EntityDoesNotExistError => {
            HttpResponse::NotFound()
        }
        Some(_) => HttpResponse::InternalServerError(),
    };
    let notifications: Vec<ActixAdminNotification> = errors
        .into_iter()
        .map(ActixAdminNotification::from)
        .collect();

    let params = Params::from_query(req.query_string());
    let search_params = SearchParams::from_params(&params, view_model);
    add_default_context_with_session(&mut ctx, req, view_model, entity_name, actix_admin, notifications, &search_params, Some(session));

    let tenant_ref = actix_admin
        .configuration
        .user_tenant_ref
        .and_then(|f| f(session));

    ctx.insert("select_lists", &E::get_select_lists(db, tenant_ref).await?);
    ctx.insert("model", &model);

    let template_path = if is_inline {
        "create_or_edit/inline.html"
    } else {
        "create_or_edit.html"
    };
    let body = render_template(&actix_admin.tera, template_path, &ctx)
        .map_err(error::ErrorInternalServerError)?;
    Ok(http_response_code.content_type("text/html").body(body))
}