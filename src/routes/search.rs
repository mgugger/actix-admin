
use actix_web::{web, Error, HttpRequest, HttpResponse};
use sea_orm::DatabaseConnection;
use tera::Context;
use actix_session::Session;
use serde::{Deserialize, Serialize};
use crate::prelude::*;
use super::list::replace_regex;
use super::{ add_auth_context, render_unauthorized, user_can_access_page};

#[derive(Serialize)]
struct LabelValue {
    label: String,
    value: String
}

#[derive(Serialize)]
struct SearchList {
    items: Vec<LabelValue>
}

#[derive(Debug, Deserialize)]
pub struct SearchParam {
    q: String
}

pub async fn search<E: ActixAdminViewModelTrait>(
    session: Session,
    req: HttpRequest,
    data: web::Data<ActixAdmin>,
    db: web::Data<DatabaseConnection>,
    _body: web::Payload,
    _text: String,
) -> Result<HttpResponse, Error> {
    let db = db.get_ref();

    let actix_admin = &data.get_ref();
    let mut ctx = Context::new();
    add_auth_context(&session, actix_admin, &mut ctx);
    let entity_name = E::get_entity_name();

    let view_model = actix_admin.view_models.get(&entity_name).unwrap();

    if !user_can_access_page(&session, actix_admin, view_model) {
        return render_unauthorized(&ctx, actix_admin);
    };

    let search_query = web::Query::<SearchParam>::from_query(req.query_string()).unwrap();

    let params = ActixAdminViewModelParams {
        page: None,
        entities_per_page: None,
        viewmodel_filter: Vec::new(),
        search: search_query.into_inner().q,
        sort_by: view_model.primary_key.clone(),
        sort_order: SortOrder::Asc,
        tenant_ref: actix_admin.configuration.user_tenant_ref.and_then(|f| f(&session)),
    };

    // TODO: Improve by not loading all values
    let entities = match E::list(&db, &params).await {
        Ok(res) => {
            let mut entities = res.1;
            replace_regex(view_model, &mut entities);
            entities.into_iter().map(|e| LabelValue {
                label: e.display_name.unwrap_or_default(),
                value: e.primary_key.unwrap()
            }).collect()
        }
        Err(_) => Vec::new(),
    };

    Ok(HttpResponse::Ok().json(SearchList { items: entities }))
}