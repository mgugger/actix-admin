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
actix-admin = "0.8.0"
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
let conn = sea_orm::Database::connect(opt).await.unwrap();
let actix_admin_builder = create_actix_admin_builder();

let app = App::new()
    .app_data(web::Data::new(app_state))
    .app_data(web::Data::new(conn.clone()))
    .app_data(web::Data::new(actix_admin_builder.get_actix_admin()))
    .service(
        actix_admin_builder.get_scope()
    )
    .wrap(middleware::Logger::default())
```

## Complete Example

The above steps will initialize an empty ActixAdmin interface under */admin/*. For a complete example please the following link: 

[https://github.com/mgugger/actix-admin/tree/main/examples/basic](https://github.com/mgugger/actix-admin/tree/main/examples/basic)