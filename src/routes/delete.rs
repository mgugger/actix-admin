use super::{render_unauthorized, user_can_access_page};
use crate::prelude::*;
use actix_session::Session;
use actix_web::http::header;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use tera::Context;

pub async fn delete<T: ActixAdminAppDataTrait, E: ActixAdminViewModelTrait>(
    session: Session,
    _req: HttpRequest,
    data: web::Data<T>,
    _text: String,
    id: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    let actix_admin = data.get_actix_admin();
    let entity_name = E::get_entity_name();

    let view_model = actix_admin.view_models.get(&entity_name).unwrap();

    if !user_can_access_page(&session, actix_admin, view_model) {
        let mut ctx = Context::new();
        ctx.insert("render_partial", &true);
        return render_unauthorized(&ctx);
    }

    let db = &data.get_db();
    let id = id.into_inner();
    let model_result = E::get_entity(db, id).await;
    let delete_result = E::delete_entity(db, id).await;

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

pub async fn delete_many<T: ActixAdminAppDataTrait, E: ActixAdminViewModelTrait>(
    session: Session,
    _req: HttpRequest,
    data: web::Data<T>,
    text: String,
) -> Result<HttpResponse, Error> {
    let actix_admin = data.get_actix_admin();
    let entity_name = E::get_entity_name();

    let view_model = actix_admin.view_models.get(&entity_name).unwrap();
    let mut errors: Vec<crate::ActixAdminError> = Vec::new();

    if !user_can_access_page(&session, actix_admin, view_model) {
        let mut ctx = Context::new();
        ctx.insert("render_partial", &true);
        return render_unauthorized(&ctx);
    }

    let db = &data.get_db();
    let entity_name = E::get_entity_name();
    let entity_ids: Vec<i32> = text
        .split("&")
        .filter(|id| !id.is_empty())
        .map(|id_str| id_str.replace("ids=", "").parse::<i32>().unwrap())
        .collect();

    // TODO: implement delete_many
    for id in entity_ids {
        let model_result = E::get_entity(db, id).await;
        let delete_result = E::delete_entity(db, id).await;
        match (delete_result, model_result) {
            (Err(e), _) => errors.push(e),
            (Ok(_), Ok(model)) => {
                for field in view_model.fields {
                    if field.field_type == ActixAdminViewModelFieldType::FileUpload {
                        let file_name = model
                            .get_value::<String>(&field.field_name, true, true)
                            .unwrap_or_default();
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
            (Ok(_), Err(e)) => errors.push(e),
        }
    }

    match errors.is_empty() {
        true => Ok(HttpResponse::SeeOther()
            .append_header((
                header::LOCATION,
                format!("/admin/{}/list?render_partial=true", entity_name),
            ))
            .finish()),
        false => Ok(HttpResponse::InternalServerError().finish()),
    }
}
