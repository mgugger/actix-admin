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
        return render_unauthorized(&ctx, &actix_admin);
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
    form: web::Form<Vec<(String, String)>>,
) -> Result<HttpResponse, Error> {
    let actix_admin = data.get_actix_admin();
    let entity_name = E::get_entity_name();

    let view_model = actix_admin.view_models.get(&entity_name).unwrap();
    let mut errors: Vec<crate::ActixAdminError> = Vec::new();

    if !user_can_access_page(&session, actix_admin, view_model) {
        let mut ctx = Context::new();
        ctx.insert("render_partial", &true);
        return render_unauthorized(&ctx, &actix_admin);
    }

    let db = &data.get_db();
    let entity_name = E::get_entity_name();

    let ids: Vec<i32> = form.iter().filter(|el| el.0 == "ids").map(|el| el.1.parse::<i32>().unwrap()).collect();

    // TODO: implement delete_many
    for id in ids {
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
                format!("/admin/{}/list?entities_per_page={}&search={}&sort_by={}&sort_order={}&page={}", entity_name, entities_per_page, search, sort_by, sort_order, page),
            ))
            .finish()),
        false => Ok(HttpResponse::InternalServerError().finish()),
    }
}
