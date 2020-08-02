# `api_derive` is an `actix_web` **REST** like api derive macro set
It provides List, Fetch, Create, Replace, Update and Delete operations for any model.
Each is a different derive, so you can choose to skip some of it.

The design brief:
Build a **Rust** toolkit to be able to hack together **low latency** *rest* apis in a matter of minutes in a modern, distributed, Cloud Native, and [Twelve factor](https://12factor.net/) fashion.

Built on `actix_web`, **ArangoDB** and [arangoq](https://github.com/element114/arangoq) for 'maximum' performance (according to the intention).

```rust
use api_derive::{derive_db_fields, Create, Delete, Fetch, GetAll, Replace, Update};
use arangoq::*;
use serde::{Deserialize, Serialize};

#[derive_db_fields]
#[derive(Debug,Clone,Serialize,Deserialize,PartialEq,Default,ArangoBuilder,GetAll,Fetch,Create,Update,Replace,Delete,)]
pub struct Post {
    #[author]
    #[serde(default)]
    pub author: String,
    pub title: String,
    pub content: String,
}
```
Configure the generated endpoints:
```rust
use crate::model::*;
use actix_web::web;

pub fn config_app(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/post")
            .route(web::get().to(list_post))
            .route(web::post().to(create_post)),
    )
    .service(
        web::resource("/post/{key}")
            .route(web::get().to(fetch_post))
            .route(web::patch().to(update_post))
            .route(web::put().to(replace_post))
            .route(web::delete().to(delete_post)),
    );
}
```
See `./example` for more details.
