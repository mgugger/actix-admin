use actix_web::http::header;
use actix_web::{web, error, Error, HttpRequest, HttpResponse};
use tera::{Context};
use crate::TERA;

use crate::prelude::*;

pub async fn edit_post<T: ActixAdminAppDataTrait, E: ActixAdminViewModelTrait>(
    _req: HttpRequest,
    data: web::Data<T>,
    text: String,
    id: web::Path<i32>
) -> Result<HttpResponse, Error> {
    let db = &data.get_db();
    let entity_name = E::get_entity_name();
    let actix_admin = data.get_actix_admin();
    let entity_names = &data.get_actix_admin().entity_names;
    let view_model = actix_admin.view_models.get(&entity_name).unwrap();
    let mut model = ActixAdminModel::from(text);
    model = E::edit_entity(db, id.into_inner(), model).await;

    if model.has_errors() {
        let mut ctx = Context::new();
        ctx.insert("entity_names", &entity_names);
        ctx.insert("view_model", &view_model);
        ctx.insert("model", &model);
        ctx.insert("select_lists", &E::get_select_lists(db).await);
        ctx.insert("list_link", &E::get_list_link(&entity_name));

    let body = TERA
        .render("create_or_edit.html", &ctx)
        .map_err(|err| error::ErrorInternalServerError(err))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
    }
    else {
    Ok(HttpResponse::SeeOther()
        .append_header((
            header::LOCATION,
            format!("/admin/{}/list", view_model.entity_name),
        ))
        .finish())
    }
}