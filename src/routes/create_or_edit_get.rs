use actix_web::{error, web, Error, HttpRequest, HttpResponse};
use sea_orm::DatabaseConnection;
use tera::Context;
use actix_session::Session;
use crate::ActixAdminError;
use crate::ActixAdminNotification;
use crate::prelude::*;

use super::helpers::add_default_context;
use super::helpers::SearchParams;
use super::Params;
use super::{add_auth_context, render_unauthorized, user_can_access_page, view_model_or_500};

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
    id: web::Path<i32>,
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

    if !user_can_access_page(session, actix_admin, view_model) {
        return render_unauthorized(&ctx, actix_admin);
    }

    let model = match model_result {
        Ok(res) => res,
        Err(e) => {
            errors.push(e);
            ActixAdminModel::create_empty()
        }
    };

    let http_response_code = match errors.first() {
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
    add_default_context(&mut ctx, req, view_model, entity_name, actix_admin, notifications, &search_params);

    let tenant_ref = actix_admin
        .configuration
        .user_tenant_ref
        .and_then(|f| f(session));

    ctx.insert("view_model", &ActixAdminViewModelSerializable::from(view_model.clone()));
    ctx.insert("select_lists", &E::get_select_lists(db, tenant_ref).await?);
    ctx.insert("model", &model);

    let template_path = if is_inline {
        "create_or_edit/inline.html"
    } else {
        "create_or_edit.html"
    };
    let mut resp = http_response_code;
    let body = actix_admin.tera
        .render(template_path, &ctx)
        .map_err(error::ErrorInternalServerError)?;
    Ok(resp.content_type("text/html").body(body))
}