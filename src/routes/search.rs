use super::list::replace_regex;
use super::{
    add_auth_context, render_unauthorized, user_can_perform, view_model_or_500, AdminAction,
};
use crate::prelude::*;
use actix_session::Session;
use actix_web::{error, web, Error, HttpRequest, HttpResponse};
use sea_orm::DatabaseConnection;
use serde_derive::{Deserialize, Serialize};
use tera::Context;

#[derive(Serialize)]
struct LabelValue {
    label: String,
    value: String,
}

#[derive(Serialize)]
struct SearchList {
    items: Vec<LabelValue>,
}

#[derive(Debug, Deserialize, Default)]
pub struct SearchParam {
    #[serde(default)]
    q: String,
}

pub async fn search<E: ActixAdminViewModelTrait>(
    session: Session,
    req: HttpRequest,
    data: web::Data<ActixAdmin>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, Error> {
    let db = db.get_ref();

    let actix_admin = data.get_ref();
    let mut ctx = Context::new();
    add_auth_context(&session, actix_admin, &mut ctx);
    let entity_name = E::get_entity_name();

    let view_model = view_model_or_500(actix_admin, &entity_name)?;

    if !user_can_perform(&session, actix_admin, view_model, AdminAction::View) {
        return render_unauthorized(&ctx, actix_admin);
    };

    let search_query: SearchParam =
        serde_urlencoded::from_str(req.query_string()).unwrap_or_default();

    let params = ActixAdminViewModelParams {
        page: None,
        entities_per_page: None,
        viewmodel_filter: Vec::new(),
        search: search_query.q,
        sort_by: view_model.primary_key.clone(),
        sort_order: SortOrder::Asc,
        tenant_ref: actix_admin
            .configuration
            .user_tenant_ref
            .and_then(|f| f(&session)),
    };

    // TODO: Improve by not loading all values (add a limit clause)
    let entities = match E::list(db, &params).await {
        Ok(res) => {
            let mut entities = res.1;
            replace_regex(view_model, &mut entities);
            entities
                .into_iter()
                .filter_map(|e| {
                    let value = e.primary_key?;
                    Some(LabelValue {
                        label: e.display_name.unwrap_or_default(),
                        value,
                    })
                })
                .collect()
        }
        Err(e) => return Err(error::ErrorInternalServerError(e.to_string())),
    };

    Ok(HttpResponse::Ok().json(SearchList { items: entities }))
}
