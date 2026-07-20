---
title: "Adding Models & Views"
date: 2023-01-17T11:44:56+01:00
draft: false
weight: 2
---

# Adding CRUD Models

CRUD Models can be added to the AdminInterface which will render a HTML table and the CRUD functions in the View.

## Struct Annotations

The struct for which the view is generated needs to be annotated and at least requires a primary key with the annotation ```#[actix_admin(primary_key)]```.

```rust
use actix_admin::prelude::*;

pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    #[actix_admin(primary_key)]
    pub id: i32,
    pub comment: String
}

// Custom Validation Functions
impl ActixAdminModelValidationTrait<ActiveModel> for Entity {}
// Custom Search Filters
impl ActixAdminModelFilterTrait<Entity> for Entity {}
```

## Derive Implementations

Actix-Admin relies on two traits to display models which can either be implemented automatically by Macros or manually:
* **ActixAdminViewModelTrait**: Handles how the CRUD Tables are displayed
* **ActixAdminModelTrait**: Acts as an abstraction between the internal ViewModel and the DB-specific interactions

These can be implemented manually or auto derived by using the following macros:
* DeriveActixAdmin
* DeriveActixAdminModel
* DeriveActixAdminViewModel

```rust
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize, 
    DeriveActixAdmin, 
    DeriveActixAdminModel, 
    DeriveActixAdminViewModel
)]
#[sea_orm(table_name = "comment")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    #[actix_admin(primary_key)]
    pub id: i32,
    pub comment: String
}
```

## Add the Views to Actix-Admin

Within the ActixAdminBuilder, the entity can be added as per following code and they will appear in the NavBar in the admin interface.

```rust
fn create_actix_admin_builder() -> ActixAdminBuilder {
    let mut admin_builder = ActixAdminBuilder::new(configuration);

    let comment_view_model = ActixAdminViewModel::from(Comment);
    admin_builder.add_entity::<Comment>(&comment_view_model);

    admin_builder
}
```

## Adapt the Views

```rust
let mut post_view_model = ActixAdminViewModel::from(Post);

// clicking on edit will allow editing the row within the table and not redirect to an edit view
post_view_model.inline_edit = true;
// hide the filter list which is open by default
post_view_model.default_show_aside = false;

admin_builder.add_entity::<Post>(&post_view_model);
```

## View Groups

Views / Models can be grouped in the Navbar by using the following functions instead of ```admin_builder.add_entity```:
```rust
admin_builder.add_entity::<Comment>(&comment_view_model);
admin_builder.add_entity_to_category::<Comment>(&comment_view_model, "Group 1");
```

## Additional Model Attributes

More attributes can be added to the model struct properties:
| | | |
|----|----|----|
| primary_key | required | defines which column is used for the primary key of the model |
| html_input_type=<String> | optional | add the defined value such as *email* as input type to the html input field
| select_list | optional | A dropdown is rendered for the specific entity, needs to match the name of a struct or an enum |
| searchable | optional | Adds a search field to the table allowing to search the specific column |
| textarea | optional | renders a textarea instead of a text input field
| file_upload | optional | renders a file upload field, storing the filename in the column, column must be a string |
| not_empty | optional | disallow empty strings such as "" |
| list_sort_position=<usize> | optional | orders the columns in the list view by ascending position |
| list_hide_column<bool> | optional | hides the column in the list view |
| foreign_key=<entity_name> | optional | shows the display of the foreign key entity instead of the id |
| ceil=<integer> | optional | ceils a float to the given precision |
| floor=<integer> | optional | floor a float to the given precision |
| dateformat=<String> | optional | formats a date or a datetime to the given format, does not work with NaiveDate or NaiveDateTime and requires a timezone |
| shorten=<integer> | optional | shortens a string to the given length |
| use_tom_select_callback | optional | uses a tom-select.js dropdown instead of a select which allows searching and loads data in the background when searching |
| html_render | optional | renders raw HTML in the list/show views (marks the column value as safe). Never enable on user-controlled input |
| url | optional | renders the string as a clickable link in list/show views |
| email | optional | renders the string as a `mailto:` link in list/show views |
| image | optional | treats a string column as an uploaded image filename. Shows a thumbnail on list and a preview on show/edit |
| wysiwyg | optional | renders a Markdown WYSIWYG editor (EasyMDE) on create/edit and treats the field type as RichText |
| readonly | optional | disables editing of the input in the create/edit form |

