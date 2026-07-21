use super::helpers::{add_default_context_with_session, SearchParams};
use super::{render_create_or_edit_form, AdminAction, Params, RoutePrelude};
use crate::admin_prelude;
use crate::ActixAdminError;
use crate::ActixAdminNotification;
use crate::{prelude::*, ActixAdminErrorType};
use actix_multipart::Multipart;
use actix_session::Session;
use actix_web::http::{header, StatusCode};
use actix_web::{error, web, Error, HttpRequest, HttpResponse};
use sea_orm::DatabaseConnection;
use std::collections::HashMap;
use tera::Context;

pub async fn create_post<E: ActixAdminViewModelTrait>(
    session: Session,
    req: HttpRequest,
    data: web::Data<ActixAdmin>,
    db: web::Data<DatabaseConnection>,
    payload: Multipart,
) -> Result<HttpResponse, Error> {
    let actix_admin = data.get_ref();
    let model = ActixAdminModel::create_from_payload(
        None,
        payload,
        &format!(
            "{}/{}",
            actix_admin.configuration.file_upload_directory,
            E::get_entity_name()
        ),
    )
    .await;
    create_or_edit_post::<E>(&session, req, db, model, None::<E::Id>, actix_admin).await
}

pub async fn edit_post<E: ActixAdminViewModelTrait>(
    session: Session,
    req: HttpRequest,
    data: web::Data<ActixAdmin>,
    db: web::Data<DatabaseConnection>,
    payload: Multipart,
    id: web::Path<E::Id>,
) -> Result<HttpResponse, Error> {
    let actix_admin = data.get_ref();
    let id = id.into_inner();
    let model = ActixAdminModel::create_from_payload(
        Some(id.to_string()),
        payload,
        &format!(
            "{}/{}",
            actix_admin.configuration.file_upload_directory,
            E::get_entity_name()
        ),
    )
    .await;
    create_or_edit_post::<E>(&session, req, db, model, Some(id), actix_admin).await
}

pub async fn create_or_edit_post<E: ActixAdminViewModelTrait>(
    session: &Session,
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    model_res: Result<ActixAdminModel, ActixAdminError>,
    id: Option<E::Id>,
    actix_admin: &ActixAdmin,
) -> Result<HttpResponse, Error> {
    let action = if id.is_some() {
        AdminAction::Edit
    } else {
        AdminAction::Create
    };
    // Note: multipart POSTs cannot re-verify CSRF from body here because
    // the payload was already consumed by `create_from_payload`; CSRF for
    // create/edit is asserted via the `_csrf` query param (see csrf.rs docs).
    let ctx = admin_prelude!(
        session,
        &req,
        actix_admin,
        RoutePrelude {
            action,
            verify_csrf: true,
            partial_unauth: true,
            with_auth_context: false,
        },
        E
    );
    let db = db.get_ref();

    let mut model = match model_res {
        Ok(m) => m,
        Err(e) => {
            // Fail closed on multipart/upload errors instead of panicking.
            return Err(actix_web::error::InternalError::from_response(
                e.to_string(),
                HttpResponse::build(actix_web::http::StatusCode::BAD_REQUEST)
                    .content_type("text/plain")
                    .body(e.to_string()),
            )
            .into());
        }
    };
    let _ = E::validate_entity(&mut model, db).await;

    if model.has_errors() {
        let notif = vec![ActixAdminNotification::from(ActixAdminError {
            ty: ActixAdminErrorType::ValidationErrors,
            msg: String::new(),
        })];
        return render_create_or_edit_form::<E>(
            session,
            req,
            actix_admin,
            ctx.view_model,
            db,
            ctx.entity_name,
            &model,
            ctx.tenant_ref,
            notif,
            ctx.view_model.inline_edit,
            StatusCode::OK,
        )
        .await;
    }

    let res = match id {
        Some(id) => E::edit_entity(db, id, model.clone(), ctx.tenant_ref).await,
        None => E::create_entity(db, model.clone(), ctx.tenant_ref).await,
    };

    match res {
        Ok(model) => {
            let params = Params::from_query(req.query_string());
            let search_params = SearchParams::from_params(&params, ctx.view_model);

            if ctx.view_model.inline_edit {
                let mut tctx = Context::new();
                tctx.insert("entity", &model);
                super::helpers::add_auth_context(session, actix_admin, &mut tctx);
                add_default_context_with_session(
                    &mut tctx,
                    req,
                    ctx.view_model,
                    ctx.entity_name,
                    actix_admin,
                    Vec::new(),
                    &search_params,
                    Some(session),
                );
                let body = actix_admin
                    .tera
                    .render("list/row.html", &tctx)
                    .map_err(error::ErrorInternalServerError)?;
                Ok(HttpResponse::Ok().content_type("text/html").body(body))
            } else {
                Ok(HttpResponse::SeeOther()
                    .append_header((
                        header::LOCATION,
                        format!(
                            "{0}/{1}/list?{2}",
                            actix_admin.configuration.base_path,
                            ctx.entity_name,
                            search_params.to_query_string()
                        ),
                    ))
                    .finish())
            }
        }
        Err(e) => {
            render_create_or_edit_form::<E>(
                session,
                req,
                actix_admin,
                ctx.view_model,
                db,
                ctx.entity_name,
                &model,
                ctx.tenant_ref,
                vec![ActixAdminNotification::from(e)],
                ctx.view_model.inline_edit,
                StatusCode::OK,
            )
            .await
        }
    }
}

#[doc(hidden)]
impl From<String> for ActixAdminModel {
    fn from(string: String) -> Self {
        // Parse application/x-www-form-urlencoded using the standard crate
        // rather than a bespoke hand-parser (which used to only decode `%3A`).
        let values: HashMap<String, String> =
            serde_urlencoded::from_str(&string).unwrap_or_default();

        ActixAdminModel {
            primary_key: None,
            values,
            errors: HashMap::new(),
            custom_errors: HashMap::new(),
            fk_values: HashMap::new(),
            display_name: None,
        }
    }
}
