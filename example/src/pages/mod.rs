use crate::model::Post;
use askama::Template;
use actix_identity::Identity;
use actix_web::{web, HttpResponse};
use arangoq::{ArangoConnection, Collection, CollectionType, GetAll};
use crate::placeholder::generate_random_upscale;

#[derive(Template)]
#[template(path = "post.html", escape = "html")]
pub struct PostTemplate {
    pub posts: Vec<Post>,
}

pub async fn posts(
    conn: web::Data<ArangoConnection>,
) -> HttpResponse {
    let coll_name = conn.context.collection_name("posts");
    let coll = Collection::new(coll_name.as_str(), CollectionType::Document);
    let query = coll.get_all();

    match query.try_exec::<Post>(&conn).await {
        Ok(req_res) => {
            let posts = req_res.result.into_iter().map(|p| {
                let mut p = p;
                p.image = format!("data:image/png;base64,{}", generate_random_upscale());
                p
            }).collect();
            let html = PostTemplate {
                posts,
            }.render().unwrap();
            HttpResponse::Ok().body(&html)
        }
        Err(err) => HttpResponse::InternalServerError()
            .json(serde_json::json!({ "error": err.to_string()})),
    }
}

pub fn config_app(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(web::resource("/").route(web::get().to(posts)));
}
