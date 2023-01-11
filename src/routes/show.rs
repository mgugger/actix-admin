use actix_web::{error, web, Error, HttpResponse};
use actix_session::{Session};
use tera::{Context};

use crate::ActixAdminNotification;
use crate::prelude::*;

use crate::TERA;
use super::{ add_auth_context, user_can_access_page, render_unauthorized};

pub async fn show<T: ActixAdminAppDataTrait, E: ActixAdminViewModelTrait>(session: Session, data: web::Data<T>, id: web::Path<i32>) -> Result<HttpResponse, Error> {
    let actix_admin = data.get_actix_admin();
    let db = &data.get_db();

    let mut ctx = Context::new();
    let entity_name = E::get_entity_name();
    let view_model: &ActixAdminViewModel = actix_admin.view_models.get(&entity_name).unwrap();
    if !user_can_access_page(&session, actix_admin, view_model) {
        return render_unauthorized(&ctx);
    }
    
    let mut errors: Vec<crate::ActixAdminError> = Vec::new();
    let result = E::get_entity(db, id.into_inner()).await;
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

    let mut http_response_code = match errors.is_empty() {
        false => HttpResponse::InternalServerError(),
        true => HttpResponse::Ok()
    };    
    let notifications: Vec<ActixAdminNotification> = errors.into_iter()
        .map(|err| ActixAdminNotification::from(err))
        .collect();

    ctx.insert("model", &model);
    ctx.insert("view_model", &ActixAdminViewModelSerializable::from(view_model.clone()));
    ctx.insert("base_path", &E::get_base_path(&entity_name));
    ctx.insert("entity_names", &actix_admin.entity_names);
    ctx.insert("notifications", &notifications);

    add_auth_context(&session, actix_admin, &mut ctx);

    let body = TERA
        .render("show.html", &ctx)
        .map_err(|err| error::ErrorInternalServerError(format!("{:?}", err)))?;
    Ok(http_response_code.content_type("text/html").body(body))
}