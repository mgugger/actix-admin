use crate::prelude::*;
use actix_session::Session;
use actix_web::{error, web, Error, HttpRequest, HttpResponse};
use sea_orm::DatabaseConnection;
use tera::Context;

use super::{render_unauthorized, user_can_access_page};

pub async fn download<E: ActixAdminViewModelTrait>(
    req: HttpRequest,
    session: Session,
    data: web::Data<ActixAdmin>,
    db: web::Data<DatabaseConnection>,
    params: web::Path<(i32, String)>,
) -> Result<HttpResponse, Error> {
    let actix_admin = &data.into_inner();
    let db = &db.into_inner();

    let ctx = Context::new();
    let entity_name = E::get_entity_name();
    let view_model: &ActixAdminViewModel = actix_admin.view_models.get(&entity_name).unwrap();
    if !user_can_access_page(&session, actix_admin, view_model) {
        return render_unauthorized(&ctx, &actix_admin);
    }

    let tenant_ref = actix_admin
        .configuration
        .user_tenant_ref
        .map_or(None, |f| f(&session));

    let (id, column_name) = params.into_inner();
    let mut errors: Vec<crate::ActixAdminError> = Vec::new();
    let result = E::get_entity(db, id, tenant_ref).await;
    let model;
    match result {
        Ok(res) => {
            model = res;
        }
        Err(e) => {
            errors.push(e);
            model = ActixAdminModel::create_empty();
        }
    }

    let file_name = model
        .get_value::<String>(&column_name, true, true)
        .unwrap_or_default();
    let file_path = format!(
        "{}/{}/{}",
        actix_admin.configuration.file_upload_directory,
        E::get_entity_name(),
        file_name.unwrap_or_default()
    );
    let file = actix_files::NamedFile::open_async(file_path).await;

    match file {
        Ok(file) => Ok(file.into_response(&req)),
        Err(_e) => Ok(HttpResponse::NotFound().content_type("text/html").body("")),
    }
}

pub async fn delete_file<E: ActixAdminViewModelTrait>(
    session: Session,
    data: web::Data<ActixAdmin>,
    db: web::Data<DatabaseConnection>,
    params: web::Path<(i32, String)>,
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

    let (id, column_name) = params.into_inner();
    let mut errors: Vec<crate::ActixAdminError> = Vec::new();
    let result = E::get_entity(db.get_ref(), id, tenant_ref).await;
    let mut model;
    match result {
        Ok(res) => {
            model = res;
        }
        Err(e) => {
            errors.push(e);
            model = ActixAdminModel::create_empty();
        }
    }

    let file_name = model
        .get_value::<String>(&column_name, true, true)
        .unwrap_or_default();
    let file_path = format!(
        "{}/{}/{}",
        actix_admin.configuration.file_upload_directory,
        E::get_entity_name(),
        file_name.unwrap_or_default()
    );
    std::fs::remove_file(file_path).unwrap();
    model.values.remove(&column_name);
    
    let tenant_ref = actix_admin
        .configuration
        .user_tenant_ref
        .map_or(None, |f| f(&session));

    let _edit_res = E::edit_entity(db.get_ref(), id, model.clone(), tenant_ref).await;

    let view_model_field = &view_model
        .fields
        .iter()
        .find(|field| field.field_name == column_name)
        .unwrap();
    ctx.insert("model_field", view_model_field);
    ctx.insert("base_path", &E::get_base_path(&entity_name));
    ctx.insert("model", &model);

    let body = actix_admin
        .tera
        .render("form_elements/input.html", &ctx)
        .map_err(|err| error::ErrorInternalServerError(err))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}
