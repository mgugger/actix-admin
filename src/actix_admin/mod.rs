use actix_web::{web, guard, HttpRequest, HttpResponse, Error, error};
use tera::{ Tera, Context};

use crate::entity::Post;
use crate::entity::post;

use sea_orm::{ entity::*, query::*, SelectorTrait, ModelTrait, ConnectionTrait, ColumnTrait, PaginatorTrait, EntityTrait };
use sea_orm::{{ DatabaseConnection, ConnectOptions }};

const DEFAULT_POSTS_PER_PAGE: usize = 5;

#[derive(Debug, Deserialize)]
pub struct Params {
    page: Option<usize>,
    posts_per_page: Option<usize>,
}

async fn index(data: web::Data<super::AppState>) -> &'static str {
    "Welcome!"
}

async fn list<T: EntityTrait>(
    req: HttpRequest,
    data: web::Data<super::AppState>) -> Result<HttpResponse, Error> 
    {
    let db = &data.db;
    let params = web::Query::<Params>::from_query(req.query_string()).unwrap();

    let page = params.page.unwrap_or(1);
    let posts_per_page = params.posts_per_page.unwrap_or(DEFAULT_POSTS_PER_PAGE);
    let paginator = Post::find()
        .order_by_asc(post::Column::Id)
        .paginate(db, posts_per_page);
    let num_pages = paginator.num_pages().await.ok().unwrap();

    let posts = paginator
        .fetch_page(page - 1)
        .await
        .expect("could not retrieve posts");
    
    let mut ctx = Context::new();
    ctx.insert("posts", &posts);
    ctx.insert("page", &page);
    ctx.insert("posts_per_page", &posts_per_page);
    ctx.insert("num_pages", &num_pages);

    let body = data.tmpl
        .render("list.html", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

fn entity_scope<T: EntityTrait>(entity: T) -> actix_web::Scope {
    let entity_name = entity.table_name();
    let scope = web::scope(&format!("/{}",entity_name))
        .route("/list", web::get().to(list::<T>));
    scope
}

pub fn admin_scope() -> actix_web::Scope {
    let scope = web::scope("/admin")
        .route("/", web::get().to(index))
        .service(entity_scope(Post));
    scope
}