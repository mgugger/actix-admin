---
title: "Custom Handlers"
date: 2023-01-17T11:44:56+01:00
draft: false
weight: 3
---

# Custom Handler

While the derived models create predefined routes and views, custom routes can be added to the admin interface and also base templates can be extended.

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
async fn custom_index<T: ActixAdminAppDataTrait + AppDataTrait>(
    session: Session,
    data: web::Data<T>
) -> Result<HttpResponse, Error> {
    
    let mut ctx = Context::new();
    ctx.extend(get_admin_ctx(session, &data));

    let body = data.get_tmpl()
    .render("custom_index.html", &ctx).unwrap();
    
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}
```

After this in the builder, pass your custom index function defined above:
```rust
let mut admin_builder = ActixAdminBuilder::new(configuration);
admin_builder.add_custom_handler_for_index::<AppState>(
    web::get().to(custom_index::<AppState>)
);
```

## Custom Handlers

Similarly to the custom index above, the builder accepts additional routes to be passed to the admin interface.

### Root Path
```rust
// This will be shown in the top level menu
let show_in_menu = true;
admin_builder.add_custom_handler("Custom Route in Menu", "/custom_route_in_menu", web::get().to       custom_index::<AppState>), show_in_menu); 
```

### Tied to a specific entity
```rust
// this will expose a menu item which links to /admin/comment/custom_handler and is shown in the NavBar menu
let show_in_menu = true;
let some_category = "Some Category";
admin_builder.add_entity_to_category::<AppState, Comment>(&comment_view_model, some_category);
admin_builder.add_custom_handler_for_entity::<AppState, Comment>(
    "My custom handler",
    "/custom_handler", 
    web::get().to(custom_handler::<AppState, Comment>),
    show_in_menu
);
```

### Added to an entity but shown grouped in a Category
```rust
// this will expose a menu item which links to /admin/comment/custom_handler and is shown in the NavBar menu in the group "Some Category"
let show_in_menu = true;
let some_category = "Some Category";
admin_builder.add_entity_to_category::<AppState, Comment>(&comment_view_model, some_category);
admin_builder.add_custom_handler_for_entity_in_category::<AppState, Comment>(
    "My custom handler",
    "/custom_handler", 
    web::get().to(custom_handler::<AppState, Comment>),
    some_category,
    show_in_menu
);
```
