use crate::prelude::*;
use crate::view_model::ActixAdminViewModelParams;
use actix_web::http::header::ContentDisposition;
use actix_web::{error, web, Error, HttpRequest, HttpResponse};
use csv::WriterBuilder;
use sea_orm::DatabaseConnection;
use serde_derive::{Deserialize, Serialize};
use std::fmt;
use tera::Context;
use urlencoding::decode;

use super::helpers::{add_default_context, SearchParams};
use super::{
    add_auth_context, render_unauthorized, user_can_access_page, Params, DEFAULT_ENTITIES_PER_PAGE,
};
use crate::ActixAdminModel;
use crate::ActixAdminNotification;
use crate::ActixAdminViewModel;
use crate::ActixAdminViewModelTrait;
use actix_session::Session;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
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
    for field in view_model.fields.iter().filter(|f| f.list_regex_mask.is_some()) {
        let regex = field.list_regex_mask.as_ref().unwrap();
        for model in models.iter_mut() {
            if let Some(value) = model.values.get_mut(&field.field_name) {
                *value = regex.replace_all(value, "****").to_string();
            }
        }
    }
}

pub async fn export_csv<E: ActixAdminViewModelTrait>(
    session: Session,
    req: HttpRequest,
    data: web::Data<ActixAdmin>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, Error> {
    let actix_admin = &data.into_inner();
    let entity_name = E::get_entity_name();
    let view_model = actix_admin.view_models.get(&entity_name).unwrap();

    if !user_can_access_page(&session, actix_admin, view_model) {
        return render_unauthorized(&Context::new(), actix_admin);
    }

    let params = web::Query::<Params>::from_query(req.query_string()).unwrap();
    let search = params.search.clone().unwrap_or_default();
    let sort_by = params.sort_by.clone().unwrap_or_else(|| view_model.primary_key.clone());
    let sort_order = params.sort_order.clone().unwrap_or(SortOrder::Asc);

    let actixadminfilters = decode(req.query_string())
        .unwrap()
        .split('&')
        .filter_map(|qf| {
            if qf.starts_with("filter_") {
                let mut kv = qf.split('=');
                Some(ActixAdminViewModelFilter {
                    name: kv.next()?.strip_prefix("filter_")?.to_string(),
                    value: kv.next().map(|s| s.to_string()).filter(|f| !f.is_empty()),
                    values: None,
                    filter_type: None,
                    foreign_key: None,
                })
            } else {
                None
            }
        })
        .collect();

    let params = ActixAdminViewModelParams {
        page: None,
        entities_per_page: None,
        viewmodel_filter: actixadminfilters,
        search,
        sort_by,
        sort_order,
        tenant_ref: actix_admin.configuration.user_tenant_ref.and_then(|f| f(&session)),
    };

    let entities = match E::list(&db, &params).await {
        Ok(res) => {
            let mut entities = res.1;
            replace_regex(view_model, &mut entities);
            entities
        }
        Err(_) => Vec::new(),
    };

    let mut writer = WriterBuilder::new().from_writer(vec![]);
    let mut fields = view_model.fields.iter().map(|f| f.field_name.clone()).collect::<Vec<_>>();
    fields.insert(0, view_model.primary_key.clone());
    writer.write_record(&fields).ok();

    for entity in entities {
        let mut values = vec![entity.primary_key.unwrap_or_default()];
        for field in view_model.fields {
            let value = entity.values.get(&field.field_name).cloned().unwrap_or_default();
            values.push(entity.fk_values.get(&field.field_name).cloned().unwrap_or(value));
        }
        writer.write_record(&values).ok();
    }

    Ok(HttpResponse::Ok()
        .content_type("text/csv")
        .insert_header(ContentDisposition::attachment("export.csv"))
        .body(writer.into_inner().unwrap()))
}

pub async fn list<E: ActixAdminViewModelTrait>(
    session: Session,
    req: HttpRequest,
    data: web::Data<ActixAdmin>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, Error> {
    let actix_admin = &data.into_inner();
    let entity_name = E::get_entity_name();
    let view_model = actix_admin.view_models.get(&entity_name).unwrap();
    let mut ctx = Context::new();
    add_auth_context(&session, actix_admin, &mut ctx);

    if !user_can_access_page(&session, actix_admin, view_model) {
        return render_unauthorized(&ctx, actix_admin);
    }

    let params = web::Query::<Params>::from_query(req.query_string()).unwrap();
    let page = params.page.unwrap_or(1);
    let entities_per_page = params.entities_per_page.unwrap_or(DEFAULT_ENTITIES_PER_PAGE);
    let search = params.search.clone().unwrap_or_default();
    let sort_by = params.sort_by.clone().unwrap_or_else(|| view_model.primary_key.clone());
    let sort_order = params.sort_order.clone().unwrap_or(SortOrder::Asc);

    let search_params = SearchParams::from_params(&params, view_model);

    let actixadminfilters = decode(req.query_string())
        .unwrap()
        .split('&')
        .filter_map(|qf| {
            if qf.starts_with("filter_") {
                let mut kv = qf.split('=');
                Some(ActixAdminViewModelFilter {
                    name: kv.next()?.strip_prefix("filter_")?.to_string(),
                    value: kv.next().map(|s| s.to_string()).filter(|f| !f.is_empty()),
                    values: None,
                    filter_type: None,
                    foreign_key: None,
                })
            } else {
                None
            }
        })
        .collect();

    let params = ActixAdminViewModelParams {
        page: Some(page),
        entities_per_page: Some(entities_per_page),
        viewmodel_filter: actixadminfilters,
        search,
        sort_by,
        sort_order,
        tenant_ref: actix_admin.configuration.user_tenant_ref.and_then(|f| f(&session)),
    };

    let (num_pages, mut entities) = match E::list(&db, &params).await {
        Ok(res) => res,
        Err(e) => {
            ctx.insert("entities", &Vec::<ActixAdminModel>::new());
            ctx.insert("num_pages", &0);
            ctx.insert("min_show_page", &1);
            ctx.insert("max_show_page", &1);
            ctx.insert("page", &1);
            ctx.insert("notifications", &[ActixAdminNotification::from(e)]);
            return Ok(HttpResponse::InternalServerError().content_type("text/html").body(
                actix_admin.tera.render("list.html", &ctx).map_err(error::ErrorInternalServerError)?,
            ));
        }
    };

    replace_regex(view_model, &mut entities);
    let num_pages = num_pages.unwrap_or(1);
    let page = page.min(num_pages);
    let min_show_page = (page.saturating_sub(4)).max(1);
    let max_show_page = (page + 4).min(num_pages);

    add_default_context(
        &mut ctx,
        req,
        view_model,
        entity_name,
        actix_admin,
        Vec::new(),
        &search_params,
    );

    ctx.insert("entities", &entities);
    ctx.insert("num_pages", &num_pages);
    ctx.insert("min_show_page", &min_show_page);
    ctx.insert("max_show_page", &max_show_page);
    ctx.insert("viewmodel_filter", &E::get_viewmodel_filter(&db).await);

    Ok(HttpResponse::Ok().content_type("text/html").body(
        actix_admin.tera.render("list.html", &ctx).map_err(|err| error::ErrorInternalServerError(format!("{:?}", err)))?,
    ))
}
