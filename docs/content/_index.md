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

## Minimal Cargo.toml

```toml
[package]
name = "actix-admin-example"
description = "An admin interface for actix-web"
version = "0.5.0"
edition = "2021"

[[bin]]
name = "actix-admin-example"
path = "main.rs"

[dependencies]
actix-web = "^4.3.1"
actix-rt = "2.7.0"
actix-multipart = "^0.4.0"
sea-orm = { version = "^0.11.3", features = [ "sqlx-sqlite", "runtime-actix-native-tls", "macros" ], default-features = true }
chrono = "0.4.23"
tera = "^1.17.1"
serde = "^1.0.152"
serde_derive = "^1.0.152"
actix-admin = "0.7.0"
regex = "1.7.1"
```