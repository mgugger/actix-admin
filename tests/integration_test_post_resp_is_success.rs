mod test_setup;
use test_setup::helper::{create_actix_admin_builder, create_tables_and_get_connection, AppState};

#[cfg(test)]
mod tests {
    extern crate serde_derive;

    use actix_admin::prelude::*;
    use actix_web::http::header::ContentType;
    use actix_web::test;
    use actix_web::{middleware, web, App};

    #[actix_web::test]
    async fn comment_create_post() {
        test_post_is_success("/admin/comment/create_post_from_plaintext").await
    }
    
    #[actix_web::test]
    async fn post_create_post() {
        test_post_is_success("/admin/post/create_post_from_plaintext").await
    }

    #[actix_web::test]
    async fn post_edit_post() {
        test_post_is_success("/admin/post/edit_post_from_plaintext").await
    }

    #[actix_web::test]
    async fn comment_edit_post() {
        test_post_is_success("/admin/comment/edit_post_from_plaintext").await
    }

    async fn test_post_is_success(url: &str) {
        let conn = super::create_tables_and_get_connection().await;

        let actix_admin_builder = super::create_actix_admin_builder();
        let actix_admin = actix_admin_builder.get_actix_admin();

        let app_state = super::AppState {
            db: conn,
            actix_admin: actix_admin,
        };

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_state.clone()))
                .service(actix_admin_builder.get_scope::<super::AppState>())
                .wrap(middleware::Logger::default()),
        )
        .await;

        let req = test::TestRequest::post()
            .insert_header(ContentType::form_url_encoded())
            .uri(url)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }
}
