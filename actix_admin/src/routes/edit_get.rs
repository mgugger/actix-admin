use actix_web::{error, web, Error, HttpRequest, HttpResponse};
use tera::{Context};

use crate::prelude::*;

use crate::TERA;

pub async fn edit_get<T: ActixAdminAppDataTrait, E: ActixAdminViewModelTrait>(
    _req: HttpRequest,
    data: web::Data<T>,
    _text: String,
    id: web::Path<i32>
) -> Result<HttpResponse, Error> {
    let db = &data.get_db();
    let entity_name = E::get_entity_name();
    let entity_names = &data.get_actix_admin().entity_names;

    let actix_admin = data.get_actix_admin();

    let view_model = actix_admin.view_models.get(&entity_name).unwrap();

    let model = E::get_entity(db, id.into_inner()).await;

    let mut ctx = Context::new();
    ctx.insert("entity_names", &entity_names);
    ctx.insert("view_model", &view_model);
    ctx.insert("model", &model);
    ctx.insert("select_lists", &E::get_select_lists(db).await);
    ctx.insert("list_link", &E::get_list_link(&entity_name));

    let body = TERA
        .render("edit.html", &ctx)
        .map_err(|err| error::ErrorInternalServerError(err))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}