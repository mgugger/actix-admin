mod test_setup;
use test_setup::helper::{create_actix_admin_builder, create_tables_and_get_connection, AppState};

#[cfg(test)]
mod tests {
    extern crate serde_derive;

    use actix_admin::prelude::*;
    use actix_web::http::header::ContentType;
    use actix_web::test;
    use actix_web::{middleware, web, App};
    use serde::{Serialize};
    use sea_orm::EntityTrait;
    use sea_orm::PaginatorTrait;

    #[actix_web::test]
    async fn comment_create_and_edit() {
        let conn = super::create_tables_and_get_connection().await;
        let actix_admin_builder = super::create_actix_admin_builder();
        let actix_admin = actix_admin_builder.get_actix_admin();
        let app_state = super::AppState {
            db: conn,
            actix_admin,
        };
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_state.clone()))
                .service(actix_admin_builder.get_scope::<super::AppState>())
                .wrap(middleware::Logger::default()),
        )
        .await;

        #[derive(Serialize)]
        pub struct CommentModel {
            id: &'static str,
            insert_date: &'static str,
            comment: &'static str,
            user: &'static str,
            is_visible: &'static str,
            post_id: Option<&'static str>,
            my_decimal: &'static str
        }

        let model = CommentModel {
            id: "0",
            insert_date: "1977-04-01T14:00",
            comment: "test",
            user: "test",
            is_visible: "true",
            post_id: None,
            my_decimal: "113.141" // must be larger than 100
        };

        let req = test::TestRequest::post()
            .insert_header(ContentType::form_url_encoded())
            .uri("/admin/comment/create_post_from_plaintext")
            .set_form(model)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_redirection());

        let entities = super::test_setup::Comment::find()
            .paginate(&app_state.db, 50)
            .fetch_page(0)
            .await
            .expect("could not retrieve entities");

        assert!(entities.len() == 1, "After post, db does not contain 1 model");

    }
    
    #[actix_web::test]
    async fn post_create_and_edit() {
        let conn = super::create_tables_and_get_connection().await;
        let actix_admin_builder = super::create_actix_admin_builder();
        let actix_admin = actix_admin_builder.get_actix_admin();
        let app_state = super::AppState {
            db: conn,
            actix_admin,
        };
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_state.clone()))
                .service(actix_admin_builder.get_scope::<super::AppState>())
                .wrap(middleware::Logger::default()),
        )
        .await;

        #[derive(Serialize)]
        pub struct PostModel {
            id: &'static str,
            title: &'static str,
            text: &'static str,
            tea_mandatory: &'static str,
            insert_date: &'static str,
        }

        let model = PostModel {
            id: "0",
            insert_date: "1977-04-01",
            title: "test",
            text: "test",
            tea_mandatory: "EverydayTea"
        };

        let req = test::TestRequest::post()
            .insert_header(ContentType::form_url_encoded())
            .uri("/admin/post/create_post_from_plaintext")
            .set_form(model)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_redirection());

        let entities = super::test_setup::Post::find()
            .paginate(&app_state.db, 50)
            .fetch_page(0)
            .await
            .expect("could not retrieve entities");

        assert!(entities.len() == 1, "After post, db does not contain 1 model");
    }
}
