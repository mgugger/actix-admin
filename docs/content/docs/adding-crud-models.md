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
    admin_builder.add_entity::<AppState, Comment>(&comment_view_model);

    admin_builder
}
```

## View Groups

Views / Models can be grouped in the Navbar by using the following functions instead of ```admin_builder.add_entity```:
```rust
admin_builder.add_entity::<AppState, Comment>(&comment_view_model);
admin_builder.add_entity_to_category::<AppState, Comment>(&comment_view_model, "Group 1");
```