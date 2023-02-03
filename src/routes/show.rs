use actix_web::HttpRequest;
use actix_web::{error, web, Error, HttpResponse};
use actix_session::{Session};
use tera::{Context};

use crate::ActixAdminNotification;
use crate::prelude::*;

use crate::TERA;
use super::{Params, DEFAULT_ENTITIES_PER_PAGE};
use super::{ add_auth_context, user_can_access_page, render_unauthorized};

pub async fn show<T: ActixAdminAppDataTrait, E: ActixAdminViewModelTrait>(session: Session, req: HttpRequest, data: web::Data<T>, id: web::Path<i32>) -> Result<HttpResponse, Error> {
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

    let params = web::Query::<Params>::from_query(req.query_string()).unwrap();

    let page = params.page.unwrap_or(1);
    let entities_per_page = params
        .entities_per_page
        .unwrap_or(DEFAULT_ENTITIES_PER_PAGE);
    let render_partial = params.render_partial.unwrap_or(false);
    let search = params.search.clone().unwrap_or(String::new());
    let sort_by = params.sort_by.clone().unwrap_or(view_model.primary_key.to_string());
    let sort_order = params.sort_order.as_ref().unwrap_or(&SortOrder::Asc);

    ctx.insert("model", &model);
    ctx.insert("view_model", &ActixAdminViewModelSerializable::from(view_model.clone()));
    ctx.insert("base_path", &E::get_base_path(&entity_name));
    ctx.insert("entity_names", &actix_admin.entity_names);
    ctx.insert("notifications", &notifications);
    ctx.insert("entities_per_page", &entities_per_page);
    ctx.insert("render_partial", &render_partial);
    ctx.insert("search", &search);
    ctx.insert("sort_by", &sort_by);
    ctx.insert("sort_order", &sort_order);
    ctx.insert("page", &page);

    add_auth_context(&session, actix_admin, &mut ctx);

    let body = TERA
        .render("show.html", &ctx)
        .map_err(|err| error::ErrorInternalServerError(format!("{:?}", err)))?;
    Ok(http_response_code.content_type("text/html").body(body))
}