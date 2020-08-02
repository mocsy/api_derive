use crate::model::Post;
use heck::CamelCase;
use oas_gen::{ApiId, ApiPath, Oas3Builder};
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Serialize, JsonSchema)]
pub struct CollectionWrapper<T> {
    collection: Vec<T>,
}
#[derive(Serialize, JsonSchema)]
pub struct ErrorResponse {
    #[serde(skip_serializing_if = "String::is_empty", default)]
    err: String,
    #[serde(skip_serializing_if = "String::is_empty", default)]
    ok: String,
}

#[allow(dead_code)]
pub(crate) fn build_public_doc() -> Oas3Builder {
    let mut oasb = Oas3Builder::default();

    let doc_name = "Post";
    let query_params = vec![];
    // list
    let list_path = ApiPath::with_queries(
        Some("api".to_owned()),
        vec![],
        Some(doc_name.to_lowercase()),
        query_params,
    );
    oasb.list::<CollectionWrapper<Post>, ErrorResponse>(&list_path, doc_name.to_camel_case(), None);

    // fetch
    let fetch_path_vec = vec![ApiId::new(doc_name.to_lowercase().as_str(), "{key}")];
    let fetch_path = ApiPath::new(Some("api".to_owned()), fetch_path_vec, None);
    oasb.fetch_with_tests::<Post, ErrorResponse>(
        &fetch_path,
        doc_name.to_camel_case(),
        None,
        &vec![],
    );

    // create
    let create_path = ApiPath::new(
        Some("api".to_owned()),
        vec![],
        Some(doc_name.to_lowercase()),
    );
    oasb.create::<Post, Post, ErrorResponse>(&create_path, doc_name.to_camel_case(), None);

    // update
    oasb.update::<Post, Post, ErrorResponse>(&fetch_path, doc_name.to_camel_case(), None);

    // replace
    oasb.replace::<Post, Post, ErrorResponse>(&fetch_path, doc_name.to_camel_case(), None);

    // delete
    oasb.delete_by_key::<Post, ErrorResponse>(&fetch_path, doc_name.to_camel_case(), None);
    oasb
}

#[test]
fn test_build_public_doc() {
    let oasb = build_public_doc();

    let version = env!("CARGO_PKG_VERSION");
    let json_str = serde_json::to_string_pretty(&oasb.build(version.to_owned()));
    let json_str = json_str.unwrap_or_default();
    let _res = std::fs::write("public_api.json", json_str);
}
