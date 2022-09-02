# Actix Admin

The actix-admin crate aims at creating a web admin interface similar to other admin interfaces (such as [flask-admin](https://github.com/flask-admin/flask-admin) in python).

## Getting Started

* See the [example](https://github.com/mgugger/actix-admin/tree/main/example).
* See the step by [step tutorial](https://github.com/mgugger/actix-admin/tree/main/example/StepbyStep.md) 

## Features
1. Async, builds on [sea-orm](https://crates.io/crates/sea-orm) for the database backend
2. Macros, generate the required implementations for models automatically
3. Authentication, optionally pass authentication handler to implement authentication for views
4. Supports custom validation rules
5. Searchable attributes can be specified
6. Supports a custom index view

## Screenshot

<img src="https://raw.githubusercontent.com/mgugger/actix-admin/main/static/Screenshot.png"/>

## Quick overview

### Required dependencies
```
itertools = "0.10.3"
sea-orm = { version = "^0.9.1", features = [ "sqlx-sqlite", "runtime-actix-native-tls", "macros" ], default-features = true }
actix_admin = { version = "^0.1.0" }
```

### Derive Macros
```
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use actix_admin::prelude::*;
use super::Post;

// Use DeriveActixAmin* Macros to implement the traits for the model
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
```

### Add actix-admin to the AppState
```
pub struct AppState {
 pub db: DatabaseConnection,
 pub actix_admin: ActixAdmin,
}

impl ActixAdminAppDataTrait for AppState {
    fn get_db(&self) -> &DatabaseConnection {
        &self.db
    }

    fn get_actix_admin(&self) -> &ActixAdmin {
        &self.actix_admin
    }
}
```

### Actix Admin Builder
```
pub fn create_actix_admin_builder() -> ActixAdminBuilder {
let comment_view_model = ActixAdminViewModel::from(Comment);
//!
let configuration = ActixAdminConfiguration {
   enable_auth: false,
   user_is_logged_in: None,
   login_link: None,
   logout_link: None,
};
//!
let mut admin_builder = ActixAdminBuilder::new(configuration);
admin_builder.add_entity::<AppState, Comment>(&comment_view_model);
//!
admin_builder
}
```

### Add to the actix app
```
let actix_admin = create_actix_admin_builder().get_actix_admin();

let app_state = AppState {
    db: conn,
    actix_admin: actix_admin,
};

HttpServer::new(move || {
    App::new()
        .app_data(web::Data::new(app_state.clone()))
        .wrap(SessionMiddleware::new(CookieSessionStore::default(), cookie_secret_key.clone()))
        .route("/", web::get().to(index))
        .service(azure_auth.clone().create_scope::<AppState>())
        .service(
            create_actix_admin_builder().get_scope::<AppState>()
        )
        .wrap(middleware::Logger::default())
})
```

### Access
The admin interface will be available under /admin/.