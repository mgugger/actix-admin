use actix_web::{error, web, Error, HttpRequest, HttpResponse};
use actix_session::Session;
use tera::Context;

use crate::prelude::*;

use super::add_auth_context;

pub async fn display_card_grid(session: Session, data: web::Data<ActixAdmin>, req: HttpRequest) -> Result<HttpResponse, Error> {
    let actix_admin = &data.into_inner();
    let path = req.path().replace(actix_admin.configuration.base_path, "").replace("/", "");
    let card_grid = actix_admin.card_grids.get(path.as_str());

    if card_grid.is_none() {
        return Err(error::ErrorNotFound("Card grid not found"));
    }
    
    let notifications: Vec<crate::ActixAdminNotification> = Vec::new();

    let mut ctx = Context::new();
    ctx.insert("entity_name", &actix_admin.entity_names.iter().map(|el| el.1).flatten().find(|el| el.link == path).unwrap().name);
    ctx.insert("entity_names", &actix_admin.entity_names);
    ctx.insert("notifications", &notifications);    
    ctx.insert("card_grid", &card_grid.unwrap());

    add_auth_context(&session, actix_admin, &mut ctx);

    let body = actix_admin.tera
        .render("card_grid.html", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Template error {err:?}"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}