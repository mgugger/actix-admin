use actix_web::{error, web, Error, HttpRequest, HttpResponse};
use tera::{Context};

use crate::prelude::*;

use crate::TERA;

pub async fn create_get<T: ActixAdminAppDataTrait, E: ActixAdminViewModelTrait>(
    _req: HttpRequest,
    data: web::Data<T>,
    _body: web::Payload,
    text: String,
) -> Result<HttpResponse, Error> {
    let db = &data.get_db();
    let entity_name = E::get_entity_name();
    let entity_names = &data.get_actix_admin().entity_names;

    let actix_admin = data.get_actix_admin();

    let view_model = actix_admin.view_models.get(&entity_name).unwrap();
    let model = ActixAdminModel::from(text);
    
    let mut ctx = Context::new();
    ctx.insert("entity_names", &entity_names);
    ctx.insert("view_model", &view_model);
    ctx.insert("select_lists", &E::get_select_lists(db).await);
    ctx.insert("list_link", &E::get_list_link(&entity_name));
    ctx.insert("model", &model);

    let body = TERA
        .render("create_or_edit.html", &ctx)
        .map_err(|err| error::ErrorInternalServerError(err))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}