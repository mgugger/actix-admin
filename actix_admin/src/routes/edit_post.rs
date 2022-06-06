use actix_web::http::header;
use actix_web::{web, Error, HttpRequest, HttpResponse};

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
    let view_model = actix_admin.view_models.get(&entity_name).unwrap();
    let mut _admin_model = ActixAdminModel::from(text);
    _admin_model = E::edit_entity(db, id.into_inner(), _admin_model).await;

    Ok(HttpResponse::Found()
        .append_header((
            header::LOCATION,
            format!("/admin/{}/list", view_model.entity_name),
        ))
        .finish())
}