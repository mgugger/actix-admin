---
title: "Getting Started"
date: 2023-01-17T11:44:56+01:00
draft: false
weight: 1
---

# Getting Started

## Import actix-admin

Cargo.toml:
```cargo
[dependencies]
actix-admin = "0.3.0"
```

## Implement the Trait for AppState

Actix-Admin requires to get the database connection and its configuration from the actix AppState. The trait "ActixAdminAppDataTrait" must be implemented for your AppState:

```rust
use actix_admin::prelude::*;

#[derive(Clone)]
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

## Build the Actix-Admin Configuration

The configuration can be built like the followings, which initializes a ActixAdminBuilder entity:

```rust
use actix_admin::prelude::*;

fn create_actix_admin_builder() -> ActixAdminBuilder {
    let configuration = ActixAdminConfiguration {
        enable_auth: false,
        user_is_logged_in: None,
        login_link: None,
        logout_link: None,
        file_upload_directory: "./file_uploads"
    };

    let mut admin_builder = ActixAdminBuilder::new(configuration);
    
    admin_builder
}
```

## Pass the configuration to Actix-Web

The AppState and the configuration can be passed to Actix-Web like in the following snippet. The ActixAdminBuilder creates an own */admin/* Scope which is registered as a service in the Actix-Web app.

```rust
let actix_admin_builder = create_actix_admin_builder();

let app_state = AppState {
    db: conn.clone(),
    actix_admin: actix_admin_builder.get_actix_admin(),
};

let app = App::new()
    .app_data(web::Data::new(app_state))
    .service(
        actix_admin_builder.get_scope::<AppState>()
    )
    .wrap(middleware::Logger::default())
```

## Complete Example

The above steps will initialize an empty ActixAdmin interface under */admin/*. For a complete example please the following link: 

[https://github.com/mgugger/actix-admin/tree/main/examples/basic](https://github.com/mgugger/actix-admin/tree/main/examples/basic)