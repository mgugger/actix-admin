---
layout: default
---

## Getting Started

* See the [example](https://github.com/mgugger/actix-admin/tree/main/example) and run with ```cargo run```.
* See the step by [step tutorial](https://github.com/mgugger/actix-admin/tree/main/example/StepbyStep.md) 

## Quick overview

### Required dependencies in Cargo.toml
```
sea-orm = { version = "^0.9.1", features = [ "sqlx-sqlite", "runtime-actix-native-tls", "macros" ], default-features = true }
actix_admin = { version = "^0.2.0" }
```

### See inlined steps
```rust
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use actix_admin::prelude::*;
use actix_web::web;
use actix_web::App;
use actix_web::HttpServer;
use sea_orm::entity::prelude::*;
use sea_orm::entity::prelude::*;
use actix_admin::prelude::*;
// 1. Import ActixAdmin
use actix_admin::prelude::*;

// 2. Use DeriveActixAmin* Macros to implement the traits for the model
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize, 
    DeriveActixAdmin, DeriveActixAdminModel, DeriveActixAdminViewModel
)]
#[sea_orm(table_name = "comment")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    #[actix_admin(primary_key)]
    pub id: i32,
    pub comment: String
}
impl ActixAdminModelValidationTrait<ActiveModel> for Entity {}
impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation { }

// 3. Add actix-admin to the AppState
#[derive(Clone)]
    pub struct AppState {
    pub db: DatabaseConnection,
    pub actix_admin: ActixAdmin,
}

// 4. Implement the ActixAdminAppDataTrait for the AppState
impl ActixAdminAppDataTrait for AppState {
    fn get_db(&self) -> &DatabaseConnection {
        &self.db
    }

    fn get_actix_admin(&self) -> &ActixAdmin {
        &self.actix_admin
    }
}

// 5. Setup the actix admin configuration
pub fn create_actix_admin_builder() -> ActixAdminBuilder {
    let comment_view_model = ActixAdminViewModel::from(Entity);

    let configuration = ActixAdminConfiguration {
    enable_auth: false,
    user_is_logged_in: None,
    login_link: None,
    logout_link: None,
    };

    let mut admin_builder = ActixAdminBuilder::new(configuration);
    admin_builder.add_entity::<AppState, Entity>(&comment_view_model);

    admin_builder
}

// 6. Add to the actix app
let actix_admin = create_actix_admin_builder().get_actix_admin();
let opt = ConnectOptions::new("sqlite::memory:".to_owned());
let conn = sea_orm::Database::connect(opt).unwrap();
let app_state = AppState {
    db: conn,
    actix_admin: actix_admin,
};

HttpServer::new(move || {
    App::new()
        //.app_data(web::Data::new(app_state.clone()))
        .service(
            create_actix_admin_builder().get_scope::<AppState>()
        )
});
```

## Access
The admin interface will be available under /admin/.