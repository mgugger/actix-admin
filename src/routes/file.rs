use crate::admin_prelude;
use crate::prelude::*;
use actix_session::Session;
use actix_web::{error, web, Error, HttpRequest, HttpResponse};
use sea_orm::DatabaseConnection;

use super::{AdminAction, RoutePrelude};

/// Returns the field descriptor if `column_name` refers to a `FileUpload`
/// or `Image` field on the given view model. Rejects anything else to
/// prevent path traversal / disclosure through arbitrary column reads.
fn file_upload_field<'a>(
    view_model: &'a ActixAdminViewModel,
    column_name: &str,
) -> Result<&'a ActixAdminViewModelField, Error> {
    view_model
        .fields
        .iter()
        .find(|f| {
            f.field_name == column_name
                && matches!(
                    f.field_type,
                    ActixAdminViewModelFieldType::FileUpload | ActixAdminViewModelFieldType::Image
                )
        })
        .ok_or_else(|| {
            error::ErrorBadRequest(format!("'{column_name}' is not a file upload field"))
        })
}

pub async fn download<E: ActixAdminViewModelTrait>(
    req: HttpRequest,
    session: Session,
    data: web::Data<ActixAdmin>,
    db: web::Data<DatabaseConnection>,
    params: web::Path<(E::Id, String)>,
) -> Result<HttpResponse, Error> {
    let actix_admin = &data.into_inner();
    let db = &db.into_inner();
    let ctx = admin_prelude!(&session, &req, actix_admin, RoutePrelude::view(), E);

    let (id, column_name) = params.into_inner();
    let _field = file_upload_field(ctx.view_model, &column_name)?;

    let model = match E::get_entity(db, id, ctx.tenant_ref).await {
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
        actix_admin.configuration.file_upload_directory, ctx.entity_name, safe
    );

    match actix_files::NamedFile::open_async(file_path).await {
        Ok(file) => Ok(file.into_response(&req)),
        Err(_) => Ok(HttpResponse::NotFound().content_type("text/html").body("")),
    }
}

pub async fn delete_file<E: ActixAdminViewModelTrait>(
    session: Session,
    req: HttpRequest,
    data: web::Data<ActixAdmin>,
    db: web::Data<DatabaseConnection>,
    params: web::Path<(E::Id, String)>,
) -> Result<HttpResponse, Error> {
    let actix_admin = &data.into_inner();
    let ctx = admin_prelude!(
        &session,
        &req,
        actix_admin,
        RoutePrelude::write(AdminAction::Edit),
        E
    );

    let (id, column_name) = params.into_inner();
    let view_model_field = file_upload_field(ctx.view_model, &column_name)?;

    let mut model = match E::get_entity(db.get_ref(), id.clone(), ctx.tenant_ref).await {
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
            actix_admin.configuration.file_upload_directory, ctx.entity_name, safe
        );
        if let Err(e) = std::fs::remove_file(&file_path) {
            log::warn!("failed to remove uploaded file {file_path}: {e}");
        }
    }
    model.values.remove(&column_name);

    let _edit_res = E::edit_entity(db.get_ref(), id, model.clone(), ctx.tenant_ref).await;

    let mut tctx = tera::Context::new();
    tctx.insert("model_field", view_model_field);
    tctx.insert("entity_name", &ctx.entity_name);
    tctx.insert("model", &model);

    let body = actix_admin
        .tera
        .render("form_elements/input.html", &tctx)
        .map_err(error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}
