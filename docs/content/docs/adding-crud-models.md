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