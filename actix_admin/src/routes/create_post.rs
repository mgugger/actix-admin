use actix_web::http::header;
use actix_web::{web, Error, HttpRequest, HttpResponse};

use crate::prelude::*;

pub async fn create_post<T: ActixAdminAppDataTrait, E: ActixAdminViewModelTrait>(
    _req: HttpRequest,
    data: web::Data<T>,
    text: String,
) -> Result<HttpResponse, Error> {
    let db = &data.get_db();
    let entity_name = E::get_entity_name();
    let actix_admin = data.get_actix_admin();
    let view_model = actix_admin.view_models.get(&entity_name).unwrap();
    let mut admin_model = ActixAdminModel::from(text);
    admin_model = E::create_entity(db, admin_model).await;

    Ok(HttpResponse::Found()
        .append_header((
            header::LOCATION,
            format!("/admin/{}/list", view_model.entity_name),
        ))
        .finish())
}