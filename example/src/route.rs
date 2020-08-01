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
