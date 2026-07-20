//! Integration tests for the "Actions & Access" release features:
//!
//! * new field-type renderers (Html / Url / Email / Image / RichText / readonly)
//! * per-view permission hooks (create / edit / delete / view / export)
//! * custom bulk actions (both metadata rendering and dispatch)
//! * advanced filter operators (`filter_<name>__op=<op>`)
//! * CSRF protection on state-changing routes
//!
//! These tests deliberately construct their own `ActixAdminBuilder` (rather
//! than reusing `test_setup::helper::create_actix_admin_builder`) so that
//! they can flip `enable_csrf`, install permission hooks and register a
//! bulk action without disturbing every other test.

mod test_setup;

use actix_admin::prelude::*;
use actix_admin::routes::ActixAdminBulkActionDispatch;
use actix_session::config::CookieContentSecurity;
use actix_session::storage::CookieSessionStore;
use actix_session::SessionMiddleware;
use actix_web::body::to_bytes;
use actix_web::cookie::Key;
use actix_web::{test, web, App};
use sea_orm::DatabaseConnection;

use test_setup::prelude::*;
use test_setup::{Comment, Post};

// --- Custom bulk-action dispatcher for the test-side Post ---------------

// Wire up a dispatcher for the test-side Post entity so
// `add_bulk_action_for_entity::<Post>` has something to call. It records
// the invocation by returning a message containing the number of ids.
#[actix_admin::prelude::async_trait(?Send)]
impl ActixAdminBulkActionDispatch for Post {
    async fn run_bulk_action(
        name: &str,
        _db: &sea_orm::DatabaseConnection,
        ids: Vec<Self::Id>,
        _tenant_ref: Option<i32>,
    ) -> Result<Option<String>, ActixAdminError> {
        match name {
            "mark_reviewed" => Ok(Some(format!("marked {} post(s)", ids.len()))),
            _ => Ok(None),
        }
    }
}

// --- App / builder factory ---------------------------------------------

fn build_admin(enable_csrf: bool, restrict_perms: bool) -> ActixAdminBuilder {
    let configuration = ActixAdminConfiguration {
        enable_auth: false,
        user_tenant_ref: None,
        user_is_logged_in: None,
        login_link: None,
        logout_link: None,
        file_upload_directory: "./file_uploads",
        navbar_title: "test",
        base_path: "/admin",
        custom_css_paths: None,
        custom_js_paths: None,
        enable_csrf,
    };

    let mut post_view_model = ActixAdminViewModel::from(Post);
    if restrict_perms {
        // Deny create/delete/export; allow list/view/edit. The buttons for
        // the denied actions must disappear from the list HTML and direct
        // POST/DELETE/GET hits must be rejected with 403.
        post_view_model.user_can_create = Some(|_s: &Session| false);
        post_view_model.user_can_delete = Some(|_s: &Session| false);
        post_view_model.user_can_export = Some(|_s: &Session| false);
    }

    let mut builder = ActixAdminBuilder::new(configuration);
    builder.add_entity::<Post>(&post_view_model);
    let comment_view_model = ActixAdminViewModel::from(Comment);
    builder.add_entity::<Comment>(&comment_view_model);
    builder.add_bulk_action_for_entity::<Post>(ActixAdminBulkAction {
        name: "mark_reviewed".into(),
        label: "Mark selected as reviewed".into(),
        icon: Some("fa-solid fa-check".into()),
        confirm: Some("Confirm?".into()),
    });
    builder
}

/// Init a service with a session middleware (required for CSRF/flash).
/// Returns the initialised `Service` and the underlying db.
macro_rules! init_app {
    ($db:expr, $enable_csrf:expr, $restrict_perms:expr) => {{
        let conn = $db.clone();
        let builder = build_admin($enable_csrf, $restrict_perms);
        let actix_admin = builder.get_actix_admin();
        // Deterministic 64-byte key so cookies survive across requests
        // in the same test run.
        let key = Key::from(&[0u8; 64]);
        test::init_service(
            App::new()
                .wrap(
                    SessionMiddleware::builder(CookieSessionStore::default(), key)
                        .cookie_secure(false)
                        .cookie_content_security(CookieContentSecurity::Private)
                        .build(),
                )
                .app_data(web::Data::new(actix_admin))
                .app_data(web::Data::new(conn))
                .service(builder.get_scope()),
        )
        .await
    }};
}

async fn body_utf8(resp: actix_web::dev::ServiceResponse) -> String {
    let body = to_bytes(resp.into_body()).await.unwrap();
    String::from_utf8_lossy(&body).into_owned()
}

