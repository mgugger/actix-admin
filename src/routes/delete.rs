use super::{render_unauthorized, user_can_access_page, view_model_or_500};
use crate::prelude::*;
use actix_session::Session;
use actix_web::http::header;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use sea_orm::DatabaseConnection;
use tera::Context;

/// Delete file(s) attached to file-upload fields on the given model, best-effort.
fn delete_uploaded_files_for(
    actix_admin: &ActixAdmin,
    entity_name: &str,
    view_model: &ActixAdminViewModel,
    model: &ActixAdminModel,
) {
    for field in view_model.fields {
        if field.field_type != ActixAdminViewModelFieldType::FileUpload {
            continue;
        }
        let file_name = match model
            .get_value::<String>(&field.field_name, true, true)
            .ok()
            .flatten()
        {
            Some(name) if !name.is_empty() => name,
            _ => continue,
        };
        // Defensive: sanitize the DB-stored name too so a poisoned DB value
        // cannot escape the upload folder.
        let safe = crate::model::sanitize_upload_filename(&file_name);
        let file_path = format!(
            "{}/{}/{}",
            actix_admin.configuration.file_upload_directory, entity_name, safe
        );
        if let Err(e) = std::fs::remove_file(&file_path) {
            log::warn!("failed to remove uploaded file {file_path}: {e}");
        }
    }
}

pub async fn delete<E: ActixAdminViewModelTrait>(
    session: Session,
    _req: HttpRequest,
    data: web::Data<ActixAdmin>,
    db: web::Data<DatabaseConnection>,
    id: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    let actix_admin = &data.into_inner();
    let entity_name = E::get_entity_name();

    let view_model = view_model_or_500(actix_admin, &entity_name)?;

    if !user_can_access_page(&session, actix_admin, view_model) {
        let mut ctx = Context::new();
        ctx.insert("render_partial", &true);
        return render_unauthorized(&ctx, actix_admin);
    }

    let db = db.get_ref();
    let id = id.into_inner();

    let tenant_ref = actix_admin
        .configuration
        .user_tenant_ref
        .and_then(|f| f(&session));

    // Fetch first (to know upload paths) then delete.
    let model_result = E::get_entity(db, id, tenant_ref).await;
    let delete_result = E::delete_entity(db, id, tenant_ref).await;

    match (model_result, delete_result) {
        (Ok(model), Ok(_)) => {
            delete_uploaded_files_for(actix_admin, &entity_name, view_model, &model);
            Ok(HttpResponse::Ok().finish())
        }
        (_, Err(e)) if e.ty == crate::ActixAdminErrorType::EntityDoesNotExistError => {
            Ok(HttpResponse::NotFound().finish())
        }
        (_, _) => Ok(HttpResponse::InternalServerError().finish()),
    }
}

pub async fn delete_many<E: ActixAdminViewModelTrait>(
    session: Session,
    _req: HttpRequest,
    data: web::Data<ActixAdmin>,
    db: web::Data<DatabaseConnection>,
    form: web::Form<Vec<(String, String)>>,
) -> Result<HttpResponse, Error> {
    let actix_admin = data.get_ref();
    let entity_name = E::get_entity_name();

    let view_model = view_model_or_500(actix_admin, &entity_name)?;
    let mut errors: Vec<crate::ActixAdminError> = Vec::new();

    if !user_can_access_page(&session, actix_admin, view_model) {
        let mut ctx = Context::new();
        ctx.insert("render_partial", &true);
        return render_unauthorized(&ctx, actix_admin);
    }

    let db = db.get_ref();

    // Silently skip un-parseable ids rather than panicking on client input.
    let ids: Vec<i32> = form
        .iter()
        .filter(|el| el.0 == "ids")
        .filter_map(|el| el.1.parse::<i32>().ok())
        .collect();

    let tenant_ref = actix_admin
        .configuration
        .user_tenant_ref
        .and_then(|f| f(&session));

    // TODO: for large id sets this should be a single DELETE ... WHERE id IN (...);
    // requires a bulk-delete method on the view model trait.
    for id in ids {
        let model_result = E::get_entity(db, id, tenant_ref).await;
        let delete_result = E::delete_entity(db, id, tenant_ref).await;
        match (delete_result, model_result) {
            (Err(e), _) => errors.push(e),
            (Ok(_), Ok(model)) => {
                delete_uploaded_files_for(actix_admin, &entity_name, view_model, &model);
            }
            (Ok(_), Err(e)) => errors.push(e),
        }
    }

    let field = |key: &str, default: &str| -> String {
        form.iter()
            .find(|el| el.0 == key)
            .map(|e| e.1.to_string())
            .unwrap_or_else(|| default.to_string())
    };
    let entities_per_page = field("entities_per_page", "10");
    let search = urlencoding::encode(&field("search", "")).into_owned();
    let sort_by = urlencoding::encode(&field("sort_by", "id")).into_owned();
    let sort_order = field("sort_order", "Asc");
    let page = field("page", "1");

    if errors.is_empty() {
        Ok(HttpResponse::SeeOther()
            .append_header((
                header::LOCATION,
                format!(
                    "list?entities_per_page={entities_per_page}&search={search}&sort_by={sort_by}&sort_order={sort_order}&page={page}"
                ),
            ))
            .finish())
    } else {
        Ok(HttpResponse::InternalServerError().finish())
    }
}
