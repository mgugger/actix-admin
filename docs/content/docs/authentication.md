---
title: "Authentication & Authorization"
date: 2023-01-17T11:44:56+01:00
draft: false
weight: 4
---

# Authentication & Authorization

The admin interface can optionally enable authentication and authorization, altough the authorization logic needs to happen outside of the admin interface.

An example for authenticating against Azure Active Directory can be found in the [examples](https://github.com/mgugger/actix-admin/tree/main/examples).

## Enabling Authentication

When creating the configuration, auth can be enabled as in the following code:

```rust
let configuration = ActixAdminConfiguration {
    enable_auth: true,
    user_is_logged_in: Some(|session: &Session| -> bool { 
            let user_info = session.get::<UserInfo>("user_info").unwrap();
            user_info.is_some()
    }),
    login_link: Some("/azure-auth/login".to_string()),
    logout_link: Some("/azure-auth/logout".to_string()),
};
```

The configuration expects a function taking a session parameter to return a bool whether the user is logged or not. Additionally, the login or logout links should be provided to redirect the user to the login url of choice.