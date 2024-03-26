mod test_setup;
use test_setup::prelude::*;

#[cfg(test)]
mod post_create_and_edit_is_success {
    use actix_admin::prelude::*;
    use actix_web::{
        test, 
        App, 
        http::header::ContentType
    };
    use chrono::{ NaiveDateTime, NaiveDate };
    use serde::{Serialize};
    use sea_orm::{ PaginatorTrait, EntityTrait, prelude::Decimal};
    
    use crate::{create_app};

    #[derive(Serialize, Clone)]
    pub struct CommentModel {
        id: &'static str,
        insert_date: &'static str,
        comment: &'static str,
        user: &'static str,
        is_visible: &'static str,
        post_id: Option<&'static str>,
        my_decimal: &'static str
    }

    #[derive(Serialize, Clone)]
    pub struct PostModel {
        id: &'static str,
        title: &'static str,
        text: &'static str,
        tea_mandatory: &'static str,
        insert_date: &'static str,
    }

    #[actix_web::test]
    async fn comment_create_and_edit() {
        let db = super::setup_db(false).await;
        let app = create_app!(db, false, None);

        // create entity
        let mut model = CommentModel {
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
            .set_form(model.clone())
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_redirection());

        let entities = super::test_setup::Comment::find()
            .paginate(&db, 50)
            .fetch_page(0)
            .await
            .expect("could not retrieve entities");

        assert_eq!(entities.len(), 1, "After post, db does not contain 1 model");
        let entity = entities.first().unwrap();
        assert_eq!(entity.id, 1);
        assert_eq!(entity.comment,"test");
        assert_eq!(entity.user, "test");
        assert!(entity.is_visible);
        assert!(entity.post_id.is_none());
        assert_eq!(entity.my_decimal, Decimal::new(113141, 3));
        assert_eq!(entity.insert_date, NaiveDateTime::parse_from_str("1977-04-01T14:00", "%Y-%m-%dT%H:%M").unwrap());

        // update entity
        model.my_decimal = "213.141";
        model.user = "updated";
        model.comment = "updated";
        model.insert_date = "1987-04-01T14:00";
        model.is_visible = "false";

        let edit_req = test::TestRequest::post()
            .insert_header(ContentType::form_url_encoded())
            .uri("/admin/comment/edit_post_from_plaintext/1")
            .set_form(model.clone())
            .to_request();
        let resp = test::call_service(&app, edit_req).await;

        assert!(resp.status().is_redirection());

        let entities = super::test_setup::Comment::find()
            .paginate(&db, 50)
            .fetch_page(0)
            .await
            .expect("could not retrieve entities");

        assert_eq!(entities.len(), 1, "After edit post, db does not contain 1 model");
        let entity = entities.first().unwrap();
        assert_eq!(entity.id, 1);
        assert_eq!(entity.comment, "updated");
        assert_eq!(entity.user, "updated");
        assert!(!entity.is_visible);
        assert!(entity.post_id.is_none());
        assert_eq!(entity.my_decimal, Decimal::new(213141, 3));
        assert_eq!(entity.insert_date, NaiveDateTime::parse_from_str("1987-04-01T14:00", "%Y-%m-%dT%H:%M").unwrap());
    }
    
    #[actix_web::test]
    async fn post_create_and_edit() {
        let db = super::setup_db(false).await;
        let app = create_app!(db, false, None);

        let mut model = PostModel {
            id: "0",
            insert_date: "1977-04-01",
            title: "test",
            text: "test",
            tea_mandatory: "EverydayTea"
        };

        let req = test::TestRequest::post()
            .insert_header(ContentType::form_url_encoded())
            .uri("/admin/post/create_post_from_plaintext")
            .set_form(model.clone())
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_redirection());

        let entities = super::test_setup::Post::find()
            .paginate(&db, 50)
            .fetch_page(0)
            .await
            .expect("could not retrieve entities");

        assert_eq!(entities.len(), 1, "After post, db does not contain 1 model");
        let entity = entities.first().unwrap();
        assert_eq!(entity.id, 1);
        assert_eq!(entity.tea_mandatory, super::test_setup::post::Tea::EverydayTea);
        assert_eq!(entity.title, model.title);
        assert_eq!(entity.text, model.text);
        assert_eq!(entity.insert_date, NaiveDate::parse_from_str("1977-04-01", "%Y-%m-%d").unwrap());

        // update entity
        model.tea_mandatory = "BreakfastTea";
        model.title = "updated";
        model.text = "updated";
        model.insert_date = "1987-04-01";

        let edit_req = test::TestRequest::post()
            .insert_header(ContentType::form_url_encoded())
            .uri("/admin/post/edit_post_from_plaintext/1")
            .set_form(model.clone())
            .to_request();
        let resp = test::call_service(&app, edit_req).await;

        assert!(resp.status().is_redirection());

        let entities = super::test_setup::Post::find()
            .paginate(&db, 50)
            .fetch_page(0)
            .await
            .expect("could not retrieve entities");

            assert_eq!(entities.len(), 1, "After edit post, db does not contain 1 model");
        let entity = entities.first().unwrap();
        assert_eq!(entity.id, 1);
        assert_eq!(entity.text, "updated");
        assert_eq!(entity.title, "updated");
        assert_eq!(entity.insert_date, NaiveDate::parse_from_str("1987-04-01", "%Y-%m-%d").unwrap());
    }
}
