use actix_web::{error, web, Error, HttpResponse};
use actix_session::{Session};
use tera::{Context};

use crate::prelude::*;

use crate::TERA;

use super::{ add_auth_context };

pub async fn show<T: ActixAdminAppDataTrait, E: ActixAdminViewModelTrait>(session: Session, data: web::Data<T>, id: web::Path<i32>) -> Result<HttpResponse, Error> {
    let actix_admin = data.get_actix_admin();
    let db = &data.get_db();
    let model = E::get_entity(db, id.into_inner()).await;
    let entity_name = E::get_entity_name();
    let view_model: &ActixAdminViewModel = actix_admin.view_models.get(&entity_name).unwrap();

    let mut ctx = Context::new();
    ctx.insert("model", &model);
    ctx.insert("view_model", &ActixAdminViewModelSerializable::from(view_model.clone()));
    ctx.insert("list_link", &E::get_list_link(&entity_name));
    ctx.insert("entity_names", &actix_admin.entity_names);

    add_auth_context(&session, actix_admin, &mut ctx);

    let body = TERA
        .render("show.html", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}