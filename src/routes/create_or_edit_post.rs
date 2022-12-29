use super::{render_unauthorized, user_can_access_page};
use crate::prelude::*;
use crate::ActixAdminError;
use crate::ActixAdminNotification;
use crate::TERA;
use actix_multipart::Multipart;
use actix_multipart::MultipartError;
use actix_session::Session;
use actix_web::http::header;
use actix_web::{error, web, Error, HttpResponse};
use std::collections::HashMap;
use tera::Context;

pub async fn create_post<T: ActixAdminAppDataTrait, E: ActixAdminViewModelTrait>(
    session: Session,
    data: web::Data<T>,
    payload: Multipart,
) -> Result<HttpResponse, Error> {
    let actix_admin = data.get_actix_admin();
    let model = ActixAdminModel::create_from_payload(
        payload,
        &format!("{}/{}", actix_admin.configuration.file_upload_directory, E::get_entity_name())
    )
    .await;
    create_or_edit_post::<T, E>(&session, &data, model, None, actix_admin).await
}

pub async fn edit_post<T: ActixAdminAppDataTrait, E: ActixAdminViewModelTrait>(
    session: Session,
    data: web::Data<T>,
    payload: Multipart,
    id: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    let actix_admin = data.get_actix_admin();
    let model = ActixAdminModel::create_from_payload(
        payload,
        &format!("{}/{}", actix_admin.configuration.file_upload_directory, E::get_entity_name()),
    )
    .await;
    create_or_edit_post::<T, E>(&session, &data, model, Some(id.into_inner()), actix_admin).await
}

pub async fn create_or_edit_post<T: ActixAdminAppDataTrait, E: ActixAdminViewModelTrait>(
    session: &Session,
    data: &web::Data<T>,
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
        return render_unauthorized(&ctx);
    }
    let db = &data.get_db();

    let mut model = model_res.unwrap();
    E::validate_entity(&mut model);

    if model.has_errors() {
        errors.push(ActixAdminError::ValidationErrors);
        render_form::<E>(actix_admin, view_model, db, entity_name, &model, errors).await
    } else {
        let res = match id {
            Some(id) => E::edit_entity(db, id, model.clone()).await,
            None => E::create_entity(db, model.clone()).await,
        };

        match res {
            Ok(_) => Ok(HttpResponse::SeeOther()
                .append_header((
                    header::LOCATION,
                    format!("/admin/{}/list", view_model.entity_name),
                ))
                .finish()),
            Err(e) => {
                errors.push(e);
                render_form::<E>(actix_admin, view_model, db, entity_name, &model, errors).await
            }
        }
    }
}

async fn render_form<E: ActixAdminViewModelTrait>(
    actix_admin: &ActixAdmin,
    view_model: &ActixAdminViewModel,
    db: &&sea_orm::DatabaseConnection,
    entity_name: String,
    model: &ActixAdminModel,
    errors: Vec<ActixAdminError>,
) -> Result<HttpResponse, Error> {
    let mut ctx = Context::new();
    ctx.insert("entity_names", &actix_admin.entity_names);
    ctx.insert(
        "view_model",
        &ActixAdminViewModelSerializable::from(view_model.clone()),
    );
    ctx.insert("select_lists", &E::get_select_lists(db).await?);
    ctx.insert("base_path", &E::get_base_path(&entity_name));
    ctx.insert("model", model);

    let notifications: Vec<ActixAdminNotification> = errors
        .into_iter()
        .map(|err| ActixAdminNotification::from(err))
        .collect();

    ctx.insert("notifications", &notifications);
    let body = TERA
        .render("create_or_edit.html", &ctx)
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
        }
    }
}
