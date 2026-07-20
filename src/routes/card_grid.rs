use actix_web::{error, web, Error, HttpRequest, HttpResponse};
use actix_session::Session;
use tera::Context;

use crate::prelude::*;

use super::add_auth_context;

pub async fn display_card_grid(session: Session, data: web::Data<ActixAdmin>, req: HttpRequest) -> Result<HttpResponse, Error> {
    let actix_admin = &data.into_inner();
    let path = req.path().replace(actix_admin.configuration.base_path, "").replace("/", "");
    let card_grid = actix_admin
        .card_grids
        .get(path.as_str())
        .ok_or_else(|| error::ErrorNotFound("Card grid not found"))?;

    let entity_name = actix_admin
        .entity_names
        .values()
        .flatten()
        .find(|el| el.link == path)
        .map(|el| el.name.as_str())
        .unwrap_or("");

    let mut ctx = Context::new();
    ctx.insert("entity_name", entity_name);
    ctx.insert("entity_names", &actix_admin.entity_names);
    ctx.insert("notifications", &Vec::<crate::ActixAdminNotification>::new());
    ctx.insert("card_grid", card_grid);

    add_auth_context(&session, actix_admin, &mut ctx);

    let body = actix_admin.tera
        .render("card_grid.html", &ctx)
        .map_err(|e| error::ErrorInternalServerError(format!("Template error: {e}")))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}
