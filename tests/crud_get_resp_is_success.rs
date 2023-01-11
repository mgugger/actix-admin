mod test_setup;
use test_setup::helper::{AppState, create_tables_and_get_connection, create_actix_admin_builder};

#[cfg(test)]
mod tests {
    extern crate serde_derive;
    use actix_admin::prelude::*;
    use actix_web::test;
    use actix_web::{web, App};
    use super::create_app;

    #[actix_web::test]
    async fn admin_index_get() {
        test_get_is_success("/admin/").await
    }

    #[actix_web::test]
    async fn post_list_get() {
        test_get_is_success("/admin/post/list").await
    }

    #[actix_web::test]
    async fn comment_list_get() {
        test_get_is_success("/admin/comment/list").await
    }

    #[actix_web::test]
    async fn post_create_get() {
        test_get_is_success("/admin/post/create").await
    }

    #[actix_web::test]
    async fn comment_create_get() {
        test_get_is_success("/admin/comment/create").await
    }

    async fn test_get_is_success(url: &str) {
        let db = super::create_tables_and_get_connection().await;
        let app = create_app!(db);     

        let req = test::TestRequest::get()
            .uri(url)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }
}
