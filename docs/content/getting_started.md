---
layout: default
---

## Getting Started

* See the [basic example](https://github.com/mgugger/actix-admin/tree/main/examples/basic) and run with ```cargo run```.

## Quick overview

### Required dependencies in Cargo.toml
```
sea-orm = { version = "^0.9.1", features = [ "sqlx-sqlite", "runtime-actix-native-tls", "macros" ], default-features = true }
actix_admin = { version = "^0.2.0" }
```

### Steps
1. Import ActixAdmin in the main.rs and your database models:
```rust
use actix_admin::prelude::*;
```

2. Use the DeriveActixAdminMacros on the Database models to implement required traits:
```rust
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

3. Add ActixAdmin to the actix admin app state
```rust
#[derive(Clone)]
    pub struct AppState {
    pub db: DatabaseConnection,
    pub actix_admin: ActixAdmin,
}
```

4. Implement the ActixAdminAppDataTrait for the AppState
```rust
impl ActixAdminAppDataTrait for AppState {
    fn get_db(&self) -> &DatabaseConnection {
        &self.db
    }

    fn get_actix_admin(&self) -> &ActixAdmin {
        &self.actix_admin
    }
}
```

5. Setup the actix admin configuration and add database models to it in main.rs
```rust
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
```

6. Add to the actix app in main.rs
```rust
let opt = ConnectOptions::new("sqlite::memory:".to_owned());
let conn = sea_orm::Database::connect(opt).unwrap();

HttpServer::new(move || {
    let actix_admin_builder = create_actix_admin_builder();

    let app_state = AppState {
        db: conn.clone(),
        actix_admin: actix_admin_builder.get_actix_admin(),
    };

    App::new()
        .app_data(web::Data::new(app_state.clone()))
        .service(
            actix_admin_builder.get_scope::<AppState>()
        )
});
```

## Access
The admin interface will be available under /admin/.