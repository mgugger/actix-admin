use sea_orm::{ConnectOptions, DatabaseConnection};
use actix_admin::prelude::*;

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
    };

    let mut admin_builder = ActixAdminBuilder::new(configuration);
    admin_builder.add_entity::<AppState, Post>(&post_view_model);
    admin_builder.add_entity::<AppState, Comment>(&comment_view_model);

    admin_builder
}
