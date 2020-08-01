use arangoq::ArangoConnection;
use log::debug;
use serde_json::json;

pub(crate) const DOCUMENT_COLLECTIONS: [&str; 1] = ["posts"];

pub async fn setup(conn: &ArangoConnection) {
    for local_name in DOCUMENT_COLLECTIONS.iter() {
        create_collection(local_name, arangoq::CollectionType::Document, conn).await;
    }
    ensure_index("posts", true, true, vec!["title".to_owned()], conn).await;
}

// TODO: cleanup & move this to arangoq
async fn create_collection(
    local_name: &str,
    collection_type: arangoq::CollectionType,
    conn: &ArangoConnection,
) {
    let qualified_name = conn.context.collection_name(local_name);
    let coll_url = conn.collection();

    let data = json!({
        "name": qualified_name,
        "type": collection_type as u8
    });
    debug!("{}", data.to_string());
    let res = conn
        .client
        .post(coll_url.as_str())
        .header("accept", "application/json")
        .header("content-type", "application/json")
        .basic_auth(
            std::env::var("ARANGO_USER_NAME").unwrap_or_default(),
            std::env::var("ARANGO_PASSWORD").ok(),
        )
        .json(&data)
        .send()
        .await;
    debug!("{:#?}", res);
}

// TODO: cleanup & move this to arangoq
/// Only supports type: Hash
async fn ensure_index(
    local_name: &str,
    unique: bool,
    sparse: bool,
    fields_to_index: Vec<String>,
    conn: &ArangoConnection,
) {
    let index_api_url = format!("{}/_db/{}/_api/index", conn.host, conn.database);
    let qualified_name = conn.context.collection_name(local_name);

    let index_details = json!({
        "type": "hash",
        "fields": fields_to_index,
        "unique": unique,
        "sparse": sparse
    });

    debug!("{}", index_details.to_string());
    let res = conn
        .client
        .post(index_api_url.as_str())
        .query(&[("collection", qualified_name)])
        .header("accept", "application/json")
        .basic_auth(
            std::env::var("ARANGO_USER_NAME").unwrap_or_default(),
            std::env::var("ARANGO_PASSWORD").ok(),
        )
        .json(&index_details)
        .send()
        .await;
    debug!("{:#?}", res);
    let jsresp: serde_json::Value = res.unwrap().json().await.unwrap();
    debug!("{:#?}", jsresp);
}
