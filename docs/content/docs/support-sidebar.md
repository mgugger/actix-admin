---
title: "Support Sidebar"
date: 2023-01-17T11:44:56+01:00
draft: false
weight: 5
---

# Support Sidebar

You can create a support sidebar with helpful content or a chat interface. The content is loaded when opening the support sidebar through the questionmark in the navbar by hx-get.

```rust
async fn support(
    tera: web::Data<Tera>
) -> Result<HttpResponse, Error> {
    let mut ctx = Context::new();
    let body = tera.into_inner().render("support.html", &ctx).unwrap();
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

fn create_actix_admin_builder() -> ActixAdminBuilder {
    let configuration = ActixAdminConfiguration {
        ...
    };

    let _support_route = admin_builder.add_support_handler("/support", web::get().to(support));

    admin_builder
}
```

The only prequisite is that the response must have an element with id="support_content".

```html
<div id="support_content" support_block">
    <article class="media">
        <div class="media-content">
            <div class="content">
                <p>
                    <strong>Support</strong>
                    <br>
                    Curabitur arcu velit, sagittis at lectus nec, efficitur faucibus sapien.
                </p>
            </div>
        </div>
    </article>
    <div class="field has-addons">
        <div class="control">
            <input class="input" type="text" placeholder="Ask a question">
        </div>
        <div class="control">
            <button class="button is-info">
                Send
            </button>
        </div>
    </div>
</div>
```