use actix_web::{error, web, Error, HttpRequest, HttpResponse};
use serde::{Deserialize};
use tera::{Context};
use crate::prelude::*;

use crate::ActixAdminViewModelTrait;
use crate::ActixAdminViewModel;
use crate::ActixAdminModel;
use crate::ActixAdminNotification;
use crate::TERA;
use actix_session::{Session};
use super::{ add_auth_context, user_can_access_page, render_unauthorized};

const DEFAULT_ENTITIES_PER_PAGE: u64 = 10;

#[derive(Debug, Deserialize)]
pub struct Params {
    page: Option<u64>,
    entities_per_page: Option<u64>,
    render_partial: Option<bool>,
    search: Option<String>
}

pub async fn list<T: ActixAdminAppDataTrait, E: ActixAdminViewModelTrait>(
    session: Session,
    req: HttpRequest,
    data: web::Data<T>,
) -> Result<HttpResponse, Error> {
    let actix_admin = data.get_actix_admin();
    let entity_name = E::get_entity_name();
    let view_model: &ActixAdminViewModel = actix_admin.view_models.get(&entity_name).unwrap();
    let mut errors: Vec<ActixAdminError> = Vec::new();
    
    let mut ctx = Context::new();
    add_auth_context(&session, actix_admin, &mut ctx);

    ctx.insert("entity_names", &actix_admin.entity_names);

    if !user_can_access_page(&session, actix_admin, view_model) {
        return render_unauthorized(&ctx);
    }

    let params = web::Query::<Params>::from_query(req.query_string()).unwrap();

    let page = params.page.unwrap_or(1);
    let entities_per_page = params
        .entities_per_page
        .unwrap_or(DEFAULT_ENTITIES_PER_PAGE);
    let render_partial = params.render_partial.unwrap_or(false);
    let search = params.search.clone().unwrap_or(String::new());

    let db = data.get_db();
    let result = E::list(db, page, entities_per_page, &search).await;
    match result {
        Ok(res) => {
            let entities = res.1;
            let num_pages = res.0;
            ctx.insert("entities", &entities);
            ctx.insert("num_pages", &num_pages);
        },
        Err(e) => {
            ctx.insert("entities", &Vec::<ActixAdminModel>::new());
            ctx.insert("num_pages", &0);
            errors.push(e);
        }
    }

    let mut http_response_code = match errors.is_empty() {
        false => HttpResponse::InternalServerError(),
        true => HttpResponse::Ok()
    };    
    let notifications: Vec<ActixAdminNotification> = errors.into_iter()
        .map(|err| ActixAdminNotification::from(err))
        .collect();

    ctx.insert("entity_name", &entity_name);
    ctx.insert("notifications", &notifications);
    ctx.insert("page", &page);
    ctx.insert("params", &entities_per_page);
    ctx.insert("entities_per_page", &entities_per_page);
    ctx.insert("render_partial", &render_partial);
    ctx.insert("view_model", &ActixAdminViewModelSerializable::from(view_model.clone()));
    ctx.insert("search", &search);

    let body = TERA
        .render("list.html", &ctx)
        .map_err(|err| error::ErrorInternalServerError(err))?;
    Ok(http_response_code.content_type("text/html").body(body))
}