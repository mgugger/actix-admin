use actix_web::http::header;
use actix_web::{web, error, Error, HttpResponse};
use tera::{Context};
use actix_session::{Session};
use crate::TERA;
use actix_multipart::Multipart;
use super::{ user_can_access_page, render_unauthorized};
use crate::prelude::*;

pub async fn create_post<T: ActixAdminAppDataTrait, E: ActixAdminViewModelTrait>(
    session: Session,
    data: web::Data<T>,
    payload: Multipart,
) -> Result<HttpResponse, Error> {
    create_or_edit_post::<T, E>(&session, &data, payload, None).await
}

pub async fn edit_post<T: ActixAdminAppDataTrait, E: ActixAdminViewModelTrait>(
    session: Session,
    data: web::Data<T>,
    payload: Multipart,
    id: web::Path<i32>
) -> Result<HttpResponse, Error> {
    create_or_edit_post::<T, E>(&session, &data, payload, Some(id.into_inner())).await
}

async fn create_or_edit_post<T: ActixAdminAppDataTrait, E: ActixAdminViewModelTrait>(session: &Session, data: &web::Data<T>, payload: Multipart, id: Option<i32>) -> Result<HttpResponse, Error> {
    let actix_admin = data.get_actix_admin();
    let entity_name = E::get_entity_name();

    let view_model = actix_admin.view_models.get(&entity_name).unwrap();

    if !user_can_access_page(&session, actix_admin, view_model) {
        let mut ctx = Context::new();
        ctx.insert("render_partial", &true);
        return render_unauthorized(&ctx);
    }
    
    let db = &data.get_db();
    let model = ActixAdminModel::create_from_payload(payload).await.unwrap();

    if model.has_errors() {
        let mut ctx = Context::new();
        ctx.insert("entity_names", &actix_admin.entity_names);
        ctx.insert("view_model", &ActixAdminViewModelSerializable::from(view_model.clone()));
        ctx.insert("select_lists", &E::get_select_lists(db).await);
        ctx.insert("list_link", &E::get_list_link(&entity_name));
        ctx.insert("model", &model);

        let body = TERA
            .render("create_or_edit.html", &ctx)
            .map_err(|err| error::ErrorInternalServerError(err))?;
        Ok(HttpResponse::Ok().content_type("text/html").body(body))
    }
    else {
        match id {
            Some(id) => E::edit_entity(db, id, model).await,
            None => E::create_entity(db, model).await
        };

        Ok(HttpResponse::SeeOther()
            .append_header((
                header::LOCATION,
                format!("/admin/{}/list", view_model.entity_name),
            ))
            .finish())
        }
}