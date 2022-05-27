use actix_web::{error, web, Error, HttpRequest, HttpResponse};
use tera::{Context};

use crate::prelude::*;

use crate::TERA;

pub async fn create_get<T: ActixAdminAppDataTrait, E: ActixAdminViewModelTrait>(
    _req: HttpRequest,
    data: web::Data<T>,
    _body: web::Payload,
    _text: String,
) -> Result<HttpResponse, Error> {
    let _db = &data.get_db();
    let entity_name = E::get_entity_name();
    let entity_names = &data.get_actix_admin().entity_names;

    let actix_admin = data.get_actix_admin();

    let view_model = actix_admin.view_models.get(&entity_name).unwrap();

    let mut ctx = Context::new();
    ctx.insert("entity_names", &entity_names);
    ctx.insert("view_model", &view_model);
    ctx.insert("model_fields", &view_model.fields);

    let body = TERA
        .render("create.html", &ctx)
        .map_err(|err| error::ErrorInternalServerError(err))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}