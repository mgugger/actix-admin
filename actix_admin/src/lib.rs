use actix_web::http::header;
use actix_web::{error, web, Error, HttpRequest, HttpResponse};
use async_trait::async_trait;
use lazy_static::lazy_static;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tera::{Context, Tera};

pub use actix_admin_macros::DeriveActixAdminModel;

const DEFAULT_ENTITIES_PER_PAGE: usize = 5;

#[macro_export]
macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key.to_string(), $val.to_string()); )*
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
    async fn list_model(
        db: &DatabaseConnection,
        page: usize,
        posts_per_page: usize,
    ) -> Vec<ActixAdminModel>;
    fn get_fields() -> Vec<(String, ActixAdminField)>;
}

#[derive(Clone, Debug, Serialize)]
pub struct ActixAdminModel {
    pub values: HashMap<String, String>,
}

// ActixAdminViewModel
#[async_trait(?Send)]
pub trait ActixAdminViewModelTrait {
    async fn list(
        db: &DatabaseConnection,
        page: usize,
        entities_per_page: usize,
    ) -> Vec<ActixAdminModel>;
    async fn create_entity(db: &DatabaseConnection, model: ActixAdminModel) -> ActixAdminModel;
    fn get_entity_name() -> String;
}

#[derive(Clone, Debug, Serialize)]
pub struct ActixAdminViewModel {
    pub entity_name: String,
    pub fields: Vec<(String, ActixAdminField)>,
}

#[derive(Clone, Debug)]
pub struct ActixAdmin {
    pub entity_names: Vec<String>,
    pub view_models: HashMap<String, ActixAdminViewModel>,
}

pub struct ActixAdminBuilder {
    pub scopes: Vec<actix_web::Scope>,
    pub actix_admin: ActixAdmin,
}

pub trait ActixAdminBuilderTrait {
    fn new() -> Self;
    fn add_entity<T: AppDataTrait + 'static, E: ActixAdminViewModelTrait + 'static>(&mut self, view_model: &ActixAdminViewModel);
    fn get_scope<T: AppDataTrait + 'static>(self) -> actix_web::Scope;
    fn get_actix_admin(&self) -> ActixAdmin;
}

impl ActixAdminBuilderTrait for ActixAdminBuilder {
    fn new() -> Self {
        ActixAdminBuilder {
            actix_admin: ActixAdmin {
                entity_names: Vec::new(),
                view_models: HashMap::new(),
            },
            scopes: Vec::new(),
        }
    }

    fn add_entity<T: AppDataTrait + 'static, E: ActixAdminViewModelTrait + 'static>(
        &mut self,
        view_model: &ActixAdminViewModel,
    ) {
        self.scopes.push(
            web::scope(&format!("/{}", E::get_entity_name()))
                .route("/list", web::get().to(self::list::<T, E>))
                .route("/create", web::get().to(self::create_get::<T, E>))
                .route("/create", web::post().to(self::create_post::<T, E>)),
        );

        self.actix_admin.entity_names.push(E::get_entity_name());
        //let view_model_cloned = view_model.clone();
        let key = E::get_entity_name();
        self.actix_admin.view_models.insert(key, view_model.clone());
    }

    fn get_scope<T: AppDataTrait + 'static>(self) -> actix_web::Scope {
        let mut scope = web::scope("/admin").route("/", web::get().to(index::<T>));
        for entity_scope in self.scopes {
            scope = scope.service(entity_scope);
        }

        scope
    }

    fn get_actix_admin(&self) -> ActixAdmin {
        self.actix_admin.clone()
    }
}

impl From<String> for ActixAdminModel {
    fn from(string: String) -> Self {
        let mut hashmap = HashMap::new();
        let key_values: Vec<&str> = string.split('&').collect();
        for key_value in key_values {
            let mut iter = key_value.splitn(2, '=');
            hashmap.insert(
                iter.next().unwrap().to_string(),
                iter.next().unwrap().to_string(),
            );
        }

        ActixAdminModel { values: hashmap }
    }
}

impl ActixAdminModel {
    pub fn get_value<T: std::str::FromStr>(&self, key: &str) -> Option<T> {
        println!("get value for key {}", key);
        let value = self.values.get(key).unwrap().to_string().parse::<T>();
        match value {
            Ok(val) => Some(val),
            Err(_) => None, //panic!("key {} could not be parsed", key)
        }
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

pub async fn list<T: AppDataTrait, E: ActixAdminViewModelTrait>(
    req: HttpRequest,
    data: web::Data<T>,
) -> Result<HttpResponse, Error> {
    let entity_name = E::get_entity_name();
    let actix_admin = data.get_actix_admin();
    let view_model: &ActixAdminViewModel = actix_admin.view_models.get(&entity_name).unwrap();
    let entity_names = &data.get_actix_admin().entity_names;

    let params = web::Query::<Params>::from_query(req.query_string()).unwrap();

    let page = params.page.unwrap_or(1);
    let entities_per_page = params
        .entities_per_page
        .unwrap_or(DEFAULT_ENTITIES_PER_PAGE);

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

pub async fn create_get<T: AppDataTrait, E: ActixAdminViewModelTrait>(
    _req: HttpRequest,
    data: web::Data<T>,
    _body: web::Payload,
    _text: String,
) -> Result<HttpResponse, Error> {
    let _db = &data.get_db();
    let entity_name = E::get_entity_name();
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

pub async fn create_post<T: AppDataTrait, E: ActixAdminViewModelTrait>(
    _req: HttpRequest,
    data: web::Data<T>,
    text: String,
) -> Result<HttpResponse, Error> {
    let db = &data.get_db();
    let entity_name = E::get_entity_name();
    let actix_admin = data.get_actix_admin();
    let view_model = actix_admin.view_models.get(&entity_name).unwrap();
    let mut admin_model = ActixAdminModel::from(text);
    admin_model = E::create_entity(db, admin_model).await;

    Ok(HttpResponse::Found()
        .append_header((
            header::LOCATION,
            format!("/admin/{}/list", view_model.entity_name),
        ))
        .finish())
}
