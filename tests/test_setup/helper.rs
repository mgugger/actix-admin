use actix_admin::prelude::*;
use actix_session::Session;
use actix_web::HttpRequest;
use actix_web::web;
use actix_web::Error;
use actix_web::HttpResponse;
use actix_web::web::Bytes;
use chrono::Local;
use sea_orm::prelude::Decimal;
use sea_orm::{ConnectOptions, DatabaseConnection, EntityTrait, Set};

use super::{comment, create_tables, post, Comment, Post};

pub async fn setup_db(create_entities: bool) -> DatabaseConnection {
    let opt = ConnectOptions::new("sqlite::memory:".to_owned());

    let db = sea_orm::Database::connect(opt).await.unwrap();
    let _ = create_tables(&db).await;

    if create_entities {
        for i in 1..1000 {
            let row = post::ActiveModel {
                title: Set(format!("Test {}", i)),
                text: Set("some content".to_string()),
                tea_mandatory: Set(post::Tea::EverydayTea),
                tea_optional: Set(None),
                insert_date: Set(Local::now().date_naive()),
                ..Default::default()
            };
            let insert_res = Post::insert(row)
                .exec(&db)
                .await
                .expect("could not insert post");

            let row = comment::ActiveModel {
                comment: Set(format!("Test {}", i)),
                user: Set("me@home.com".to_string()),
                my_decimal: Set(Decimal::new(105, 0)),
                insert_date: Set(Local::now().naive_utc()),
                is_visible: Set(i % 2 == 0),
                post_id: Set(Some(insert_res.last_insert_id as i32)),
                ..Default::default()
            };
            let _res = Comment::insert(row)
                .exec(&db)
                .await
                .expect("could not insert comment");
        }
    }

    db
}

#[macro_export]
macro_rules! create_app (
    ($db: expr) => ({
        let conn = $db.clone();
        let actix_admin_builder = super::create_actix_admin_builder();
        let actix_admin = actix_admin_builder.get_actix_admin();
        let app_state = super::AppState {
            db: conn,
            actix_admin,
        };

        test::init_service(
            App::new()
                .app_data(actix_web::web::Data::new(app_state.clone()))
                .service(actix_admin_builder.get_scope::<super::AppState>())
        )
        .await
    });
);

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
        file_upload_directory: "./file_uploads",
    };

    let mut admin_builder = ActixAdminBuilder::new(configuration);
    admin_builder.add_entity::<AppState, Post>(&post_view_model);
    admin_builder.add_entity::<AppState, Comment>(&comment_view_model);

    admin_builder.add_custom_handler_for_entity::<AppState, Comment>(
        "Create Comment From Plaintext",
        "/create_post_from_plaintext",
        web::post().to(create_post_from_plaintext::<AppState, Comment>),
        false,
    );

    admin_builder.add_custom_handler_for_entity::<AppState, Post>(
        "Create Post From Plaintext",
        "/create_post_from_plaintext",
        web::post().to(create_post_from_plaintext::<AppState, Post>),
        false,
    );

    admin_builder.add_custom_handler_for_entity::<AppState, Post>(
        "Edit Post From Plaintext",
        "/edit_post_from_plaintext/{id}",
        web::post().to(edit_post_from_plaintext::<AppState, Post>),
        false,
    );

    admin_builder.add_custom_handler_for_entity::<AppState, Comment>(
        "Edit Comment From Plaintext",
        "/edit_post_from_plaintext/{id}",
        web::post().to(edit_post_from_plaintext::<AppState, Comment>),
        false,
    );

    admin_builder
}

async fn create_post_from_plaintext<T: ActixAdminAppDataTrait, E: ActixAdminViewModelTrait>(
    session: Session,
    req: HttpRequest,
    data: web::Data<T>,
    text: String,
) -> Result<HttpResponse, Error> {
    let actix_admin = data.get_actix_admin();
    let model = ActixAdminModel::from(text);
    create_or_edit_post::<T, E>(&session, req, &data, Ok(model), None, actix_admin).await
}

async fn edit_post_from_plaintext<T: ActixAdminAppDataTrait, E: ActixAdminViewModelTrait>(
    session: Session,
    req: HttpRequest,
    data: web::Data<T>,
    text: String,
    id: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    let actix_admin = data.get_actix_admin();
    let model = ActixAdminModel::from(text);
    create_or_edit_post::<T, E>(
        &session,
        req,
        &data,
        Ok(model),
        Some(id.into_inner()),
        actix_admin,
    )
    .await
}

pub trait BodyTest {
    fn as_str(&self) -> &str;
}

impl BodyTest for Bytes {
    fn as_str(&self) -> &str {
        std::str::from_utf8(self).unwrap()
    }
}