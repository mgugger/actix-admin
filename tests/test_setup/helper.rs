use actix_admin::prelude::*;
use actix_session::Session;
use actix_web::web;
use actix_web::web::Bytes;
use actix_web::Error;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use chrono::Local;
use sea_orm::prelude::Decimal;
use sea_orm::{ConnectOptions, DatabaseConnection, EntityTrait, Set};

use super::sample_with_tenant_id;
use super::SampleWithTenantId;
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

            let row = sample_with_tenant_id::ActiveModel {
                title: Set(format!("TestTenant{}", i % 2)),
                text: Set("me@home.com".to_string()),
                tenant_id: Set(i % 2),
                ..Default::default()
            };
            let _res = SampleWithTenantId::insert(row)
                .exec(&db)
                .await
                .expect("could not insert sample with tenant id");
        }
    }

    db
}

#[macro_export]
macro_rules! create_app (
    ($db: expr, $enable_auth: expr, $tenant_ref: expr) => ({
        let conn = $db.clone();
        let actix_admin_builder = super::create_actix_admin_builder($enable_auth, $tenant_ref);
        let actix_admin = actix_admin_builder.get_actix_admin();

        test::init_service(
            App::new()
                .app_data(actix_web::web::Data::new(actix_admin))
                .app_data(actix_web::web::Data::new(conn))
                .service(actix_admin_builder.get_scope())
        )
        .await
    });
);

#[macro_export]
macro_rules! create_server (
    ($db: expr, $enable_auth: expr, $tenant_ref: expr) => ({
        use actix_web::{App, HttpServer};
        use actix_admin::builder::ActixAdminBuilderTrait;

        // Create and start the Actix-web server
        let _server = HttpServer::new(move || {
            let conn = $db.clone();
            let actix_admin_builder = create_actix_admin_builder($enable_auth, $tenant_ref);
            let actix_admin = actix_admin_builder.get_actix_admin();

            App::new()
                .app_data(actix_web::web::Data::new(actix_admin))
                .app_data(actix_web::web::Data::new(conn))
                .service(actix_admin_builder.get_scope())
        }
        ).bind("127.0.0.1:5555").unwrap().run().await
        .expect("Failed to run server");
    });
);

pub fn create_actix_admin_builder(
    enable_auth: bool,
    tenant_ref: Option<for<'a> fn(&'a Session) -> Option<i32>>,
) -> ActixAdminBuilder {
    let post_view_model = ActixAdminViewModel::from(Post);
    let comment_view_model = ActixAdminViewModel::from(Comment);
    let sample_with_tenant_id_view_model = ActixAdminViewModel::from(SampleWithTenantId);

    let configuration = ActixAdminConfiguration {
        enable_auth: enable_auth,
        user_tenant_ref: tenant_ref,
        user_is_logged_in: None,
        login_link: None,
        logout_link: None,
        file_upload_directory: "./file_uploads",
        navbar_title: "test",
        base_path: "/admin",
    };

    let mut admin_builder = ActixAdminBuilder::new(configuration);
    admin_builder.add_entity::<Post>(&post_view_model);
    admin_builder.add_entity::<Comment>(&comment_view_model);
    admin_builder.add_entity::<SampleWithTenantId>(&sample_with_tenant_id_view_model);

    admin_builder.add_custom_handler_for_entity::<Comment>(
        "Create Comment From Plaintext",
        "/create_post_from_plaintext",
        web::post().to(create_post_from_plaintext::<Comment>),
        false,
    );

    admin_builder.add_custom_handler_for_entity::<Post>(
        "Create Post From Plaintext",
        "/create_post_from_plaintext",
        web::post().to(create_post_from_plaintext::<Post>),
        false,
    );

    admin_builder.add_custom_handler_for_entity::<SampleWithTenantId>(
        "Create Sample With Tenant Id From Plaintext",
        "/create_post_from_plaintext",
        web::post().to(create_post_from_plaintext::<SampleWithTenantId>),
        false,
    );

    admin_builder.add_custom_handler_for_entity::<Post>(
        "Edit Post From Plaintext",
        "/edit_post_from_plaintext/{id}",
        web::post().to(edit_post_from_plaintext::<Post>),
        false,
    );

    admin_builder.add_custom_handler_for_entity::<Comment>(
        "Edit Comment From Plaintext",
        "/edit_post_from_plaintext/{id}",
        web::post().to(edit_post_from_plaintext::<Comment>),
        false,
    );

    admin_builder
}

async fn create_post_from_plaintext<E: ActixAdminViewModelTrait>(
    session: Session,
    req: HttpRequest,
    data: web::Data<ActixAdmin>,
    db: web::Data<DatabaseConnection>,
    text: String,
) -> Result<HttpResponse, Error> {
    let actix_admin = data.get_ref();
    let model = ActixAdminModel::from(text);
    create_or_edit_post::<E>(&session, req, db, Ok(model), None, actix_admin).await
}

async fn edit_post_from_plaintext<E: ActixAdminViewModelTrait>(
    session: Session,
    req: HttpRequest,
    data: web::Data<ActixAdmin>,
    db: web::Data<DatabaseConnection>,
    text: String,
    id: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    let actix_admin = data.get_ref();
    let model = ActixAdminModel::from(text);
    create_or_edit_post::<E>(
        &session,
        req,
        db,
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