// ------------------------------------------------------------------
// 1. Field-type renderers on the list page
// ------------------------------------------------------------------

#[actix_web::test]
async fn list_renders_new_field_types() {
    let db: DatabaseConnection = setup_db(true).await;
    let app = init_app!(&db, false, false);

    let req = test::TestRequest::get().uri("/admin/post/list").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status: {}", resp.status());
    let body = body_utf8(resp).await;

    // Html render — the raw <em> tag from the fixture must reach the page unescaped.
    assert!(
        body.contains("<em>row-1</em>"),
        "Html renderer did not emit the raw <em> tag"
    );
    // Url renderer — anchor with target=_blank.
    assert!(
        body.contains("https://example.com/1") && body.contains("target=\"_blank\""),
        "Url renderer did not build the <a target=_blank> link"
    );
    // Email renderer — mailto scheme.
    assert!(
        body.contains("mailto:row1@example.com"),
        "Email renderer did not build the mailto: link"
    );
    // Image renderer — <img src=..file/{id}/cover_image..>
    assert!(
        body.contains("/file/") && body.contains("cover_image"),
        "Image renderer did not build the thumbnail <img> tag"
    );
}

// ------------------------------------------------------------------
// 2. Show page renders new field types (readonly / rich-text / image large preview)
// ------------------------------------------------------------------

#[actix_web::test]
async fn show_renders_new_field_types_and_readonly() {
    let db = setup_db(true).await;
    let app = init_app!(&db, false, false);

    let req = test::TestRequest::get().uri("/admin/post/show/3").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = body_utf8(resp).await;

    // Cover image row 3 was populated with placeholder.png; the show page
    // renders a full-size preview.
    assert!(
        body.contains("placeholder.png") && body.contains("<img"),
        "Show page did not render the image preview"
    );
    // The `readonly` external_id is emitted verbatim.
    assert!(body.contains("EXT-00003"), "readonly value missing from show");

    // Edit page: the readonly attribute must be present on the input
    // and EasyMDE textarea id must appear for the notes_md wysiwyg column.
    let req = test::TestRequest::get().uri("/admin/post/edit/1").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = body_utf8(resp).await;
    assert!(
        body.contains("aa-wysiwyg-notes_md"),
        "wysiwyg column did not emit its EasyMDE hook id"
    );
    assert!(
        body.contains("readonly"),
        "readonly attribute not emitted on the create/edit form"
    );
}

// ------------------------------------------------------------------
// 3. Per-view permissions: buttons disappear + direct hits get 403
// ------------------------------------------------------------------

#[actix_web::test]
async fn permissions_hide_buttons_and_reject_direct_hits() {
    let db = setup_db(true).await;
    let app = init_app!(&db, false, /* restrict_perms */ true);

    // Buttons in the list HTML must be gone.
    let req = test::TestRequest::get().uri("/admin/post/list").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = body_utf8(resp).await;
    assert!(
        !body.contains("/admin/post/create"),
        "Create button leaked to HTML even though can_create=false"
    );
    assert!(
        !body.contains("Export as CSV"),
        "Export button leaked to HTML even though can_export=false"
    );

    // Direct hits are 403.
    let req = test::TestRequest::get().uri("/admin/post/create").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 403, "GET /create should be 403 when can_create=false");

    let req = test::TestRequest::get()
        .uri("/admin/post/export_csv")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 403, "GET /export_csv should be 403 when can_export=false");

    let req = test::TestRequest::delete()
        .uri("/admin/post/delete/1")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 403, "DELETE /delete/{{id}} should be 403 when can_delete=false");
}

// ------------------------------------------------------------------
// 4. Custom bulk actions
// ------------------------------------------------------------------

#[actix_web::test]
async fn bulk_action_dropdown_and_dispatch() {
    let db = setup_db(true).await;
    let app = init_app!(&db, false, false);

    // The list page must expose the action entry (label + POST url).
    let req = test::TestRequest::get().uri("/admin/post/list").to_request();
    let resp = test::call_service(&app, req).await;
    let body = body_utf8(resp).await;
    assert!(
        body.contains("Mark selected as reviewed"),
        "bulk-action label missing from list dropdown"
    );
    assert!(
        body.contains("action/mark_reviewed"),
        "bulk-action POST url missing from list dropdown"
    );

    // Dispatching a known action returns 303 (see-other back to /list).
    let req = test::TestRequest::post()
        .uri("/admin/post/action/mark_reviewed")
        .set_form(&[("ids", "1"), ("ids", "2"), ("ids", "3")])
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(
        resp.status().is_redirection(),
        "known bulk action must redirect to /list, got {}",
        resp.status()
    );

    // Dispatching an unknown action returns 404 (the {entity}/action/{name}
    // route only knows about names that were registered on the builder).
    let req = test::TestRequest::post()
        .uri("/admin/post/action/does_not_exist")
        .set_form(&[("ids", "1")])
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        404,
        "unknown bulk action must be 404, got {}",
        resp.status()
    );
}

