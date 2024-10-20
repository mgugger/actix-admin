use actix_session::Session;
use actix_web::HttpRequest;
use actix_web::{error, web, Error, HttpResponse};
use sea_orm::DatabaseConnection;
use tera::Context;
use super::helpers::{add_default_context, SearchParams};
use crate::prelude::*;
use crate::ActixAdminNotification;

use super::{add_auth_context, render_unauthorized, user_can_access_page};
use super::{Params, DEFAULT_ENTITIES_PER_PAGE};

pub async fn show<E: ActixAdminViewModelTrait>(
    session: Session,
    req: HttpRequest,
    data: web::Data<ActixAdmin>,
    id: web::Path<i32>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, Error> {
    let actix_admin = &data.into_inner();

    let mut ctx = Context::new();
    let entity_name = E::get_entity_name();
    let view_model: &ActixAdminViewModel = actix_admin.view_models.get(&entity_name).unwrap();
    if !user_can_access_page(&session, actix_admin, view_model) {
        return render_unauthorized(&ctx, &actix_admin);
    }

    let tenant_ref = actix_admin
        .configuration
        .user_tenant_ref
        .map_or(None, |f| f(&session));

    let mut errors: Vec<crate::ActixAdminError> = Vec::new();
    let model = match E::get_entity(&db, id.into_inner(), tenant_ref).await {
        Ok(res) => res,
        Err(e) => {
            errors.push(e);
            ActixAdminModel::create_empty()
        }
    };

    let mut http_response_code = match errors.is_empty() {
        false => HttpResponse::InternalServerError(),
        true => HttpResponse::Ok(),
    };
    let notifications: Vec<ActixAdminNotification> = errors
        .into_iter()
        .map(|err| ActixAdminNotification::from(err))
        .collect();

    let params = web::Query::<Params>::from_query(req.query_string()).unwrap();

    let search_params = SearchParams {
        page: params.page.unwrap_or(1),
        entities_per_page: params
            .entities_per_page
            .unwrap_or(DEFAULT_ENTITIES_PER_PAGE),
        search: params.search.clone().unwrap_or(String::new()),
        sort_by: params
            .sort_by
            .clone()
            .unwrap_or(view_model.primary_key.to_string()),
        sort_order: params
            .sort_order
            .as_ref()
            .unwrap_or(&SortOrder::Asc)
            .clone(),
    };

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

    let body = actix_admin
        .tera
        .render("show.html", &ctx)
        .map_err(|err| error::ErrorInternalServerError(err))?;
    Ok(http_response_code.content_type("text/html").body(body))
}