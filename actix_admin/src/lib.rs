use actix_web::{error, guard, web, Error, HttpRequest, HttpResponse};
use actix_web::{ dev, App, FromRequest};
use actix_web::error::ErrorBadRequest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tera::{Context, Tera};
use futures::future::{ok, err, Ready};
use lazy_static::lazy_static;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use sea_orm::ModelTrait;
use std::pin::Pin;
use std::any::Any;

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
#[derive(Clone, Debug, Serialize)]
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
pub trait ActixAdminModelTrait : Clone {
    async fn list(&self, db: &DatabaseConnection, page: usize, posts_per_page: usize) -> Vec<&str>;
}

#[derive(Clone, Debug, Serialize)]
pub struct ActixAdminModel {
    pub fields: Vec<(&'static str, Field)>,
}

// ActixAdminViewModel
pub trait ActixAdminViewModelTrait : Clone {
    fn get_model_name(&self) -> &str;
}

#[derive(Clone, Debug, Serialize)]
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
    let view_models = Vec::from_iter(data.get_view_model_map().values());

    let mut ctx = Context::new();
    ctx.insert("view_models", &view_models);

    let body = TERA
         .render("index.html", &ctx)
         .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

async fn list<T: AppDataTrait>(req: HttpRequest, data: web::Data<T>) -> Result<HttpResponse, Error> {
    let view_model = data.get_view_model_map().get("posts").unwrap();

    let db = &data.get_db();
    let params = web::Query::<Params>::from_query(req.query_string()).unwrap();

    let page = params.page.unwrap_or(1);
    let posts_per_page = params.posts_per_page.unwrap_or(DEFAULT_POSTS_PER_PAGE);

    let columns: Vec<String> = Vec::new();

    let entities: Vec<&str> = Vec::new(); // view_model.get_entities()

    let mut ctx = Context::new();
    ctx.insert("posts", &entities);
    ctx.insert("page", &page);
    ctx.insert("posts_per_page", &posts_per_page);
    ctx.insert("num_pages", "5" /*&num_pages*/);
    ctx.insert("columns", &columns);

    let body = TERA
         .render("list.html", &ctx)
         .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}