use super::helpers::{add_default_context, SearchParams};
use super::{add_auth_context, render_unauthorized, user_can_access_page};
use super::Params;
use crate::ActixAdminError;
use crate::ActixAdminNotification;
use crate::{prelude::*, ActixAdminErrorType};
use actix_multipart::Multipart;
use actix_multipart::MultipartError;
use actix_session::Session;
use actix_web::http::header;
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
    create_or_edit_post::<E>(&session, req, db, model, None, actix_admin).await
}

pub async fn edit_post<E: ActixAdminViewModelTrait>(
    session: Session,
    req: HttpRequest,
    data: web::Data<ActixAdmin>,
    db: web::Data<DatabaseConnection>,
    payload: Multipart,
    id: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    let actix_admin = &data.get_ref();
    let id = id.into_inner();
    let model = ActixAdminModel::create_from_payload(
        Some(id), payload,
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
    model_res: Result<ActixAdminModel, MultipartError>,
    id: Option<i32>,
    actix_admin: &ActixAdmin,
) -> Result<HttpResponse, Error> {
    let entity_name = E::get_entity_name();

    let view_model = actix_admin.view_models.get(&entity_name).unwrap();
    let mut errors: Vec<ActixAdminError> = Vec::new();

    if !user_can_access_page(&session, actix_admin, view_model) {
        let mut ctx = Context::new();
        ctx.insert("render_partial", &true);
        return render_unauthorized(&ctx, &actix_admin);
    }
    let db = db.get_ref();

    let mut model = model_res.unwrap();
    let _ = E::validate_entity(&mut model, db).await;

    if model.has_errors() {
        let error = ActixAdminError {
            ty: ActixAdminErrorType::ValidationErrors,
            msg: "".to_owned(),
        };
        errors.push(error);
        render_form::<E>(
            session,
            req,
            actix_admin,
            view_model,
            &db,
            entity_name,
            &model,
            errors,
        )
        .await
    } else {
        let tenant_ref = actix_admin
            .configuration
            .user_tenant_ref
            .map_or(None, |f| f(&session));

        let res = match id {
            Some(id) => E::edit_entity(db, id, model.clone(), tenant_ref).await,
            None => E::create_entity(db, model.clone(), tenant_ref).await,
        };

        match res {
            Ok(model) => {
                let params = web::Query::<Params>::from_query(req.query_string()).unwrap();

                let entity_name = E::get_entity_name();

                let search_params = SearchParams::from_params(&params, view_model);

                if view_model.inline_edit {
                    let mut ctx = Context::new();

                    ctx.insert("entity", &model);

                    add_auth_context(&session, actix_admin, &mut ctx);
                    add_default_context(&mut ctx, req, view_model, entity_name, actix_admin, Vec::new(), &search_params);

                    let body = actix_admin
                        .tera
                        .render("list/row.html", &ctx)
                        .map_err(|err| { error::ErrorInternalServerError(err) })?;
                    Ok(HttpResponse::Ok().content_type("text/html").body(body))
                } else {
                    Ok(HttpResponse::SeeOther()
                .append_header((
                    header::LOCATION,
                    format!("{0}/{1}/list?{2}", actix_admin.configuration.base_path, entity_name, search_params.to_query_string()),
                ))
                .finish())
                }
            }
            Err(e) => {
                errors.push(e);
                render_form::<E>(
                    session,
                    req,
                    actix_admin,
                    view_model,
                    &db,
                    entity_name,
                    &model,
                    errors,
                )
                .await
            }
        }
    }
}

async fn render_form<E: ActixAdminViewModelTrait>(
    session: &Session,
    req: HttpRequest,
    actix_admin: &ActixAdmin,
    view_model: &ActixAdminViewModel,
    db: &&sea_orm::DatabaseConnection,
    entity_name: String,
    model: &ActixAdminModel,
    errors: Vec<ActixAdminError>,
) -> Result<HttpResponse, Error> {
    let mut ctx = Context::new();

    let params = web::Query::<Params>::from_query(req.query_string()).unwrap();

    let tenant_ref = actix_admin
        .configuration
        .user_tenant_ref
        .map_or(None, |f| f(&session));

    ctx.insert("select_lists", &E::get_select_lists(db, tenant_ref).await?);
    ctx.insert("model", model);

    let notifications: Vec<ActixAdminNotification> = errors
    .into_iter()
    .map(|err| ActixAdminNotification::from(err))
    .collect();

    add_auth_context(&session, actix_admin, &mut ctx);

    let search_params = SearchParams::from_params(&params, view_model);
    add_default_context(&mut ctx, req, view_model, entity_name, actix_admin, notifications, &search_params);

    let template_path = match (view_model.inline_edit, model.primary_key.is_some()) {
        (true, true) => "create_or_edit/inline.html",
        (_, _) => "create_or_edit.html",
    };
    let body = actix_admin
        .tera
        .render(template_path, &ctx)
        .map_err(|err| error::ErrorInternalServerError(err))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

#[doc(hidden)]
impl From<String> for ActixAdminModel {
    fn from(string: String) -> Self {
        let mut hashmap = HashMap::new();
        let key_values: Vec<&str> = string.split('&').collect();
        for key_value in key_values {
            if !key_value.is_empty() {
                let mut iter = key_value.splitn(2, '=');
                hashmap.insert(
                    iter.next().unwrap().to_string().replace("%3A", ":"),
                    iter.next().unwrap().to_string().replace("%3A", ":"),
                );
            }
        }

        ActixAdminModel {
            primary_key: None,
            values: hashmap,
            errors: HashMap::new(),
            custom_errors: HashMap::new(),
            fk_values: HashMap::new(),
            display_name: None
        }
    }
}