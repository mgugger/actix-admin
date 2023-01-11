use sea_orm::{ConnectOptions, DatabaseConnection};
use actix_admin::prelude::*;
use actix_web::Error;
use actix_session::Session;
use actix_web::HttpResponse;
use actix_web::{web};

use super::{Post, Comment, create_tables};

pub async fn create_tables_and_get_connection() -> DatabaseConnection {
    let opt = ConnectOptions::new("sqlite::memory:".to_owned());

    let conn = sea_orm::Database::connect(opt).await.unwrap();
    let _ = create_tables(&conn).await;

    conn
}

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub actix_admin: ActixAdmin,
}

impl ActixAdminAppDataTrait for AppState {
    fn get_db(&self) -> &DatabaseConnection {
        &self.db
    }

    fn get_actix_admin(&self) -> &ActixAdmin {
        &self.actix_admin
    }
}

pub fn create_actix_admin_builder() -> ActixAdminBuilder {
    let post_view_model = ActixAdminViewModel::from(Post);
    let comment_view_model = ActixAdminViewModel::from(Comment);

    let configuration = ActixAdminConfiguration {
        enable_auth: false,
        user_is_logged_in: None,
        login_link: None,
        logout_link: None,
        file_upload_directory: "./file_uploads"
    };

    let mut admin_builder = ActixAdminBuilder::new(configuration);
    admin_builder.add_entity::<AppState, Post>(&post_view_model);
    admin_builder.add_entity::<AppState, Comment>(&comment_view_model);

    admin_builder.add_custom_handler_for_entity::<AppState, Comment>(
        "Create Comment From Plaintext",
        "/create_post_from_plaintext", 
        web::post().to(create_post_from_plaintext::<AppState, Comment>), false);

    admin_builder.add_custom_handler_for_entity::<AppState, Post>(
        "Create Post From Plaintext",
        "/create_post_from_plaintext", 
        web::post().to(create_post_from_plaintext::<AppState, Post>), false);

    admin_builder.add_custom_handler_for_entity::<AppState, Post>(
        "Edit Post From Plaintext",
        "/edit_post_from_plaintext/{id}", 
        web::post().to(edit_post_from_plaintext::<AppState, Post>), false);

    admin_builder.add_custom_handler_for_entity::<AppState, Comment>(
        "Edit Comment From Plaintext",
        "/edit_post_from_plaintext/{id}", 
        web::post().to(edit_post_from_plaintext::<AppState, Comment>), false);

    admin_builder
}

async fn create_post_from_plaintext<
    T: ActixAdminAppDataTrait,
    E: ActixAdminViewModelTrait,
>(
    session: Session,
    data: web::Data<T>,
    text: String,
) -> Result<HttpResponse, Error> {
    let actix_admin = data.get_actix_admin();
    let model = ActixAdminModel::from(text);
    create_or_edit_post::<T, E>(&session, &data, Ok(model), None, actix_admin).await
}

async fn edit_post_from_plaintext<
    T: ActixAdminAppDataTrait,
    E: ActixAdminViewModelTrait,
>(
    session: Session,
    data: web::Data<T>,
    text: String,
    id: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    let actix_admin = data.get_actix_admin();
    let model = ActixAdminModel::from(text);
    create_or_edit_post::<T, E>(&session, &data, Ok(model), Some(id.into_inner()), actix_admin).await
}