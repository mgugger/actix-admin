mod test_setup;
use test_setup::prelude::*;

#[cfg(test)]
mod get_request_is_success {
    extern crate serde_derive;
    use actix_admin::prelude::*;
    use actix_web::body::to_bytes;
    use actix_web::test;
    use actix_web::App;
    use sea_orm::DatabaseConnection;
    use sea_orm::EntityTrait;
    use sea_orm::PaginatorTrait;
    use sea_orm::QueryOrder;
    use super::create_app;
    use super::BodyTest;

    #[actix_web::test]
    async fn get_admin_index() {
        let db = super::setup_db(false).await;
        test_get_is_success("/admin/", &db).await
    }

    #[actix_web::test]
    async fn get_post_list() {
        let db = super::setup_db(true).await;
        let url = format!("/admin/{}/list", crate::Post::get_entity_name());
        test_get_is_success(url.as_str(), &db).await
    }

    #[actix_web::test]
    async fn get_post_list_page() {
        let db = super::setup_db(true).await;
        let page = 5;
        let page_size = 50; // Verify with default size in list.rs
        let url = format!("/admin/{}/list?page={}&entities_per_page={}", crate::Post::get_entity_name(), page, page_size);

        let entities =  crate::Post::find()
            .order_by_asc(crate::post::Column::Id)
            .paginate(&db, page_size)
            .fetch_page(page-1)
            .await
            .unwrap();

        let verify_titles = entities.iter().map(|e| e.title.to_string()).collect();
        test_response_contains(url.as_str(), &db, verify_titles).await
    }

    #[actix_web::test]
    async fn get_post_list_search() {
        let db = super::setup_db(true).await;
        let url = format!("/admin/{}/list?search=Test%20155", crate::Post::get_entity_name());

        test_response_contains(url.as_str(), &db, vec!("Test 155".to_string())).await
    }

    #[actix_web::test]
    async fn get_post_csv_export() {
        let db = super::setup_db(true).await;
        let url = format!("/admin/{}/export_csv", crate::Post::get_entity_name());

        test_response_contains(url.as_str(), &db, vec!("Test 155".to_string(), "some content".to_string(), "EverydayTea".to_string())).await
    }

    #[actix_web::test]
    async fn get_comment_list_search() {
        let db = super::setup_db(true).await;
        let search_string_encoded = "Test%2015";
        let entities_per_page = 11;
        let url = format!("/admin/{}/list?search={}&entities_per_page={}", crate::Comment::get_entity_name(), search_string_encoded, entities_per_page);

        let mut elements_to_verify = Vec::new();
        elements_to_verify.push("Test 15".to_string());
        for i in 150..159 {
            elements_to_verify.push(format!("Test {}", i));
        }

        test_response_contains(url.as_str(), &db, elements_to_verify).await
    }

    #[actix_web::test]
    async fn get_comment_list_page() {
        let db = super::setup_db(true).await;
        let page = 17;

        let page_size = 20; // Verify with default size in list.rs
        let url = format!("/admin/{}/list?page={}&entities_per_page={}", crate::Comment::get_entity_name(), page, page_size);

        let query = if page_size == 5 {
            crate::Comment::find().order_by_asc(crate::comment::Column::Id)
        } else {
            crate::Comment::find().order_by_asc(crate::comment::Column::Id)
        };
        
        let entities = query
            .paginate(&db, page_size)
            .fetch_page(page-1)
            .await
            .unwrap();

        let verify_comments = entities.iter().map(|e| e.comment.to_string()).collect();
        test_response_contains(url.as_str(), &db, verify_comments).await
    }

    #[actix_web::test]
    async fn get_comment_list() {
        let db = super::setup_db(true).await;
        let url = format!("/admin/{}/list", crate::Comment::get_entity_name());
        test_get_is_success(url.as_str(), &db).await
    }

    #[actix_web::test]
    async fn get_post_create() {
        let db = super::setup_db(false).await;
        let url = format!("/admin/{}/create", crate::Post::get_entity_name());
        test_get_is_success(url.as_str(), &db).await
    }

    #[actix_web::test]
    async fn get_comment_create() {
        let db = super::setup_db(false).await;
        let url = format!("/admin/{}/create", crate::Comment::get_entity_name());
        test_get_is_success(url.as_str(), &db).await
    }

    #[actix_web::test]
    async fn get_comment_show() {
        let db = super::setup_db(true).await;

        let url = format!(
            "/admin/{}/show/{}", 
            crate::Comment::get_entity_name(),
            1
        );
        test_get_is_success(url.as_str(), &db).await
    }

    #[actix_web::test]
    async fn get_post_show() {
        let db = super::setup_db(true).await;

        let url = format!(
            "/admin/{}/show/{}", 
            crate::Comment::get_entity_name(),
            1
        );
        test_get_is_success(url.as_str(), &db).await
    }

    async fn test_response_contains(url: &str, db: &DatabaseConnection, elements_to_verify: Vec<String>) {
        let app = create_app!(db, false, None, false);     

        let req = test::TestRequest::get()
            .uri(url)
            .to_request();

        let resp = test::call_service(&app, req).await;
        let body = to_bytes(resp.into_body()).await.unwrap();
        let body = body.as_str();
        for element in elements_to_verify {
            assert!(body.contains(&element), "Body did not contain element {}: \n{}", element, body);
        }

    }

    async fn test_get_is_success(url: &str, db: &DatabaseConnection) {
        let app = create_app!(db, false, None, false);     

        let req = test::TestRequest::get()
            .uri(url)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }
}

