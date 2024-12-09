mod test_setup;
use test_setup::prelude::*;

#[cfg(test)]
mod post_delete_is_success {
    use actix_admin::prelude::*;
    use actix_web::{http::header::ContentType, test, App};
    use itertools::Itertools;
    use sea_orm::{
        sea_query::{Expr, Value},
        ColumnTrait, EntityTrait, QueryFilter,
    };

    use crate::create_app;

    #[actix_web::test]
    async fn post_delete() {
        let db = super::setup_db(true).await;
        let app = create_app!(db, false, None, false);
        let id = 1;
        let entity = super::test_setup::Post::find_by_id(id)
            .one(&db)
            .await
            .unwrap();
        assert!(entity.is_some());

        let uri = format!("/admin/post/delete/{}", id);
        let req = test::TestRequest::delete().uri(&uri).to_request();
        let resp = test::call_service(&app, req).await;

        // Delete should fail due to foreign key
        assert!(!resp.status().is_success());

        let comment_delete_res = super::test_setup::Comment::delete_by_id(id)
            .exec(&db)
            .await
            .unwrap();
        assert_eq!(comment_delete_res.rows_affected, 1);

        let uri = format!("/admin/post/delete/{}", id);
        let req = test::TestRequest::delete().uri(&uri).to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let entity_after_delete = super::test_setup::Post::find_by_id(id)
            .one(&db)
            .await
            .unwrap();
        assert!(entity_after_delete.is_none());
    }

    #[actix_web::test]
    async fn comment_delete() {
        let db = super::setup_db(true).await;
        let app = create_app!(db, false, None, false);
        let id = 1;
        let entity = super::test_setup::Comment::find_by_id(id)
            .one(&db)
            .await
            .unwrap();
        assert!(entity.is_some());

        let uri = format!("/admin/comment/delete/{}", id);
        let req = test::TestRequest::delete().uri(&uri).to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let entity_after_delete = super::test_setup::Comment::find_by_id(id)
            .one(&db)
            .await
            .unwrap();
        assert!(entity_after_delete.is_none());
    }

    #[actix_web::test]
    async fn comment_delete_many() {
        let db = super::setup_db(true).await;
        let app = create_app!(db, false, None, false);
        let ids = vec![1, 2, 3];
        for id in &ids {
            let entity = super::test_setup::Comment::find_by_id(*id)
                .one(&db)
                .await
                .unwrap();
            assert!(entity.is_some());
        }

        let payload: String = ids.iter().map(|i| format!("ids={}", i)).join("&");
        let ids_payload = payload.into_bytes();
        let req = test::TestRequest::delete()
            .uri("/admin/comment/delete")
            .insert_header(ContentType::form_url_encoded())
            .set_payload(ids_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_redirection());

        for id in ids {
            let entity_after_delete = super::test_setup::Comment::find_by_id(id)
                .one(&db)
                .await
                .unwrap();
            assert!(entity_after_delete.is_none());
        }
    }

    #[actix_web::test]
    async fn post_delete_many() {
        let db = super::setup_db(true).await;
        let app = create_app!(db, false, None, false);
        let ids = vec![1, 2, 3];
        for id in &ids {
            let entity = super::test_setup::Post::find_by_id(*id)
                .one(&db)
                .await
                .unwrap();
            assert!(entity.is_some());
        }

        let payload: String = ids.iter().map(|i| format!("ids={}", i)).join("&");
        let ids_payload = payload.into_bytes();
        let req = test::TestRequest::delete()
            .uri("/admin/post/delete")
            .insert_header(ContentType::form_url_encoded())
            .set_payload(ids_payload.clone())
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Fails because of FK constraints
        assert!(resp.status().is_server_error());

        // Remove FK
        let update_res = super::test_setup::Comment::update_many()
            .col_expr(
                super::test_setup::comment::Column::PostId,
                Expr::value(Value::Int(None)),
            )
            .filter(super::test_setup::comment::Column::PostId.is_in(ids.clone()))
            .exec(&db)
            .await;
        assert!(update_res.is_ok());

        // Delete again
        let req = test::TestRequest::delete()
            .uri("/admin/post/delete")
            .insert_header(ContentType::form_url_encoded())
            .set_payload(ids_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Should not fail anymore and redirect correctly
        assert!(resp.status().is_redirection());

        for id in ids {
            let entity_after_delete = super::test_setup::Post::find_by_id(id)
                .one(&db)
                .await
                .unwrap();
            assert!(entity_after_delete.is_none());
        }
    }
}
