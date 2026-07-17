use crate::prelude::*;
use actix_session::Session;
use actix_web::{error, web, Error, HttpRequest, HttpResponse};
use sea_orm::DatabaseConnection;
use tera::Context;

use super::{render_unauthorized, user_can_access_page, view_model_or_500};

/// Returns the field descriptor if `column_name` refers to a `FileUpload` field
/// on the given view model. Rejects anything else to prevent path traversal /
/// disclosure through arbitrary column reads.
fn file_upload_field<'a>(
    view_model: &'a ActixAdminViewModel,
    column_name: &str,
) -> Result<&'a ActixAdminViewModelField, Error> {
    view_model
        .fields
        .iter()
        .find(|f| f.field_name == column_name && f.field_type == ActixAdminViewModelFieldType::FileUpload)
        .ok_or_else(|| error::ErrorBadRequest(format!("'{column_name}' is not a file upload field")))
}

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
    let view_model: &ActixAdminViewModel = view_model_or_500(actix_admin, &entity_name)?;
    if !user_can_access_page(&session, actix_admin, view_model) {
        return render_unauthorized(&ctx, actix_admin);
    }

    let (id, column_name) = params.into_inner();
    let _field = file_upload_field(view_model, &column_name)?;

    let tenant_ref = actix_admin
        .configuration
        .user_tenant_ref
        .and_then(|f| f(&session));

    let model = match E::get_entity(db, id, tenant_ref).await {
        Ok(m) => m,
        Err(e) if e.ty == crate::ActixAdminErrorType::EntityDoesNotExistError => {
            return Ok(HttpResponse::NotFound().finish());
        }
        Err(e) => return Err(error::ErrorInternalServerError(e.to_string())),
    };

    let file_name = model
        .get_value::<String>(&column_name, true, true)
        .ok()
        .flatten()
        .unwrap_or_default();
    if file_name.is_empty() {
        return Ok(HttpResponse::NotFound().finish());
    }
    let safe = crate::model::sanitize_upload_filename(&file_name);
    let file_path = format!(
        "{}/{}/{}",
        actix_admin.configuration.file_upload_directory, entity_name, safe
    );

    match actix_files::NamedFile::open_async(file_path).await {
        Ok(file) => Ok(file.into_response(&req)),
        Err(_) => Ok(HttpResponse::NotFound().content_type("text/html").body("")),
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
    let view_model: &ActixAdminViewModel = view_model_or_500(actix_admin, &entity_name)?;
    if !user_can_access_page(&session, actix_admin, view_model) {
        return render_unauthorized(&ctx, actix_admin);
    }

    let (id, column_name) = params.into_inner();
    let view_model_field = file_upload_field(view_model, &column_name)?;

    let tenant_ref = actix_admin
        .configuration
        .user_tenant_ref
        .and_then(|f| f(&session));

    let mut model = match E::get_entity(db.get_ref(), id, tenant_ref).await {
        Ok(m) => m,
        Err(e) if e.ty == crate::ActixAdminErrorType::EntityDoesNotExistError => {
            return Ok(HttpResponse::NotFound().finish());
        }
        Err(e) => return Err(error::ErrorInternalServerError(e.to_string())),
    };

    if let Some(file_name) = model
        .get_value::<String>(&column_name, true, true)
        .ok()
        .flatten()
        .filter(|s| !s.is_empty())
    {
        let safe = crate::model::sanitize_upload_filename(&file_name);
        let file_path = format!(
            "{}/{}/{}",
            actix_admin.configuration.file_upload_directory, entity_name, safe
        );
        if let Err(e) = std::fs::remove_file(&file_path) {
            log::warn!("failed to remove uploaded file {file_path}: {e}");
        }
    }
    model.values.remove(&column_name);

    let _edit_res = E::edit_entity(db.get_ref(), id, model.clone(), tenant_ref).await;

    ctx.insert("model_field", view_model_field);
    ctx.insert("entity_name", &entity_name);
    ctx.insert("model", &model);

    let body = actix_admin
        .tera
        .render("form_elements/input.html", &ctx)
        .map_err(error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}
