use actix_web::error::ErrorBadRequest;
use actix_web::{dev, App, FromRequest, Route};
use actix_web::{error, guard, web, Error, HttpRequest, HttpResponse};
use futures::future::{err, ok, Ready};
use lazy_static::lazy_static;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use sea_orm::ModelTrait;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use std::pin::Pin;
use tera::{Context, Tera};

use async_trait::async_trait;

pub use actix_admin_macros::DeriveActixAdminModel;

const DEFAULT_POSTS_PER_PAGE: usize = 5;

// globals
lazy_static! {
    static ref TERA: Tera =
        Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();
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
    Text,
}

// AppDataTrait
pub trait AppDataTrait {
    fn get_db(&self) -> &DatabaseConnection;
    fn get_actix_admin(&self) -> &ActixAdmin;
}

// ActixAdminModel
#[async_trait]
pub trait ActixAdminModelTrait: Clone {
    async fn list_db(db: &DatabaseConnection, page: usize, posts_per_page: usize) -> Vec<&str>;
}

#[derive(Clone, Debug, Serialize)]
pub struct ActixAdminModel {
    pub fields: Vec<(&'static str, Field)>,
}

// ActixAdminViewModel
#[async_trait(?Send)]
pub trait ActixAdminViewModelTrait {
    async fn list<T: AppDataTrait + Sync + Send>(req: HttpRequest, data: web::Data<T>) -> Result<HttpResponse, Error>;
}

#[derive(Clone, Debug, Serialize)]
pub struct ActixAdminViewModel {
    pub entity_name: String,
}

// ActixAdminController
#[derive(Clone, Debug)]
pub struct ActixAdmin {
    pub entity_names: Vec<String> 
}

impl ActixAdmin {
    pub fn new() -> Self {
        let actix_admin = ActixAdmin {
            entity_names: Vec::new()
        };
        actix_admin
    }

    pub fn add_entity<T: AppDataTrait + 'static>(mut self, view_model: &ActixAdminViewModel) -> Self {
        self.entity_names.push(view_model.entity_name.to_string());
        self
    }

    pub fn create_scope<T: AppDataTrait + 'static>(&self) -> actix_web::Scope {
        let scope = web::scope("/admin").route("/", web::get().to(index::<T>));

        scope
    }
}

async fn index<T: AppDataTrait>(data: web::Data<T>) -> Result<HttpResponse, Error> {

    let entity_names = &data.get_actix_admin().entity_names;
    let mut ctx = Context::new();
    ctx.insert("entity_names", &entity_names);

    let body = TERA
        .render("index.html", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

pub fn list_model<T: AppDataTrait>(req: HttpRequest, data: &web::Data<T>, view_model: ActixAdminViewModel, entity_names: &Vec<String>) -> Result<HttpResponse, Error> {
    let params = web::Query::<Params>::from_query(req.query_string()).unwrap();

    let page = params.page.unwrap_or(1);
    let posts_per_page = params.posts_per_page.unwrap_or(DEFAULT_POSTS_PER_PAGE);

    let columns: Vec<String> = Vec::new();

    let entities: Vec<&str> = Vec::new(); // view_model.get_entities()

    let mut ctx = Context::new();
    ctx.insert("entity_names", &entity_names);
    ctx.insert("posts", &entities);
    ctx.insert("page", &page);
    ctx.insert("posts_per_page", &posts_per_page);
    ctx.insert("num_pages", "5" /*&num_pages*/);
    ctx.insert("columns", &columns);

    let body = TERA
        .render("list.html", &ctx)
        .map_err(|err| error::ErrorInternalServerError(err))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}