## Advanced Filters (Operators)

Custom filters can be declared with explicit comparison operators (equals,
contains, starts_with, ends_with, gt, ge, lt, le, in, not_in, is_null,
is_not_null, between). The operator picker appears in the sidebar filter
form and is preserved across pagination.

```rust
use actix_admin::prelude::*;

impl ActixAdminModelFilterTrait<Entity> for Entity {
    fn get_filter() -> Vec<ActixAdminModelFilter<Entity>> {
        vec![
            ActixAdminModelFilter::new("title", ActixAdminModelFilterType::Text)
                .with_operators(vec![
                    ActixAdminFilterOperator::Contains,
                    ActixAdminFilterOperator::StartsWith,
                    ActixAdminFilterOperator::Equals,
                ])
                .with_operator_filter(|q, op, val| match (op, val) {
                    (ActixAdminFilterOperator::StartsWith, Some(v)) =>
                        q.filter(Column::Title.starts_with(&v)),
                    (ActixAdminFilterOperator::Equals, Some(v)) =>
                        q.filter(Column::Title.eq(v)),
                    (_, Some(v)) => q.filter(Column::Title.contains(&v)),
                    _ => q,
                }),
        ]
    }
}
```

The operator selected by the user is delivered on the query string as
`filter_<name>__op=<snake_case>` and is passed to `with_operator_filter`.
If no operators are configured, the classic single-op `filter(|q, val|)`
closure is used (backwards compatible).

## Per-View Permissions

Each `ActixAdminViewModel` exposes five permission hooks that gate the
corresponding UI buttons and reject direct hits to the routes with 403.
They receive the current `Session` and default to "allow":

```rust
let mut vm = ActixAdminViewModel::from(Post);
vm.user_can_create = Some(|session| user_is_editor(session));
vm.user_can_edit   = Some(|session| user_is_editor(session));
vm.user_can_delete = Some(|session| user_is_admin(session));
vm.user_can_view_details = Some(|_| true);
vm.user_can_export = Some(|session| user_is_admin(session));
admin_builder.add_entity::<Post>(&vm);
```

When a hook returns `false`, the associated button (Create, Edit,
Delete, Export as CSV, row-level Edit/Delete icons) is hidden and any
request that reaches the corresponding route is answered with
`403 Forbidden`. Hooks are independent from the top-level
`user_is_logged_in` check, which still applies.

## Custom Bulk Actions

Beyond the built-in bulk delete, entities can register named bulk
actions that appear in the list-page actions dropdown. Register the
metadata on the builder and provide a dispatcher implementation on the
entity:

```rust
admin_builder.add_bulk_action_for_entity::<Post>(ActixAdminBulkAction {
    name: "publish".into(),
    label: "Publish selected".into(),
    icon: Some("fa-solid fa-check".into()),
    confirm: Some("Publish selected posts?".into()),
});

#[async_trait::async_trait(?Send)]
impl actix_admin::routes::ActixAdminBulkActionDispatch for post::Entity {
    async fn run_bulk_action(
        name: &str,
        db: &sea_orm::DatabaseConnection,
        ids: Vec<i32>,
    ) -> Result<String, actix_admin::ActixAdminError> {
        match name {
            "publish" => {
                // ... update rows ...
                Ok(format!("Published {} post(s).", ids.len()))
            }
            other => Err(actix_admin::ActixAdminError::UnknownBulkAction(other.into())),
        }
    }
}
```

The route `/{entity}/action/{name}` is only registered for entities that
have at least one bulk action declared, so entities that never opt in
don't need to implement the trait.

## CSRF Protection

Cross-Site Request Forgery protection is opt-in via the configuration
flag `enable_csrf` on `ActixAdminConfiguration`. When enabled:

* A per-session token is placed in the Actix session and exposed to
  templates as `{{ csrf_token }}` and as `<meta name="csrf-token">`.
* HTMX requests automatically send the token in the `X-CSRF-Token`
  header (wired from `head.html`).
* Traditional form submissions include a hidden `_csrf` input.
* All state-changing routes (`POST` create/edit, `DELETE`,
  file-delete, bulk-action, delete-many) reject requests whose
  token doesn't match with `403 Forbidden`.

A session middleware must be configured for the flag to have any effect
(actix-admin already requires one for flash notifications).
