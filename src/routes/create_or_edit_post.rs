use actix_web::http::header;
use actix_web::{web, error, Error, HttpRequest, HttpResponse};
use tera::{Context};
use actix_session::{Session};
use crate::TERA;
use actix_multipart::Multipart;

use crate::prelude::*;

pub async fn create_post<T: ActixAdminAppDataTrait, E: ActixAdminViewModelTrait>(
    session: Session,
    _req: HttpRequest,
    data: web::Data<T>,
    payload: Multipart,
) -> Result<HttpResponse, Error> {
    let db = &data.get_db();
    let mut model = ActixAdminModel::create_from_payload(payload).await.unwrap();
    model = E::create_entity(db, model).await;

    create_or_edit_post::<T, E>(session, &data, db, model).await
}

pub async fn edit_post<T: ActixAdminAppDataTrait, E: ActixAdminViewModelTrait>(
    session: Session,
    _req: HttpRequest,
    data: web::Data<T>,
    payload: Multipart,
    id: web::Path<i32>
) -> Result<HttpResponse, Error> {
    let db = &data.get_db();
    let mut model = ActixAdminModel::create_from_payload(payload).await.unwrap();
    model = E::edit_entity(db, id.into_inner(), model).await;

    create_or_edit_post::<T, E>(session, &data, db, model).await
}

async fn create_or_edit_post<T: ActixAdminAppDataTrait, E: ActixAdminViewModelTrait>(_session: Session, data: &web::Data<T>, db: &sea_orm::DatabaseConnection, model: ActixAdminModel) -> Result<HttpResponse, Error> {
    let entity_name = E::get_entity_name();
    let entity_names = &data.get_actix_admin().entity_names;
    let actix_admin = data.get_actix_admin();
    let view_model = actix_admin.view_models.get(&entity_name).unwrap();

    // TODO: verify is user is logged in and can delete entity

    if model.has_errors() {
        let mut ctx = Context::new();
        ctx.insert("entity_names", &entity_names);
        ctx.insert("view_model", &view_model);
        ctx.insert("select_lists", &E::get_select_lists(db).await);
        ctx.insert("list_link", &E::get_list_link(&entity_name));
        ctx.insert("model", &model);

        let body = TERA
            .render("create_or_edit.html", &ctx)
            .map_err(|err| error::ErrorInternalServerError(err))?;
        Ok(HttpResponse::Ok().content_type("text/html").body(body))
    }
    else {
        Ok(HttpResponse::SeeOther()
            .append_header((
                header::LOCATION,
                format!("/admin/{}/list", view_model.entity_name),
            ))
            .finish())
        }
}