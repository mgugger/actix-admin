use actix_web::{error, guard, web, Error, HttpRequest, HttpResponse};
use actix_web::{ dev, App, FromRequest};
use actix_web::error::ErrorBadRequest;
use serde_derive::Deserialize;
use std::collections::HashMap;
use tera::{Context, Tera};
use futures::future::{ok, err, Ready};
use lazy_static::lazy_static;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use sea_orm::ModelTrait;

use async_trait::async_trait;

pub use actix_admin_macros::DeriveActixAdminModel;

const DEFAULT_POSTS_PER_PAGE: usize = 5;

// templates
lazy_static! {
    static ref TERA: Tera = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();
}

// Paging
#[derive(Debug, Deserialize)]
pub struct Params {
    page: Option<usize>,
    posts_per_page: Option<usize>,
}

// Fields
#[derive(Clone, Debug)]
pub enum Field {
    Text
}

// AppDataTrait
pub trait AppDataTrait {
    fn get_db(&self) -> &DatabaseConnection;
    fn get_view_model_map(&self) -> &HashMap<&'static str, ActixAdminViewModel>;
}

// ActixAdminModel
#[async_trait]
pub trait ActixAdminModelTrait {
    async fn list(db: &DatabaseConnection, page: usize, posts_per_page: usize) -> Vec<ActixAdminModel>;
}

#[derive(Clone, Debug)]
pub struct ActixAdminModel {
    pub fields: Vec<(&'static str, Field)>
}

// ActixAdminViewModel
pub trait ActixAdminViewModelTrait : Clone {
    fn get_model_name(&self) -> &str;
    //fn get_entities() -> Vec<ActixAdminModel>;
}

impl ActixAdminViewModelTrait for ActixAdminViewModel {
    fn get_model_name(&self) -> &str {
        &self.entity_name
    }
}

#[derive(Clone, Debug)]
pub struct ActixAdminViewModel {
    pub entity_name: &'static str,
    pub admin_model: ActixAdminModel
}

// ActixAdminController
#[derive(Clone, Debug)]
pub struct ActixAdmin {
    view_models: HashMap<&'static str, ActixAdminViewModel>,
}

impl ActixAdmin {
    pub fn new() -> Self {
        let actix_admin = ActixAdmin {
            view_models: HashMap::new(),
        };
        actix_admin
    }

    pub fn create_scope<T: AppDataTrait + 'static>(self, _app_state: &T) -> actix_web::Scope {
        let mut scope = web::scope("/admin").route("/", web::get().to(index::<T>));

        for view_model in self.view_models {
            scope = scope.service(
                web::scope(&format!("/{}", view_model.0)).route("/list", web::get().to(list::<T>))
            );
        }

        scope
    }

    pub fn add_entity(mut self, view_model: ActixAdminViewModel) -> Self {
        self.view_models.insert(view_model.entity_name, view_model);
        self
    }

    pub fn get_view_model_map(&self) -> HashMap<&'static str, ActixAdminViewModel> {
        self.view_models.clone()
    }
}

async fn index<T: AppDataTrait>(data: web::Data<T>) -> Result<HttpResponse, Error> {
    let keys = Vec::from_iter(data.get_view_model_map().keys());

    let mut ctx = Context::new();
    ctx.insert("view_models", &keys);

    let body = TERA
         .render("index.html", &ctx)
         .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

async fn list<T: AppDataTrait>(req: HttpRequest, data: web::Data<T>) -> Result<HttpResponse, Error> {
    let db = &data.get_db();
    let params = web::Query::<Params>::from_query(req.query_string()).unwrap();

    let page = params.page.unwrap_or(1);
    let posts_per_page = params.posts_per_page.unwrap_or(DEFAULT_POSTS_PER_PAGE);

    let columns: Vec<String> = Vec::new();

    // let paginator = post::Entity::find()
    //     .order_by_asc(post::Column::Id)
    //     .paginate(db, posts_per_page);
    //let num_pages = paginator.num_pages().await.ok().unwrap();

    let posts: Vec<&str> = Vec::new();
    //let posts = paginator
    //     .fetch_page(page - 1)
    //     .await
    //     .expect("could not retrieve posts");
    let mut ctx = Context::new();
    ctx.insert("posts", &posts);
    ctx.insert("page", &page);
    ctx.insert("posts_per_page", &posts_per_page);
    ctx.insert("num_pages", "5" /*&num_pages*/);
    ctx.insert("columns", &columns);

    // let body = data.tmpl
    //     .render("list.html", &ctx)
    //     .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    //Ok(HttpResponse::Ok().content_type("text/html").body(body))
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body("<html></html>"))
}