---
title: "Custom Handlers"
date: 2023-01-17T11:44:56+01:00
draft: false
weight: 3
---

# Custom Handler

While the derived models create predefined routes and views, custom routes can be added to the admin interface and also base templates can be extended.

## Extend tera instances with base templates from actix-admin

```rust
let mut tera = Tera::parse("templates/**/*.html").unwrap();
tera.extend(&actix_admin_builder.get_actix_admin().tera).unwrap();
let _tera_res = tera.build_inheritance_chains();        
```

## Custom Index

A custom *custom_index.html* view can be defined as follows by extending the base template:
```html
{% extends "base.html" %}
{% block content %}
<p>This is a custom index page shown under /admin/ extending the base template<p>
{% endblock content %}
```

To display the *custom_index.html*, define the corresponding function which extends the current tera context from actix-admin and also uses the actix-admin tera instance to render the custom index function.

```rust
use actix_admin::prelude::*;

async fn custom_index(
    session: Session,
    tera: web::Data<Tera>,
    actix_admin: web::Data<ActixAdmin>
) -> Result<HttpResponse, Error> {
    
    let mut ctx = get_admin_ctx(session, &actix_admin);
    ctx.insert("your_own_key", "your_own_value");

    let body = tera.render("custom_index.html", &ctx).unwrap();
    
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}
```

After this in the builder, pass your custom index function defined above:
```rust
let mut admin_builder = ActixAdminBuilder::new(configuration);
admin_builder.add_custom_handler_for_index(
    web::get().to(custom_index)
);
```

## Custom Handlers

Similarly to the custom index above, the builder accepts additional routes to be passed to the admin interface.

### Root Path
```rust
// This will be shown in the top level menu
let show_in_menu = true;
admin_builder.add_custom_handler("Custom Route in Menu", "/custom_route_in_menu", web::get().to(custom_index), show_in_menu); 
```

### Tied to a specific entity
```rust
// this will expose a menu item which links to /admin/comment/custom_handler and is shown in the NavBar menu
let show_in_menu = true;
let some_category = "Some Category";
admin_builder.add_entity_to_category::<Comment>(&comment_view_model, some_category);
admin_builder.add_custom_handler_for_entity::<Comment>(
    "My custom handler",
    "/custom_handler", 
    web::get().to(custom_handler::<Comment>),
    show_in_menu
);
```

> **_NOTE:_**  the category "navbar-end" is used for the dropdown containing the sign out button and will show only when auth is enabled.

### Added to an entity but shown grouped in a Category
```rust
// this will expose a menu item which links to /admin/comment/custom_handler and is shown in the NavBar menu in the group "Some Category"
let show_in_menu = true;
let some_category = "Some Category";
admin_builder.add_entity_to_category::<Comment>(&comment_view_model, some_category);
admin_builder.add_custom_handler_for_entity_in_category::<Comment>(
    "My custom handler",
    "/custom_handler", 
    web::get().to(custom_handler::<Comment>),
    some_category,
    show_in_menu
);
```

