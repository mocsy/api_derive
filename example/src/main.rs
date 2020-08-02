#![forbid(unsafe_code)]
// #![warn(clippy::pedantic)]
#[macro_use]
extern crate validator_derive;

#[cfg(test)]
mod test;

mod actors;
mod init;
mod model;
mod pages;
mod placeholder;
mod route;

use crate::actors::CreatedActor;
use actix::Actor;
use actix_cors::Cors;
use actix_files as fs;
use actix_web::{middleware, web, App, HttpResponse, HttpServer};
use arangoq::ArangoConnection;
use log::info;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let db_conn = std::env::var("DB_CONN").expect("DB_CONN is mandatory.");
    let db_name = std::env::var("DB_NAME").expect("DB_NAME is mandatory.");
    let app_prefix: String = std::env::var("DB_COLL_PREFIX").unwrap_or_default();
    let connection = ArangoConnection::with_context(
        db_conn,
        db_name,
        reqwest::Client::new(),
        arangoq::Context { app_prefix },
    );
    init::setup(&connection).await;

    let cacti = CreatedActor {
        conn: connection.clone(),
    }
    .start();

    let bind_url =
        std::env::var("BIND_URL").unwrap_or_else(|_| panic!("{} must be set", "BIND_URL"));
    info!("Listening on http://{}", bind_url);

    HttpServer::new(move || {
        App::new()
            .data(connection.clone())
            .data(cacti.clone())
            .wrap(Cors::new().supports_credentials().max_age(43200).finish())
            .service(web::resource("/health").route(web::get().to(|| HttpResponse::Ok().finish())))
            .service(fs::Files::new("/static", "static"))
            .configure(pages::config_app)
            .configure(route::config_app)
            .wrap(middleware::Logger::default())
    })
    .bind(bind_url)?
    .run()
    .await
}
