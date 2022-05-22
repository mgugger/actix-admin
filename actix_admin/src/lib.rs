use actix_web::{error, web, Error, HttpRequest, HttpResponse};
use lazy_static::lazy_static;
use sea_orm::{ DatabaseConnection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use actix_web::http::header;
use tera::{Context, Tera};
use async_trait::async_trait;

pub use actix_admin_macros::DeriveActixAdminModel;

const DEFAULT_ENTITIES_PER_PAGE: usize = 5;

#[macro_export]
macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key, $val); )*
         map
    }}
}

// globals
lazy_static! {
    static ref TERA: Tera =
        Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();
}

// Paging
#[derive(Debug, Deserialize)]
pub struct Params {
    page: Option<usize>,
    entities_per_page: Option<usize>,
}

// Fields
#[derive(Clone, Debug, Serialize)]
pub enum ActixAdminField {
    Text,
}

// AppDataTrait
pub trait AppDataTrait {
    fn get_db(&self) -> &DatabaseConnection;
    fn get_actix_admin(&self) -> &ActixAdmin;
}

// ActixAdminModel
#[async_trait]
pub trait ActixAdminModelTrait {
    async fn list_model(db: &DatabaseConnection, page: usize, posts_per_page: usize) -> Vec<ActixAdminModel>;
    fn get_fields() -> Vec<(String, ActixAdminField)>;
}

#[derive(Clone, Debug, Serialize)]
pub struct ActixAdminModel {
    pub values: HashMap<&'static str, String>
}

// ActixAdminViewModel
#[async_trait(?Send)]
pub trait ActixAdminViewModelTrait {
    async fn list(db: &DatabaseConnection, page: usize, entities_per_page: usize) -> Vec<ActixAdminModel>;
    async fn create_entity(db: &DatabaseConnection, model: ActixAdminModel) -> ActixAdminModel;
}

#[derive(Clone, Debug, Serialize)]
pub struct ActixAdminViewModel {
    pub entity_name: String,
    pub fields: Vec<(String, ActixAdminField)>,
}

// ActixAdminController
#[derive(Clone, Debug)]
pub struct ActixAdmin {
    pub entity_names: Vec<String>, 
    pub view_models: HashMap<String, ActixAdminViewModel>
}

impl ActixAdmin {
    pub fn new() -> Self {
        let actix_admin = ActixAdmin {
            entity_names: Vec::new(),
            view_models: HashMap::new()
        };
        actix_admin
    }

    pub fn add_entity(mut self, view_model: &ActixAdminViewModel) -> Self {
        self.entity_names.push(view_model.entity_name.to_string());
        let view_model_cloned = view_model.clone();
        let key = view_model.entity_name.to_string();
        self.view_models.insert(key, view_model_cloned);
        self
    }

    pub fn create_scope<T: AppDataTrait + 'static >(&self) -> actix_web::Scope {
        let scope = web::scope("/admin").route("/", web::get().to(index::<T>));
        scope
    }
}

pub async fn index<T: AppDataTrait>(data: web::Data<T>) -> Result<HttpResponse, Error> {
    let entity_names = &data.get_actix_admin().entity_names;
    let mut ctx = Context::new();
    ctx.insert("entity_names", &entity_names);

    let body = TERA
        .render("index.html", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

pub async fn list<T: AppDataTrait, E: ActixAdminViewModelTrait>(req: HttpRequest, data: web::Data<T>, path: web::Path<String>) -> Result<HttpResponse, Error> {
    let entity_name: String = path.into_inner();
    let actix_admin = data.get_actix_admin();
    let view_model: &ActixAdminViewModel = actix_admin.view_models.get(&entity_name).unwrap();
    let entity_names = &data.get_actix_admin().entity_names;

    let params = web::Query::<Params>::from_query(req.query_string()).unwrap();

    let page = params.page.unwrap_or(1);
    let entities_per_page = params.entities_per_page.unwrap_or(DEFAULT_ENTITIES_PER_PAGE);

    let db = data.get_db();
    let entities: Vec<ActixAdminModel> = E::list(db, page, entities_per_page).await;

    let mut ctx = Context::new();
    ctx.insert("entity_names", &entity_names);
    ctx.insert("entities", &entities);
    ctx.insert("page", &page);
    ctx.insert("entities_per_page", &entities_per_page);
    ctx.insert("num_pages", "5" /*&num_pages*/);
    ctx.insert("view_model", &view_model);

    let body = TERA
        .render("list.html", &ctx)
        .map_err(|err| error::ErrorInternalServerError(err))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

pub async fn create_get<T: AppDataTrait, E: ActixAdminViewModelTrait>(_req: HttpRequest, data: web::Data<T>, _body: web::Payload, _text: String, entity_name: web::Path<String>) -> Result<HttpResponse, Error>   {
    let _db = &data.get_db();
    let entity_name: String = entity_name.into_inner();
    println!("{}", &entity_name);
    let entity_names = &data.get_actix_admin().entity_names;

    let actix_admin = data.get_actix_admin();

    let view_model = actix_admin.view_models.get(&entity_name).unwrap();

    let mut ctx = Context::new();
    ctx.insert("entity_names", &entity_names);
    ctx.insert("view_model", &view_model);
    ctx.insert("model_fields", &view_model.fields);

    let body = TERA
        .render("create.html", &ctx)
        .map_err(|err| error::ErrorInternalServerError(err))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

pub async fn create_post<T: AppDataTrait, E: ActixAdminViewModelTrait>(_req: HttpRequest, data: web::Data<T>, text: String, entity_name: web::Path<String>) -> Result<HttpResponse, Error> {
    let db = &data.get_db();
    let entity_name: String = entity_name.into_inner();
    let actix_admin = data.get_actix_admin();
    
    let view_model = actix_admin.view_models.get(&entity_name).unwrap();
    let mut model = ActixAdminModel{ values: HashMap::new() };
    model = E::create_entity(db, model).await;

    println!("{}", &entity_name);
    println!("{}", &text);

    Ok(HttpResponse::Found()
        .append_header((header::LOCATION, format!("/admin/{}/list", view_model.entity_name)))
        .finish())
}