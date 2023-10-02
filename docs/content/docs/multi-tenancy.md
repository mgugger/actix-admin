---
title: "Multi-Tenancy"
date: 2023-01-17T11:44:56+01:00
draft: false
weight: 7
---

# Multi-Tenancy

A column can be defined as "tenant_ref", actix-admin will then filter out entities from other tenants based on the configuration.

ActixAdmin Configuration:
```rust
let configuration = ActixAdminConfiguration {
    enable_auth: true,
    user_is_logged_in: Some(|session: &Session| -> bool { 
            let user_info = session.get::<user::UserProfile>("userProfile").unwrap();
            user_info.is_some()
    }),
    user_tenant_ref: Some(|session: &Session| -> Option<i32> { 
        return Some(1) // tenant_id = 1 in this example, None will show all rows
    })
};
```

In the sea-orm model, a column must be defined as follows:
```rust
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    #[actix_admin(primary_key)]
    pub id: i32,
    
    // Access will be filtered based on this column
    #[actix_admin(tenant_ref)]
    pub tenant_id: i32
}
```