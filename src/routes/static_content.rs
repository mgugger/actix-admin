use actix_web::{web, error, Error, HttpResponse, HttpRequest};
use actix_session::{Session};
use tera::{Context};
use crate::prelude::*;

use super::{ user_can_access_page, render_unauthorized};

pub async fn download<T: ActixAdminAppDataTrait, E: ActixAdminViewModelTrait>(req: HttpRequest, session: Session, data: web::Data<T>, params: web::Path<(i32, String)>) -> Result<HttpResponse, Error> {
    let actix_admin = data.get_actix_admin();
    let db = &data.get_db();

    let ctx = Context::new();
    let entity_name = E::get_entity_name();
    let view_model: &ActixAdminViewModel = actix_admin.view_models.get(&entity_name).unwrap();
    if !user_can_access_page(&session, actix_admin, view_model) {
        return render_unauthorized(&ctx);
    }
    
    let (id, column_name) = params.into_inner();
    let mut errors: Vec<crate::ActixAdminError> = Vec::new();
    let result = E::get_entity(db, id).await;
    let model;
    match result {
        Ok(res) => {
            model = res;
        },
        Err(e) => {
            errors.push(e);
            model = ActixAdminModel::create_empty();
        }
    }

    let file_name = model.get_value::<String>(&column_name, true, true).unwrap_or_default();
    let file_path = format!("{}/{}/{}", actix_admin.configuration.file_upload_directory, E::get_entity_name(), file_name.unwrap_or_default());
    let file = actix_files::NamedFile::open_async(file_path).await;

    match file {
        Ok(file) => Ok(file.into_response(&req)),
        Err(_e) => Ok(HttpResponse::NotFound().content_type("text/html").body(""))
    }
    
}

pub async fn delete_static_content<T: ActixAdminAppDataTrait, E: ActixAdminViewModelTrait>(session: Session, data: web::Data<T>, params: web::Path<(i32, String)>) -> Result<HttpResponse, Error> {
    let actix_admin = data.get_actix_admin();
    let db = &data.get_db();

    let mut ctx = Context::new();
    let entity_name = E::get_entity_name();
    let view_model: &ActixAdminViewModel = actix_admin.view_models.get(&entity_name).unwrap();
    if !user_can_access_page(&session, actix_admin, view_model) {
        return render_unauthorized(&ctx);
    }
    
    let (id, column_name) = params.into_inner();
    let mut errors: Vec<crate::ActixAdminError> = Vec::new();
    let result = E::get_entity(db, id).await;
    let mut model;
    match result {
        Ok(res) => {
            model = res;
        },
        Err(e) => {
            errors.push(e);
            model = ActixAdminModel::create_empty();
        }
    }

    let file_name = model.get_value::<String>(&column_name, true, true).unwrap_or_default();
    let file_path = format!("{}/{}/{}", actix_admin.configuration.file_upload_directory, E::get_entity_name(), file_name.unwrap_or_default());
    std::fs::remove_file(file_path).unwrap();
    model.values.remove(&column_name);
    let _edit_res = E::edit_entity(db, id, model.clone()).await;

    let view_model_field = &view_model.fields.iter().find(|field| field.field_name == column_name).unwrap();
    ctx.insert("model_field", view_model_field);
    ctx.insert("base_path", &E::get_base_path(&entity_name));
    ctx.insert("model", &model);

    let body = TERA
        .render("form_elements/input.html", &ctx)
        .map_err(|err| error::ErrorInternalServerError(err))? ;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
    
}