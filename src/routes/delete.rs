use super::{render_unauthorized, user_can_access_page};
use crate::prelude::*;
use actix_session::Session;
use actix_web::http::header;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use sea_orm::DatabaseConnection;
use tera::Context;

pub async fn delete<E: ActixAdminViewModelTrait>(
    session: Session,
    _req: HttpRequest,
    data: web::Data<ActixAdmin>,
    db: web::Data<DatabaseConnection>,
    _text: String,
    id: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    let actix_admin = &data.into_inner();
    let entity_name = E::get_entity_name();

    let view_model = actix_admin.view_models.get(&entity_name).unwrap();

    if !user_can_access_page(&session, actix_admin, view_model) {
        let mut ctx = Context::new();
        ctx.insert("render_partial", &true);
        return render_unauthorized(&ctx, &actix_admin);
    }

    let db = &db.get_ref();
    let id = id.into_inner();

    let tenant_ref = actix_admin
        .configuration
        .user_tenant_ref
        .map_or(None, |f| f(&session));

    let model_result = E::get_entity(db, id, tenant_ref).await;
    let delete_result = E::delete_entity(db, id, tenant_ref).await;

    match (model_result, delete_result) {
        (Ok(model), Ok(_)) => {
            for field in view_model.fields {
                if field.field_type == ActixAdminViewModelFieldType::FileUpload {
                    let file_name = model
                        .get_value::<String>(&field.field_name, true, true)
                        .unwrap_or_default();
                    if file_name.is_some() {
                        let file_path = format!(
                            "{}/{}/{}",
                            actix_admin.configuration.file_upload_directory,
                            E::get_entity_name(),
                            file_name.unwrap()
                        );
                        std::fs::remove_file(file_path)?;
                    }
                }
            }

            Ok(HttpResponse::Ok().finish())
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

    let view_model = actix_admin.view_models.get(&entity_name).unwrap();
    let mut errors: Vec<crate::ActixAdminError> = Vec::new();

    if !user_can_access_page(&session, actix_admin, view_model) {
        let mut ctx = Context::new();
        ctx.insert("render_partial", &true);
        return render_unauthorized(&ctx, &actix_admin);
    }

    let db = &db.get_ref();
    let entity_name = E::get_entity_name();

    let ids: Vec<i32> = form.iter().filter(|el| el.0 == "ids").map(|el| el.1.parse::<i32>().unwrap()).collect();

    // TODO: implement delete_many
        let tenant_ref = actix_admin
        .configuration
        .user_tenant_ref
        .map_or(None, |f| f(&session));

    for id in ids {
        let model_result = E::get_entity(db, id, tenant_ref).await;
        let delete_result = E::delete_entity(db, id, tenant_ref).await;
        match (delete_result, model_result) {
            (Err(e), _) => errors.push(e),
            (Ok(_), Ok(model)) => {
                for field in view_model.fields {
                    if field.field_type == ActixAdminViewModelFieldType::FileUpload {
                        let file_name = model
                            .get_value::<String>(&field.field_name, true, true)
                            .unwrap_or_default();
                        if file_name.is_some() {
                            let file_path = format!(
                                "{}/{}/{}",
                                actix_admin.configuration.file_upload_directory,
                                E::get_entity_name(),
                                file_name.unwrap_or_default()
                            );
                            std::fs::remove_file(file_path)?;
                        }
                    }
                }
            }
            (Ok(_), Err(e)) => errors.push(e),
        }
    }

    let entities_per_page = form.iter()
        .find(|el| el.0 == "entities_per_page")
        .map(|e| e.1.to_string())
        .unwrap_or("10".to_string());
    let search = form.iter()
        .find(|el| el.0 == "search")
        .map(|e| e.1.to_string())
        .unwrap_or_default();
    let sort_by = form.iter()
        .find(|el| el.0 == "sort_by")
        .map(|e| e.1.to_string())
        .unwrap_or("id".to_string());
    let sort_order = form.iter()
        .find(|el| el.0 == "sort_order")
        .map(|e| e.1.to_string())
        .unwrap_or("Asc".to_string());
    let page = form.iter()
        .find(|el| el.0 == "page")
        .map(|e| e.1.to_string())
        .unwrap_or("1".to_string());

    match errors.is_empty() {
        true => Ok(HttpResponse::SeeOther()
            .append_header((
                header::LOCATION,
                format!("list?entities_per_page={0}&search={1}&sort_by={2}&sort_order={3}&page={4}", entities_per_page, search, sort_by, sort_order, page),
            ))
            .finish()),
        false => Ok(HttpResponse::InternalServerError().finish()),
    }
}
