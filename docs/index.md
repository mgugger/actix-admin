---
layout: default
list_title: ' '
---

The actix-admin crate aims at creating a web admin interface similar to other admin interfaces (such as [flask-admin](https://github.com/flask-admin/flask-admin) in python).

## Features
1. Async: Builds on [sea-orm](https://crates.io/crates/sea-orm) for the database backend
2. Macros: Generate the required implementations for models automatically
3. Authentication: optionally pass authentication handler to implement authentication for views
4. Supports custom validation rules
5. Searchable attributes can be specified
6. Supports custom views which are added to the Navbar

## Example

Check the [example](https://github.com/mgugger/actix-admin/tree/main/example) and run with ```cargo run```. The admin interface is accessible under ```localhost:5000/admin/```.

## Screenshot

<img src="https://raw.githubusercontent.com/mgugger/actix-admin/main/static/Screenshot.png"/>