use actix_web::{web, Error, HttpRequest, HttpResponse};
use serde::{Deserialize};
use actix_web::http::header;
use crate::prelude::*;

pub async fn delete_many_post<T: ActixAdminAppDataTrait, E: ActixAdminViewModelTrait>(
    req: HttpRequest,
    data: web::Data<T>,
    text: String,
) -> Result<HttpResponse, Error> {
    let _db = &data.get_db();
    let entity_name = E::get_entity_name();
    println!("{:?}", text);
    
    Ok(HttpResponse::Found()
    .append_header((
        header::LOCATION,
        format!("/admin/{}/list?render_partial=true", entity_name),
    ))
    .finish())
}