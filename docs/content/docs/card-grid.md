---
title: "Card Grid"
date: 2023-01-17T11:44:56+01:00
draft: false
weight: 5
---

# Card Grid

You can create a view by using a card grid, which will load a predefined number of cards and load their content via hx-get on htmx.

```rust
async fn card(
    tera: web::Data<Tera>
) -> Result<HttpResponse, Error> {
    let mut ctx = Context::new();
    let body = tera.into_inner().render("card.html", &ctx).unwrap();
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

fn create_actix_admin_builder() -> ActixAdminBuilder {
    let configuration = ActixAdminConfiguration {
        ...
    };

    let mut admin_builder = ActixAdminBuilder::new(configuration);

    let _card_route = admin_builder.add_custom_handler("card", "/card/{id}", web::get().to(card), false);

    // this will load 2 rows of cards, the first row containts 2 cards (1 & 2) and the second row contains card 3
    let card_grid: Vec<Vec<String>> = vec![
        vec!["admin/card/1".to_string(), "admin/card/2".to_string()],
        vec!["admin/card/3".to_string()],
    ];
    // add the card to the navbar
    admin_builder.add_card_grid("Card Grid", "/my_card_grid", card_grid, true);

    admin_builder
}
```