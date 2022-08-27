mod test_setup;
use test_setup::helper::{AppState, create_tables_and_get_connection, create_actix_admin_builder};

#[cfg(test)]
mod tests {
    extern crate serde_derive;

    use actix_web::test;
    use actix_web::{middleware, web, App};

    use actix_admin::prelude::*;

    #[actix_web::test]
    async fn test_admin_index_get() {
        test_get_is_success("/admin/").await
    }

    #[actix_web::test]
    async fn test_post_list() {
        test_get_is_success("/admin/post/list").await
    }

    #[actix_web::test]
    async fn test_comment_list() {
        test_get_is_success("/admin/comment/list").await
    }

    #[actix_web::test]
    async fn test_post_create() {
        test_get_is_success("/admin/post/create").await
    }

    #[actix_web::test]
    async fn test_comment_create() {
        test_get_is_success("/admin/comment/create").await
    }

    async fn test_get_is_success(url: &str) {
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

        let req = test::TestRequest::get()
            .uri(url)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }
}
