use crate::model::Post;
use crate::placeholder::*;
use actix_web::{web, HttpResponse};
use arangoq::{ArangoConnection, Collection, CollectionType, GetAll};
use askama::Template;
use serde::Deserialize;

#[derive(Template)]
#[template(path = "post.html", escape = "html")]
pub struct PostTemplate {
    pub posts: Vec<Post>,
}

#[derive(Deserialize)]
pub struct ImgOpt {
    #[serde(default)]
    pub gen: ImageGen,
}

#[derive(Deserialize)]
pub enum ImageGen {
    Bricks,
    Mix,
    Noise,
    Fractal,
    Synth,
}
impl Default for ImageGen {
    fn default() -> Self {
        ImageGen::Bricks
    }
}
impl ImageGen {
    pub fn generate(&self) -> String {
        match self {
            ImageGen::Bricks => generate_random_upscale(),
            ImageGen::Mix => mix_image(),
            ImageGen::Noise => generate_random_image(),
            ImageGen::Fractal => generate_fractal_image(),
            ImageGen::Synth => generate_image(),
        }
    }
}

pub async fn posts(
    conn: web::Data<ArangoConnection>,
    image_generator: web::Query<ImgOpt>,
) -> HttpResponse {
    let image_generator = &image_generator.gen;
    let coll_name = conn.context.collection_name("posts");
    let coll = Collection::new(coll_name.as_str(), CollectionType::Document);
    let query = coll.get_all();

    match query.try_exec::<Post>(&conn).await {
        Ok(req_res) => {
            let posts = req_res
                .result
                .into_iter()
                .map(|p| {
                    let mut p = p;
                    p.image = format!("data:image/png;base64,{}", image_generator.generate());
                    p
                })
                .collect();
            let html = PostTemplate { posts }.render().unwrap();
            HttpResponse::Ok().body(&html)
        }
        Err(err) => {
            HttpResponse::InternalServerError().json(serde_json::json!({ "error": err.to_string()}))
        }
    }
}

pub fn config_app(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(web::resource("/").route(web::get().to(posts)));
}
