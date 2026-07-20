use actix_session::Session;
use actix_web::HttpRequest;
use actix_web::{error, web, Error, HttpResponse};
use sea_orm::DatabaseConnection;
use tera::Context;
use super::helpers::{add_default_context, SearchParams};
use crate::prelude::*;
use crate::ActixAdminNotification;

use super::{add_auth_context, render_template, render_unauthorized, user_can_access_page, view_model_or_500};
use super::Params;

pub async fn show<E: ActixAdminViewModelTrait>(
    session: Session,
    req: HttpRequest,
    data: web::Data<ActixAdmin>,
    id: web::Path<E::Id>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, Error> {
    let actix_admin = &data.into_inner();

    let mut ctx = Context::new();
    let entity_name = E::get_entity_name();
    let view_model: &ActixAdminViewModel = view_model_or_500(actix_admin, &entity_name)?;
    if !user_can_access_page(&session, actix_admin, view_model) {
        return render_unauthorized(&ctx, actix_admin);
    }

    let tenant_ref = actix_admin
        .configuration
        .user_tenant_ref
        .and_then(|f| f(&session));

    let mut errors: Vec<crate::ActixAdminError> = Vec::new();
    let model = match E::get_entity(&db, id.into_inner(), tenant_ref).await {
        Ok(res) => res,
        Err(e) if e.ty == crate::ActixAdminErrorType::EntityDoesNotExistError => {
            // Short-circuit: don't try to render show.html with an empty model.
            let body = actix_admin
                .tera
                .render("not_found.html", &tera::Context::new())
                .unwrap_or_else(|_| String::from("Not Found"));
            return Ok(HttpResponse::NotFound().content_type("text/html").body(body));
        }
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

    add_auth_context(&session, actix_admin, &mut ctx);

    add_default_context(
        &mut ctx,
        req,
        view_model,
        entity_name,
        actix_admin,
        notifications,
        &search_params,
    );
    ctx.insert("model", &model);

    let body = render_template(&actix_admin.tera, "show.html", &ctx)
        .map_err(error::ErrorInternalServerError)?;
    Ok(http_response_code.content_type("text/html").body(body))
}