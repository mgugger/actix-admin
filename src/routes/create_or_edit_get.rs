use crate::admin_prelude;
use crate::prelude::*;
use actix_session::Session;
use actix_web::http::StatusCode;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use sea_orm::DatabaseConnection;

use super::{render_create_or_edit_form, RoutePrelude};

pub async fn create_get<E: ActixAdminViewModelTrait>(
    session: Session,
    req: HttpRequest,
    data: web::Data<ActixAdmin>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, Error> {
    let actix_admin = data.get_ref();
    let ctx = admin_prelude!(&session, &req, actix_admin, RoutePrelude::create(), E);

    render_create_or_edit_form::<E>(
        &session,
        req,
        actix_admin,
        ctx.view_model,
        db.get_ref(),
        ctx.entity_name,
        &ActixAdminModel::create_empty(),
        ctx.tenant_ref,
        Vec::new(),
        false,
        StatusCode::OK,
    )
    .await
}

pub async fn edit_get<E: ActixAdminViewModelTrait>(
    session: Session,
    req: HttpRequest,
    data: web::Data<ActixAdmin>,
    db: web::Data<DatabaseConnection>,
    id: web::Path<E::Id>,
) -> Result<HttpResponse, Error> {
    let actix_admin = data.get_ref();
    let ctx = admin_prelude!(&session, &req, actix_admin, RoutePrelude::edit(), E);

    let db = db.get_ref();
    let model_result = E::get_entity(db, id.into_inner(), ctx.tenant_ref).await;

    let (model, notifications, status) = match model_result {
        Ok(m) => (m, Vec::new(), StatusCode::OK),
        Err(e) if e.ty == crate::ActixAdminErrorType::EntityDoesNotExistError => {
            let body = actix_admin
                .tera
                .render("not_found.html", &tera::Context::new())
                .unwrap_or_else(|_| String::from("Not Found"));
            return Ok(HttpResponse::NotFound()
                .content_type("text/html")
                .body(body));
        }
        Err(e) => (
            ActixAdminModel::create_empty(),
            vec![crate::ActixAdminNotification::from(e)],
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
    };

    render_create_or_edit_form::<E>(
        &session,
        req,
        actix_admin,
        ctx.view_model,
        db,
        ctx.entity_name,
        &model,
        ctx.tenant_ref,
        notifications,
        ctx.view_model.inline_edit,
        status,
    )
    .await
}
