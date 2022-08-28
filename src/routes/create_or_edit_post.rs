use super::{render_unauthorized, user_can_access_page};
use crate::prelude::*;
use crate::TERA;
use actix_session::Session;
use actix_web::http::header;
use actix_web::{error, web, Error, HttpResponse};
use tera::Context;
use actix_multipart::Multipart;
use std::collections::HashMap;

pub async fn create_post<T: ActixAdminAppDataTrait, E: ActixAdminViewModelTrait>(
    session: Session,
    data: web::Data<T>,
    payload: Multipart,
) -> Result<HttpResponse, Error> {
    let model = ActixAdminModel::create_from_payload(payload).await.unwrap();
    create_or_edit_post::<T, E>(&session, &data, model, None).await
}

pub async fn edit_post<T: ActixAdminAppDataTrait, E: ActixAdminViewModelTrait>(
    session: Session,
    data: web::Data<T>,
    payload: Multipart,
    id: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    let model = ActixAdminModel::create_from_payload(payload).await.unwrap();
    create_or_edit_post::<T, E>(&session, &data, model, Some(id.into_inner())).await
}

pub async fn create_or_edit_post<T: ActixAdminAppDataTrait, E: ActixAdminViewModelTrait>(
    session: &Session,
    data: &web::Data<T>,
    mut model: ActixAdminModel,
    id: Option<i32>,
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
    E::validate_entity(&mut model);

    if model.has_errors() {
        let mut ctx = Context::new();
        ctx.insert("entity_names", &actix_admin.entity_names);
        ctx.insert(
            "view_model",
            &ActixAdminViewModelSerializable::from(view_model.clone()),
        );
        ctx.insert("select_lists", &E::get_select_lists(db).await);
        ctx.insert("list_link", &E::get_list_link(&entity_name));
        ctx.insert("model", &model);

        let body = TERA
            .render("create_or_edit.html", &ctx)
            .map_err(|err| error::ErrorInternalServerError(err))?;
        Ok(HttpResponse::Ok().content_type("text/html").body(body))
    } else {
        match id {
            Some(id) => E::edit_entity(db, id, model).await,
            None => E::create_entity(db, model).await,
        };

        Ok(HttpResponse::SeeOther()
            .append_header((
                header::LOCATION,
                format!("/admin/{}/list", view_model.entity_name),
            ))
            .finish())
    }
}

#[doc(hidden)]
impl From<String> for ActixAdminModel {
    fn from(string: String) -> Self {
        let mut hashmap = HashMap::new();
        let key_values: Vec<&str> = string.split('&').collect();
        for key_value in key_values {
            if !key_value.is_empty() {
                let mut iter = key_value.splitn(2, '=');
                hashmap.insert(
                    iter.next().unwrap().to_string().replace("%3A", ":"),
                    iter.next().unwrap().to_string().replace("%3A", ":"),
                );
            }
        }

        ActixAdminModel {
            primary_key: None,
            values: hashmap,
            errors: HashMap::new(),
            custom_errors: HashMap::new(),
        }
    }
}