use actix_web::http::header;
use actix_web::{web, Error, HttpRequest, HttpResponse};

use crate::prelude::*;

pub async fn delete_post<T: ActixAdminAppDataTrait, E: ActixAdminViewModelTrait>(
    _req: HttpRequest,
    data: web::Data<T>,
    text: String,
    id: web::Path<i32>
) -> Result<HttpResponse, Error> {
    let db = &data.get_db();
    let entity_name = E::get_entity_name();
    let actix_admin = data.get_actix_admin();
    //let view_model = actix_admin.view_models.get(&entity_name).unwrap();
    
    // TODO:handle any errors
    let _result = E::delete_entity(db, id.into_inner()).await;

    Ok(HttpResponse::Ok()
        .finish())
}