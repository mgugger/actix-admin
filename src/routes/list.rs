use std::fmt;

use crate::prelude::*;
use actix_web::{error, web, Error, HttpRequest, HttpResponse};
use serde::Deserialize;
use serde::Serialize;
use tera::Context;

use super::{
    add_auth_context, render_unauthorized, user_can_access_page, Params, DEFAULT_ENTITIES_PER_PAGE,
};
use crate::ActixAdminModel;
use crate::ActixAdminNotification;
use crate::ActixAdminViewModel;
use crate::ActixAdminViewModelTrait;
use actix_session::Session;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SortOrder {
    Asc,
    Desc,
}

impl fmt::Display for SortOrder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SortOrder::Asc => write!(f, "Asc"),
            SortOrder::Desc => write!(f, "Desc"),
        }
    }
}

pub fn replace_regex(view_model: &ActixAdminViewModel, models: &mut Vec<ActixAdminModel>) {
    view_model
        .fields
        .iter()
        .filter(|f| f.list_regex_mask.is_some())
        .for_each(|f| {
            models.into_iter().for_each(|m| {
                let regex = f.list_regex_mask.as_ref().unwrap();
                let field = f;
                let vals = &mut m.values;
                vals.entry(field.field_name.to_string())
                    .and_modify(|f| *f = regex.replace_all(f, "****").to_string());
            })
        });
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
        return render_unauthorized(&ctx, actix_admin);
    }

    let params = web::Query::<Params>::from_query(req.query_string()).unwrap();

    let mut page = params.page.unwrap_or(1);
    let entities_per_page = params
        .entities_per_page
        .unwrap_or(DEFAULT_ENTITIES_PER_PAGE);
    let render_partial = req.headers().contains_key("HX-Target");
    let search = params.search.clone().unwrap_or(String::new());

    let db = data.get_db();
    let sort_by = params
        .sort_by
        .clone()
        .unwrap_or(view_model.primary_key.to_string());
    let sort_order = params.sort_order.as_ref().unwrap_or(&SortOrder::Asc);

    let result = E::list(db, page, entities_per_page, &search, &sort_by, &sort_order).await;

    match result {
        Ok(res) => {
            let mut entities = res.1;
            replace_regex(view_model, &mut entities);
            let num_pages = std::cmp::max(res.0, 1);
            ctx.insert("entities", &entities);
            ctx.insert("num_pages", &num_pages);
            ctx.insert("page", &std::cmp::min(num_pages, page));
            page = std::cmp::min(page, num_pages);
            let min_show_page = if &page < &5 {
                1
            } else {
                let max_page = &page - &5;
                max_page
            };
            let max_show_page = if &page >= &num_pages {
                std::cmp::max(1, num_pages - 1)
            } else {
                let max_page = &page + &5;
                std::cmp::min(num_pages - 1, max_page)
            };
            ctx.insert("min_show_page", &min_show_page);
            ctx.insert("max_show_page", &max_show_page);
        }
        Err(e) => {
            ctx.insert("entities", &Vec::<ActixAdminModel>::new());
            ctx.insert("num_pages", &0);
            ctx.insert("min_show_page", &1);
            ctx.insert("max_show_page", &1);
            ctx.insert("page", &1);
            errors.push(e);
        }
    }

    let mut http_response_code = match errors.is_empty() {
        false => HttpResponse::InternalServerError(),
        true => HttpResponse::Ok(),
    };
    let notifications: Vec<ActixAdminNotification> = errors
        .into_iter()
        .map(|err| ActixAdminNotification::from(err))
        .collect();

    ctx.insert("entity_name", &entity_name);
    ctx.insert("notifications", &notifications);
    ctx.insert("entities_per_page", &entities_per_page);
    ctx.insert("render_partial", &render_partial);
    ctx.insert(
        "view_model",
        &ActixAdminViewModelSerializable::from(view_model.clone()),
    );
    ctx.insert("search", &search);
    ctx.insert("sort_by", &sort_by);
    ctx.insert("sort_order", &sort_order);

    let body = actix_admin.tera
        .render("list.html", &ctx)
        .map_err(|err| error::ErrorInternalServerError(err))?;
    Ok(http_response_code.content_type("text/html").body(body))
}
