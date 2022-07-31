use actix_web::{web, Error, HttpRequest, HttpResponse};

use crate::prelude::*;

pub async fn delete_post<T: ActixAdminAppDataTrait, E: ActixAdminViewModelTrait>(
    _req: HttpRequest,
    data: web::Data<T>,
    _text: String,
    id: web::Path<i32>
) -> Result<HttpResponse, Error> {
    let db = &data.get_db();

    let _result = E::delete_entity(db, id.into_inner()).await;

    Ok(HttpResponse::Ok()
        .finish())
}