// ------------------------------------------------------------------
// 5. Advanced filter operators — the operator picker is rendered and
//    the `filter_<name>__op=` param is accepted by the list route.
// ------------------------------------------------------------------

#[actix_web::test]
async fn filter_operator_picker_is_rendered_and_accepted() {
    let db = setup_db(true).await;
    let app = init_app!(&db, false, false);

    // (a) The list page renders the operator picker for the "User"
    //     filter declared on Comment (which advertises Contains / Equals /
    //     NotEquals / IsNull).
    let req = test::TestRequest::get()
        .uri("/admin/comment/list")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = body_utf8(resp).await;
    assert!(
        body.contains("filter_User__op"),
        "operator <select name='filter_User__op'> missing from filter form"
    );
    // The snake_case wire values (contains, equals, not_equals, is_null)
    // must be emitted as <option value=..>.
    for op in ["contains", "equals", "not_equals", "is_null"] {
        assert!(
            body.contains(&format!("value=\"{}\"", op)),
            "operator option {op} missing from filter form"
        );
    }

    // (b) The list route accepts and does not choke on an operator query
    //     string. We just assert 200: the semantics of the filter itself
    //     are exercised by the doc / macro.
    let req = test::TestRequest::get()
        .uri("/admin/comment/list?filter_User=me@home.com&filter_User__op=contains")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(
        resp.status().is_success(),
        "list route rejected operator query string: {}",
        resp.status()
    );

    // (c) An unknown operator name must not 500 the route.
    let req = test::TestRequest::get()
        .uri("/admin/comment/list?filter_User=x&filter_User__op=bogus")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(
        resp.status().is_success(),
        "unknown operator crashed the list route: {}",
        resp.status()
    );
}

// ------------------------------------------------------------------
// 6. CSRF protection
// ------------------------------------------------------------------

#[actix_web::test]
async fn csrf_rejects_state_changes_without_token() {
    let db = setup_db(true).await;
    let app = init_app!(&db, /* enable_csrf */ true, false);

    // GET list is safe and must render + set a session cookie carrying
    // the freshly minted CSRF token.
    let req = test::TestRequest::get().uri("/admin/post/list").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // DELETE without token → 403.
    let req = test::TestRequest::delete()
        .uri("/admin/post/delete/1")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        403,
        "delete without CSRF token must be 403, got {}",
        resp.status()
    );

    // POST bulk action without token → 403.
    let req = test::TestRequest::post()
        .uri("/admin/post/action/mark_reviewed")
        .set_form(&[("ids", "1")])
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        403,
        "bulk action without CSRF token must be 403, got {}",
        resp.status()
    );

    // With CSRF disabled, the same DELETE goes through (returns
    // redirect or 200). This guards against a regression where CSRF
    // is always on.
    let db2 = setup_db(true).await;
    let app2 = init_app!(&db2, /* enable_csrf */ false, false);
    let req = test::TestRequest::delete()
        .uri("/admin/post/delete/1")
        .to_request();
    let resp = test::call_service(&app2, req).await;
    assert!(
        !matches!(resp.status().as_u16(), 401 | 403),
        "delete with CSRF disabled unexpectedly rejected: {}",
        resp.status()
    );
}

// ------------------------------------------------------------------
// 7. When CSRF is enabled, the token is exposed to templates.
// ------------------------------------------------------------------

#[actix_web::test]
async fn csrf_token_is_exposed_in_rendered_page() {
    let db = setup_db(true).await;
    let app = init_app!(&db, true, false);

    let req = test::TestRequest::get().uri("/admin/post/list").to_request();
    let resp = test::call_service(&app, req).await;
    let body = body_utf8(resp).await;
    assert!(
        body.contains("csrf-token"),
        "expected <meta name=\"csrf-token\"> in rendered page when enable_csrf=true"
    );
    assert!(
        body.contains("X-CSRF-Token"),
        "expected the HTMX csrf header hook to be rendered"
    );
}
