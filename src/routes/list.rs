use actix_web::{error, web, Error, HttpRequest, HttpResponse};
use serde::{Deserialize};
use tera::{Context};
use crate::prelude::*;

use crate::ActixAdminViewModelTrait;
use crate::ActixAdminViewModel;
use crate::ActixAdminModel;
use crate::TERA;
use actix_session::{Session};
use super::{ add_auth_context, user_can_access_page, render_unauthorized};

const DEFAULT_ENTITIES_PER_PAGE: usize = 10;

#[derive(Debug, Deserialize)]
pub struct Params {
    page: Option<usize>,
    entities_per_page: Option<usize>,
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
    let result: (usize, Vec<ActixAdminModel>) = E::list(db, page, entities_per_page, &search).await;
    let entities = result.1;
    let num_pages = result.0;

    ctx.insert("entity_name", &entity_name);
    ctx.insert("entities", &entities);
    ctx.insert("page", &page);
    ctx.insert("params", &entities_per_page);
    ctx.insert("entities_per_page", &entities_per_page);
    ctx.insert("render_partial", &render_partial);
    ctx.insert("num_pages", &num_pages);
    ctx.insert("view_model", &ActixAdminViewModelSerializable::from(view_model.clone()));
    ctx.insert("search", &search);

    let body = TERA
        .render("list.html", &ctx)
        .map_err(|err| error::ErrorInternalServerError(err))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}