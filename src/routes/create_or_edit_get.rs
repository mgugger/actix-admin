use actix_web::{error, web, Error, HttpRequest, HttpResponse};
use tera::{Context};
use actix_session::{Session};
use crate::ActixAdminError;
use crate::ActixAdminNotification;
use crate::prelude::*;

use crate::TERA;
use super::{ add_auth_context, user_can_access_page, render_unauthorized};

pub async fn create_get<T: ActixAdminAppDataTrait, E: ActixAdminViewModelTrait>(
    session: Session,
    _req: HttpRequest,
    data: web::Data<T>,
    _body: web::Payload,
    _text: String,
) -> Result<HttpResponse, Error> {
    let db = &data.get_db();
    let model = ActixAdminModel::create_empty();
    
    create_or_edit_get::<T, E>(&session, &data, db, Ok(model)).await
}

pub async fn edit_get<T: ActixAdminAppDataTrait, E: ActixAdminViewModelTrait>(
    session: Session,
    _req: HttpRequest,
    data: web::Data<T>,
    _text: String,
    id: web::Path<i32>
) -> Result<HttpResponse, Error> {
    let db = &data.get_db();
    let model = E::get_entity(db, id.into_inner()).await;

    create_or_edit_get::<T, E>(&session, &data, db, model).await
}

async fn create_or_edit_get<T: ActixAdminAppDataTrait, E: ActixAdminViewModelTrait>(session: &Session, data: &web::Data<T>, db: &sea_orm::DatabaseConnection, model_result: Result<ActixAdminModel, ActixAdminError>) -> Result<HttpResponse, Error>{
    let actix_admin = &data.get_actix_admin();
    let mut ctx = Context::new();
    add_auth_context(&session, actix_admin, &mut ctx);
    let entity_names = &actix_admin.entity_names;
    ctx.insert("entity_names", entity_names);
    let entity_name = E::get_entity_name();
    let mut errors: Vec<crate::ActixAdminError> = Vec::new();

    let view_model = actix_admin.view_models.get(&entity_name).unwrap();

    if !user_can_access_page(&session, actix_admin, view_model) {
        return render_unauthorized(&ctx);
    }

    let model;
    match model_result {
        Ok(res) => {
            model = res;
        },
        Err(e) => {
            errors.push(e);
            model = ActixAdminModel::create_empty();
        }
    }

    let mut http_response_code = match errors.is_empty() {
        true => HttpResponse::Ok(),
        false => HttpResponse::InternalServerError(),
    };    
    let notifications: Vec<ActixAdminNotification> = errors.into_iter()
        .map(|err| ActixAdminNotification::from(err))
        .collect();

    ctx.insert("view_model", &ActixAdminViewModelSerializable::from(view_model.clone()));
    ctx.insert("select_lists", &E::get_select_lists(db).await?);
    ctx.insert("list_link", &E::get_list_link(&entity_name));
    ctx.insert("model", &model);
    ctx.insert("notifications", &notifications);
    
    let body = TERA
        .render("create_or_edit.html", &ctx)
        .map_err(|err| error::ErrorInternalServerError(err))?;
    Ok(http_response_code.content_type("text/html").body(body))
}