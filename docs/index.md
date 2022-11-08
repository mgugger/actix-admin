---
layout: default
list_title: ' '
---

The actix-admin crate aims at creating a web admin interface similar to other admin interfaces (such as [flask-admin](https://github.com/flask-admin/flask-admin) in python).

## Features
1. Async: Builds on [sea-orm](https://crates.io/crates/sea-orm) as the database backend
2. Macros generate the required implementations for models
3. Authentication: optionally pass authentication handler to implement authentication for views
4. Supports custom validation rules
5. Searchable attributes can be specified
6. Supports custom views, handlers and groups in the Navbar

## Example

Check the [examples](https://github.com/mgugger/actix-admin/tree/main/examples) and run with ```cargo run```. The admin interface is accessible under ```localhost:5000/admin/```.

## Screenshot

<img src="https://raw.githubusercontent.com/mgugger/actix-admin/main/static/Screenshot.png"/>