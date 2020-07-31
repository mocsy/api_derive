#![forbid(unsafe_code)]
// #![warn(clippy::pedantic)]
#[macro_use]
extern crate validator_derive;

#[cfg(test)]
mod test;

mod actors;
mod model;
mod pages;

use actix::Actor;
use actix_cors::Cors;
use actix_identity::{CookieIdentityPolicy, Identity, IdentityService};
use actix_web::{middleware, web, App, HttpResponse, HttpServer};
use arangoq::ArangoConnection;

use crate::actors::CreatedActor;
use pages::*;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let cookie_key = b"moAZVQajijrZeXTZTfiZSgujcCMMB6F7X64uZPt3SGxrzT4XiaHh78TJQ3CCRJYW";
    let db_conn = std::env::var("DB_CONN").expect("DB_CONN is mandatory.");
    let db_name = std::env::var("DB_NAME").expect("DB_NAME is mandatory.");
    let app_prefix: String = std::env::var("DB_COLL_PREFIX").unwrap_or_default();
    let connection = ArangoConnection::with_context(
        db_conn,
        db_name,
        reqwest::Client::new(),
        arangoq::Context { app_prefix },
    );

    let cacti = CreatedActor {
        conn: connection.clone(),
    }
    .start();

    let bind_url =
        std::env::var("BIND_URL").unwrap_or_else(|_| panic!("{} must be set", "BIND_URL"));

    HttpServer::new(move || {
        App::new()
            .data(connection.clone())
            .data(cacti.clone())
            .wrap(Cors::new().supports_credentials().max_age(43200).finish())
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(cookie_key)
                    .name("blog-auth")
                    // .domain(&site_domain)
                    .path("/"),
            ))
            .service(
                web::resource("/health").route(web::get().to(|id: Identity| {
                    log::debug!("Identity: {:?}", id.identity());
                    HttpResponse::Ok().finish()
                })),
            )
            // .configure(base_like::config_app)
            .wrap(middleware::Logger::default())
    })
    .bind(bind_url)?
    .run()
    .await
}
