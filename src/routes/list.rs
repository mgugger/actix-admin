use crate::prelude::*;
use actix_web::http::header::ContentDisposition;
use actix_web::{error, web, Error, HttpRequest, HttpResponse};
use csv::WriterBuilder;
use sea_orm::DatabaseConnection;
use serde_derive::{Deserialize, Serialize};
use std::fmt;
use tera::Context;
use urlencoding::decode;

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

pub async fn export_csv<E: ActixAdminViewModelTrait>(
    session: Session,
    req: HttpRequest,
    data: web::Data<ActixAdmin>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, Error> {
    let actix_admin = &data.into_inner();
    let entity_name = E::get_entity_name();
    let view_model: &ActixAdminViewModel = actix_admin.view_models.get(&entity_name).unwrap();

    if !user_can_access_page(&session, actix_admin, view_model) {
        let ctx = Context::new();
        return render_unauthorized(&ctx, actix_admin);
    }

    let params = web::Query::<Params>::from_query(req.query_string()).unwrap();

    let search = params.search.clone().unwrap_or(String::new());

    let sort_by = params
        .sort_by
        .clone()
        .unwrap_or(view_model.primary_key.to_string());
    let sort_order = params.sort_order.as_ref().unwrap_or(&SortOrder::Asc);

    let decoded_querystring = decode(req.query_string()).unwrap();
    let actixadminfilters: Vec<ActixAdminViewModelFilter> = decoded_querystring
        .split("&")
        .filter(|qf| qf.starts_with("filter_"))
        .map(|f| {
            let mut kv = f.split("=");
            let af = ActixAdminViewModelFilter {
                name: kv
                    .next()
                    .unwrap()
                    .strip_prefix("filter_")
                    .unwrap_or_default()
                    .to_string(),
                value: kv.next().map(|s| s.to_string()).filter(|f| !f.is_empty()),
                values: None,
                filter_type: None,
            };
            af
        })
        .collect();

    let result = E::list(
        &db,
        None,
        None,
        actixadminfilters,
        &search,
        &sort_by,
        &sort_order,
    )
    .await;

    let mut entities;
    match result {
        Ok(res) => {
            entities = res.1;
            replace_regex(view_model, &mut entities);
        }
        Err(_) => {
            entities = Vec::new();
        }
    }

    let mut writer = WriterBuilder::new().from_writer(vec![]);

    let mut fields = view_model
        .fields
        .iter()
        .map(|f| f.field_name.clone())
        .collect::<Vec<String>>();

    fields.insert(0, view_model.primary_key.clone());

    let _ = writer.write_record(&fields);

    for entity in entities {
        let mut values = Vec::new();
        values.push(entity.primary_key.unwrap_or_default());
        for field in view_model.fields.iter() {
            let empty_string = "".to_string();
            
            let model_value = entity.values.get(&field.field_name).unwrap_or(&empty_string).clone();
            // TODO: fk_values must be HashMap<Hashmap<String, String>> in case multiply FK share same index
            let fk_value = entity.fk_values.get(&field.field_name);

            match fk_value {
                Some(val) => {
                    values.push(val.clone());
                }
                None => {
                    values.push(model_value);
                }
            }
        }
        let _ = writer.write_record(&values);
    }
    let csv_bytes = writer.into_inner();

    Ok(HttpResponse::Ok()
        .content_type("text/csv")
        .insert_header(ContentDisposition::attachment("export.csv"))
        .body(csv_bytes.unwrap()))
}

pub async fn list<E: ActixAdminViewModelTrait>(
    session: Session,
    req: HttpRequest,
    data: web::Data<ActixAdmin>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, Error> {
    let actix_admin = &data.into_inner();
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

    let sort_by = params
        .sort_by
        .clone()
        .unwrap_or(view_model.primary_key.to_string());
    let sort_order = params.sort_order.as_ref().unwrap_or(&SortOrder::Asc);

    let decoded_querystring = decode(req.query_string()).unwrap();
    let actixadminfilters: Vec<ActixAdminViewModelFilter> = decoded_querystring
        .split("&")
        .filter(|qf| qf.starts_with("filter_"))
        .map(|f| {
            let mut kv = f.split("=");
            let af = ActixAdminViewModelFilter {
                name: kv
                    .next()
                    .unwrap()
                    .strip_prefix("filter_")
                    .unwrap_or_default()
                    .to_string(),
                value: kv.next().map(|s| s.to_string()).filter(|f| !f.is_empty()),
                values: None,
                filter_type: None,
            };
            af
        })
        .collect();

    let result = E::list(
        &db,
        Some(page),
        Some(entities_per_page),
        actixadminfilters,
        &search,
        &sort_by,
        &sort_order,
    )
    .await;

    match result {
        Ok(res) => {
            let mut entities = res.1;
            replace_regex(view_model, &mut entities);
            let num_pages = std::cmp::max(res.0, Some(1));
            ctx.insert("entities", &entities);
            ctx.insert("num_pages", &num_pages);
            ctx.insert("page", &std::cmp::min(num_pages, Some(page)));
            page = std::cmp::min(page, num_pages.unwrap());
            let min_show_page = if &page < &5 {
                1
            } else {
                let max_page = &page - &5;
                max_page
            };
            let max_show_page = if &page >= &num_pages.unwrap() {
                std::cmp::max(1, num_pages.unwrap() - 1)
            } else {
                let max_page = &page + &5;
                std::cmp::min(num_pages.unwrap() - 1, max_page)
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
    ctx.insert("viewmodel_filter", &E::get_viewmodel_filter(&db).await);
    ctx.insert(
        "view_model",
        &ActixAdminViewModelSerializable::from(view_model.clone()),
    );
    ctx.insert("search", &search);
    ctx.insert("sort_by", &sort_by);
    ctx.insert("sort_order", &sort_order);

    let body = actix_admin
        .tera
        .render("list.html", &ctx)
        .map_err(|err| error::ErrorInternalServerError(err))?;
    Ok(http_response_code.content_type("text/html").body(body))
}
