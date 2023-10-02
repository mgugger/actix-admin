mod test_setup;
use test_setup::prelude::*;

#[cfg(test)]
mod verify_tenant_ref {
    extern crate serde_derive;
    use super::create_app;
    use super::BodyTest;
    use actix_admin::prelude::*;
    use actix_web::body::to_bytes;
    use actix_web::http::header::ContentType;
    use actix_web::test;
    use actix_web::App;
    use sea_orm::DatabaseConnection;
    use sea_orm::EntityTrait;
    use sea_orm::PaginatorTrait;
    use serde_derive::Serialize;

    // This test should only return entities that belong to tenant 1
    fn tenant_ref_fn(_session: &Session) -> Option<i32> {
        Some(1)
    }

    #[actix_web::test]
    async fn get_sample_list_page() {
        let db = super::setup_db(true).await;
        let page = 5;
        let page_size = 50; // Verify with default size in list.rs
        let url = format!(
            "/admin/{}/list?page={}&entities_per_page={}",
            crate::SampleWithTenantId::get_entity_name(),
            page,
            page_size
        );

        test_response_contains(url.as_str(), &db, vec!["TestTenant1".to_string()], true).await;
        test_response_contains(url.as_str(), &db, vec!["TestTenant0".to_string()], false).await;
    }

    #[actix_web::test]
    async fn get_sample_show_page() {
        let db = super::setup_db(true).await;
        for i in 1..20 {
            let url = format!(
                "/admin/{}/show/{}",
                crate::SampleWithTenantId::get_entity_name(),
                i
            );
            if i % 2 == 1 {
                test_response_contains(url.as_str(), &db, vec!["TestTenant1".to_string()], true).await;
            } else {
                test_response_contains(url.as_str(), &db, vec!["TestTenant".to_string()], false).await;
            }
            // ensure that no entities from tenant with id 0 are returned
            test_response_contains(url.as_str(), &db, vec!["TestTenant0".to_string()], false).await;
        }
    }

    async fn test_response_contains(
        url: &str,
        db: &DatabaseConnection,
        elements_to_verify: Vec<String>,
        should_contain: bool,
    ) {
        let app = create_app!(db, false, Some(tenant_ref_fn));

        let req = test::TestRequest::get().uri(url).to_request();

        let resp = test::call_service(&app, req).await;
        let body = to_bytes(resp.into_body()).await.unwrap();
        let body = body.as_str();

        for element in elements_to_verify {
            let contains_element = body.contains(&element);

            if should_contain {
                assert!(
                    contains_element,
                    "Body did not contain element {}: \n{}",
                    element, body
                );
            } else {
                assert!(
                    !contains_element,
                    "Body contained element {}: \n{}",
                    element, body
                );
            }
        }
    }

    #[actix_web::test]
    async fn sample_with_tenant_id_delete_own_tenant() {
        let db = super::setup_db(true).await;
        let app = create_app!(db, false, Some(tenant_ref_fn));
        let id = 1;
        let entity = super::SampleWithTenantId::find_by_id(id)
            .one(&db)
            .await
            .unwrap();
        assert!(entity.is_some() && entity.unwrap().tenant_id == 1);

        let uri = format!("/admin/{}/delete/{}", super::SampleWithTenantId::get_entity_name(), id);
        let req = test::TestRequest::delete().uri(&uri).to_request();
        let resp = test::call_service(&app, req).await;

        // Delete should fail due to wrong tenant
        assert!(resp.status().is_success());

        let entity_after_delete = super::SampleWithTenantId::find_by_id(id)
            .one(&db)
            .await
            .unwrap();
        assert!(entity_after_delete.is_none());
    }

    #[actix_web::test]
    async fn sample_with_tenant_id_delete_other_tenant() {
        let db = super::setup_db(true).await;
        let app = create_app!(db, false, Some(tenant_ref_fn));
        let id = 2;
        let entity = super::SampleWithTenantId::find_by_id(id)
            .one(&db)
            .await
            .unwrap();
        assert!(entity.is_some() && entity.unwrap().tenant_id == 0);

        let uri = format!("/admin/{}/delete/{}", super::SampleWithTenantId::get_entity_name(), id);
        let req = test::TestRequest::delete().uri(&uri).to_request();
        let resp = test::call_service(&app, req).await;

        // Delete should fail due to wrong tenant
        assert!(!resp.status().is_success());

        let entity_after_delete = super::SampleWithTenantId::find_by_id(id)
            .one(&db)
            .await
            .unwrap();
        assert!(entity_after_delete.is_some());
    }

    #[derive(Serialize, Clone)]
    pub struct SampleWithTenantIdModel {
        id: &'static str,
        title: &'static str,
        text: &'static str
    }

    #[actix_web::test]
    async fn sample_with_tenant_id_create() {
        let db = super::setup_db(false).await;
        let app = create_app!(db, false, Some(tenant_ref_fn));

        let model = SampleWithTenantIdModel {
            id: "0",
            title: "test",
            text: "test"
        };

        let req = test::TestRequest::post()
            .insert_header(ContentType::form_url_encoded())
            .uri(format!("/admin/{}/create_post_from_plaintext", super::SampleWithTenantId::get_entity_name()).as_ref())
            .set_form(model.clone())
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_redirection());

        let entities = super::SampleWithTenantId::find()
            .paginate(&db, 50)
            .fetch_page(0)
            .await
            .expect("could not retrieve entities");

        assert_eq!(entities.len(), 1, "After create, db does not contain 1 model");
        let entity = entities.first().unwrap();
        assert_eq!(entity.tenant_id, 1, "After create, entity does not have the correct tenant id assigned");
    }
}
