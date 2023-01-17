---
title: ""
date: 2023-01-17T11:44:56+01:00
draft: false
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

Check the [examples](https://github.com/mgugger/actix-admin/tree/main/examples) and run  ```cargo run --example basic``` from the root folder for a basic in-memory sqlite version. The admin interface is accessible under ```localhost:5000/admin/```.