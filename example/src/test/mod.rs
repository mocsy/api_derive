use actix::{Actor, Context as ActorContext, Handler, Message, ResponseFuture};
use actix_service::Service;
use actix_web::{http::StatusCode, test, web, App};
use api_derive::{derive_db_fields, Create, Delete, Fetch, GetAll, Replace, Update};
use arangoq::test::TestResponse;
use arangoq::*;
use bytes::Bytes;
use mockito::mock;
use serde::{Deserialize, Serialize};
use validator::Validate;

// Set your own Created actor
#[derive(Debug, Serialize, Deserialize)]
pub struct Created<T> {
    pub data: T,
}

impl<T> Message for Created<T> {
    type Result = Result<bool, ()>;
}

#[derive(Clone)]
pub struct CreatedActor {
    pub conn: ArangoConnection,
}
// Keep actor implementations here
impl Actor for CreatedActor {
    type Context = ActorContext<Self>;
}

/// GetAll, Fetch, Create require Serialize, Deserialize, ArangoBuilder
/// Create also requires Clone and Validate
/// PartialEq required by Validate
#[derive_db_fields(DropExtra)]
#[derive(
    Clone,
    Debug,
    Default,
    Serialize,
    Deserialize,
    ArangoBuilder,
    GetAll,
    Fetch,
    Create,
    Validate,
    Update,
    Replace,
    Delete,
)]
pub struct TestDocument {
    #[validate(range(min = 0))]
    pub id: u64,

    #[serde(default)]
    pub number: u64,
    #[validate(length(min = 2), non_control_character)]
    pub title: String,
    #[validate(length(min = 0), non_control_character)]
    #[author]
    pub name: String,
}
impl Handler<Created<TestDocument>> for CreatedActor {
    type Result = ResponseFuture<Result<bool, ()>>;

    fn handle(&mut self, msg: Created<TestDocument>, _: &mut actix::Context<Self>) -> Self::Result {
        log::debug!("CreatedActor handle TestDocument {:#?}", msg);
        Box::pin(futures::future::ok(false))
    }
}

#[actix_rt::test]
async fn test_fetch() {
    // std::env::set_var("RUST_LOG", "debug,hyper=info");
    // let _res = env_logger::try_init();

    let test_doc = TestDocument {
        id: 8,
        title: "NU".to_owned(),
        name: "4242".to_owned(),
        _key: "537130".to_owned(),
        ..TestDocument::default()
    };
    let mock_resp =
        serde_json::to_string(&TestResponse::with_results(&[test_doc.clone()])).unwrap();
    std::env::set_var("ARANGO_USER_NAME", "test_write");
    std::env::set_var("ARANGO_PASSWORD", "not_a_real_password");

    let _m = mock("POST", "/_db/test_db/_api/cursor")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(mock_resp)
        .expect(4)
        .create();

    let connection = ArangoConnection::with_context(
        mockito::server_url(),
        "test_db".to_owned(),
        reqwest::Client::new(),
        Context {
            app_prefix: "api".to_owned(),
        },
    );

    let mut app = test::init_service(
        App::new()
            .data(connection)
            .service(
                web::resource("/parents/{oid}/testdocument/{key}")
                    .route(web::get().to(fetch_testdocument)),
            )
            .service(web::resource("/testdocument/{key}").route(web::get().to(fetch_testdocument))),
    )
    .await;

    let request = test::TestRequest::get()
        .uri("/testdocument/537130")
        .to_request();

    let response = app.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Check content type
    let hmap = response.headers();
    log::debug!("response: {:#?}", hmap);
    assert_eq!("application/json", hmap.get("content-type").unwrap());

    // Check payload
    let bdy = test::read_body(response).await;
    log::debug!("response: {:?}", bdy);
    assert_eq!(
        bdy,
        Bytes::from_static(
            b"{\"_key\":\"537130\",\"id\":8,\"number\":0,\"title\":\"NU\",\"name\":\"4242\"}"
        )
    );

    let request = test::TestRequest::get()
        .uri("/parents/4242/testdocument/537130")
        .to_request();

    let response = app.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let request = test::TestRequest::get()
        .uri("/testdocument/537130")
        .to_request();

    let response = app.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let request = test::TestRequest::get()
        .uri("/parents/5500/testdocument/537130")
        .to_request();

    let response = app.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Check content type
    let hmap = response.headers();
    log::debug!("response: {:#?}", hmap);
    assert_eq!("application/json", hmap.get("content-type").unwrap());

    // Check payload
    let bdy = test::read_body(response).await;
    log::debug!("response: {:?}", bdy);
    assert_eq!(
        bdy,
        Bytes::from_static(
            b"{\"_key\":\"537130\",\"id\":8,\"number\":0,\"title\":\"NU\",\"name\":\"4242\"}"
        )
    );

    _m.assert();
}

#[actix_rt::test]
async fn test_create_api() {
    // std::env::set_var("RUST_LOG", "debug,hyper=info");
    // let _res = env_logger::try_init();

    let test_doc = TestDocument {
        id: 8,
        number: 96,
        title: "NU".to_owned(),
        name: "4242".to_owned(),
        _key: "537130".to_owned(),
        ..TestDocument::default()
    };
    let mock_resp =
        serde_json::to_string(&TestResponse::with_results(&[test_doc.clone()])).unwrap();
    std::env::set_var("ARANGO_USER_NAME", "test_user");
    std::env::set_var("ARANGO_PASSWORD", "not_a_real_password");

    let _m = mock("POST", "/_db/test_db/_api/cursor")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(mock_resp)
        .expect(2)
        .create();

    let connection = ArangoConnection::with_context(
        mockito::server_url(),
        "test_db".to_owned(),
        reqwest::Client::new(),
        Context {
            app_prefix: "api".to_owned(),
        },
    );

    let cacti = CreatedActor {
        conn: connection.clone(),
    }
    .start();

    let mut app = test::init_service(
        App::new()
            .data(connection)
            .data(cacti)
            .service(
                web::resource("/parents/{oid}/testdocument")
                    .route(web::post().to(create_testdocument)),
            )
            .service(web::resource("/testdocument").route(web::post().to(create_testdocument))),
    )
    .await;

    let test_data = serde_json::json!(test_doc);
    let request = test::TestRequest::post()
        .uri("/testdocument")
        .set_json(&test_data)
        .to_request();

    let response = app.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Check content type
    let hmap = response.headers();
    log::debug!("response: {:#?}", hmap);
    assert_eq!("application/json", hmap.get("content-type").unwrap());

    // Check payload
    let bdy = test::read_body(response).await;
    log::debug!("response: {:?}", bdy);
    assert_eq!(
        bdy,
        Bytes::from_static(
            b"{\"_key\":\"537130\",\"id\":8,\"number\":96,\"title\":\"NU\",\"name\":\"4242\"}"
        )
    );

    let request = test::TestRequest::post()
        .uri("/parents/4242/testdocument")
        .set_json(&test_data)
        .to_request();

    let response = app.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let request = test::TestRequest::post()
        .uri("/parents/5500/testdocument")
        .set_json(&test_data)
        .to_request();

    // Check status code
    let response = app.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Check payload
    let bdy = test::read_body(response).await;
    log::debug!("response: {:?}", bdy);
    assert_eq!(
        bdy,
        Bytes::from_static(
            b"{\"Err\":\"Invalid name of TestDocument: \\\"4242\\\", should be: 5500\"}"
        )
    );

    _m.assert();
}

#[actix_rt::test]
async fn test_update() {
    // std::env::set_var("RUST_LOG", "debug,hyper=info");
    // let _res = env_logger::try_init();

    let test_doc = TestDocument {
        id: 16,
        number: 69,
        title: "NU".to_owned(),
        name: "4242".to_owned(),
        _key: "537130".to_owned(),
        ..TestDocument::default()
    };
    let mock_resp =
        serde_json::to_string(&TestResponse::with_results(&[test_doc.clone()])).unwrap();
    std::env::set_var("ARANGO_USER_NAME", "test_user");
    std::env::set_var("ARANGO_PASSWORD", "not_a_real_password");

    let _m = mock("POST", "/_db/test_db/_api/cursor")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(mock_resp)
        .expect(6)
        .create();

    let connection = ArangoConnection::with_context(
        mockito::server_url(),
        "test_db".to_owned(),
        reqwest::Client::new(),
        Context {
            app_prefix: "api".to_owned(),
        },
    );

    let mut app = test::init_service(
        App::new()
            .data(connection)
            .service(
                web::resource("/parents/{oid}/testdocument/{key}")
                    .route(web::patch().to(update_testdocument)),
            )
            .service(
                web::resource("/testdocument/{key}").route(web::patch().to(update_testdocument)),
            ),
    )
    .await;

    let request = test::TestRequest::patch()
        .uri("/testdocument/537130")
        .set_json(&serde_json::json!({"title":"NU"}))
        .to_request();

    let response = app.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Check content type
    let hmap = response.headers();
    log::debug!("response: {:#?}", hmap);
    assert_eq!("application/json", hmap.get("content-type").unwrap());

    // Check payload
    let bdy = test::read_body(response).await;
    log::debug!("response: {:?}", bdy);
    assert_eq!(
        bdy,
        Bytes::from_static(
            b"{\"_key\":\"537130\",\"id\":16,\"number\":69,\"title\":\"NU\",\"name\":\"4242\"}"
        )
    );

    let request = test::TestRequest::patch()
        .uri("/parents/4242/testdocument/537130")
        .set_json(&serde_json::json!({"title":"NU"}))
        .to_request();

    let response = app.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let request = test::TestRequest::patch()
        .uri("/parents/4242/testdocument/537130")
        .set_json(&serde_json::json!({"title":"RU"}))
        .to_request();

    let response = app.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let request = test::TestRequest::patch()
        .uri("/testdocument/537130")
        .set_json(&serde_json::json!({"title":"NU"}))
        .to_request();

    let response = app.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let request = test::TestRequest::patch()
        .uri("/parents/5500/testdocument/537130")
        .set_json(&serde_json::json!({"title":"NU", "id": 100, "number": 100}))
        .to_request();

    let response = app.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Check content type
    let hmap = response.headers();
    log::debug!("response: {:#?}", hmap);
    assert_eq!("application/json", hmap.get("content-type").unwrap());

    // Check payload
    let bdy = test::read_body(response).await;
    log::debug!("response: {:?}", bdy);
    assert_eq!(
        bdy,
        Bytes::from_static(
            b"{\"_key\":\"537130\",\"id\":16,\"number\":69,\"title\":\"NU\",\"name\":\"4242\"}"
        )
    );

    let request = test::TestRequest::patch()
        .uri("/parents/4242/testdocument/537130")
        .set_json(&serde_json::json!({"title":"NU", "parent":"5500"}))
        .to_request();

    let response = app.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Check content type
    let hmap = response.headers();
    log::debug!("response: {:#?}", hmap);
    assert_eq!("application/json", hmap.get("content-type").unwrap());

    // Check payload
    let bdy = test::read_body(response).await;
    log::debug!("response: {:?}", bdy);
    assert_eq!(
        bdy,
        Bytes::from_static(
            b"{\"_key\":\"537130\",\"id\":16,\"number\":69,\"title\":\"NU\",\"name\":\"4242\"}"
        )
    );

    _m.assert();
}

#[actix_rt::test]
async fn test_replace() {
    // std::env::set_var("RUST_LOG", "debug,hyper=info");
    // let _res = env_logger::try_init();

    let test_doc = TestDocument {
        id: 8,
        title: "NU".to_owned(),
        name: "4242".to_owned(),
        _key: "537130".to_owned(),
        ..TestDocument::default()
    };
    let mock_resp =
        serde_json::to_string(&TestResponse::with_results(&[test_doc.clone()])).unwrap();
    std::env::set_var("ARANGO_USER_NAME", "test_user");
    std::env::set_var("ARANGO_PASSWORD", "not_a_real_password");

    let _m = mock("POST", "/_db/test_db/_api/cursor")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(mock_resp)
        .expect(3)
        .create();

    let connection = ArangoConnection::with_context(
        mockito::server_url(),
        "test_db".to_owned(),
        reqwest::Client::new(),
        Context {
            app_prefix: "api".to_owned(),
        },
    );

    let mut app = test::init_service(
        App::new()
            .data(connection)
            .service(
                web::resource("/parents/{oid}/testdocument/{key}")
                    .route(web::put().to(replace_testdocument)),
            )
            .service(
                web::resource("/testdocument/{key}").route(web::put().to(replace_testdocument)),
            ),
    )
    .await;

    let test_data = serde_json::json!({
        "id": 8,
        "title": "Watanuki",
        "name": "",
    });

    let request = test::TestRequest::put()
        .uri("/testdocument/8")
        .set_json(&test_data)
        .to_request();

    let response = app.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Check content type
    let hmap = response.headers();
    log::debug!("response: {:#?}", hmap);
    assert_eq!("application/json", hmap.get("content-type").unwrap());

    // Check payload
    let bdy = test::read_body(response).await;
    log::debug!("response: {:?}", bdy);
    assert_eq!(
        bdy,
        Bytes::from_static(
            b"{\"_key\":\"537130\",\"id\":8,\"number\":0,\"title\":\"NU\",\"name\":\"4242\"}"
        )
    );

    let request = test::TestRequest::put()
        .uri("/parents/4242/testdocument/2")
        .set_json(&test_data)
        .to_request();

    let response = app.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Check content type
    let hmap = response.headers();
    log::debug!("response: {:#?}", hmap);
    assert_eq!("application/json", hmap.get("content-type").unwrap());

    // Check payload
    let bdy = test::read_body(response).await;
    log::debug!("response: {:?}", bdy);
    assert_eq!(
        bdy,
        Bytes::from_static(
            b"{\"_key\":\"537130\",\"id\":8,\"number\":0,\"title\":\"NU\",\"name\":\"4242\"}"
        )
    );

    let request = test::TestRequest::put()
        .uri("/parents/5500/testdocument/2")
        .set_json(&test_data)
        .to_request();

    let response = app.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Check content type
    let hmap = response.headers();
    log::debug!("response: {:#?}", hmap);
    assert_eq!("application/json", hmap.get("content-type").unwrap());

    // Check payload
    let bdy = test::read_body(response).await;
    log::debug!("response: {:?}", bdy);
    assert_eq!(
        bdy,
        Bytes::from_static(
            b"{\"_key\":\"537130\",\"id\":8,\"number\":0,\"title\":\"NU\",\"name\":\"4242\"}"
        )
    );

    let test_data = serde_json::json!({
        "id": 8,
        "title": "Watanuki",
        "name": "5500",
    });
    let request = test::TestRequest::put()
        .uri("/parents/4242/testdocument/2")
        .set_json(&test_data)
        .to_request();

    // Check status code
    let response = app.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Check payload
    let bdy = test::read_body(response).await;
    log::debug!("response: {:?}", bdy);
    assert_eq!(
        bdy,
        Bytes::from_static(
            b"{\"Err\":\"Invalid name of TestDocument: \\\"5500\\\", should be: 4242\"}"
        )
    );

    _m.assert();
}

#[actix_rt::test]
async fn test_delete() {
    // std::env::set_var("RUST_LOG", "debug,hyper=info");
    // let _res = env_logger::try_init();

    let test_doc = TestDocument {
        id: 8,
        title: "NU".to_owned(),
        name: "4242".to_owned(),
        _key: "537130".to_owned(),
        ..TestDocument::default()
    };
    let mock_resp =
        serde_json::to_string(&TestResponse::with_results(&[test_doc.clone()])).unwrap();
    std::env::set_var("ARANGO_USER_NAME", "test_user");
    std::env::set_var("ARANGO_PASSWORD", "not_a_real_password");

    let _m = mock("POST", "/_db/test_db/_api/cursor")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(mock_resp)
        .expect(3)
        .create();

    let connection = ArangoConnection::with_context(
        mockito::server_url(),
        "test_db".to_owned(),
        reqwest::Client::new(),
        Context {
            app_prefix: "api".to_owned(),
        },
    );

    let mut app = test::init_service(
        App::new()
            .data(connection)
            .service(
                web::resource("/parents/{oid}/testdocument/{key}")
                    .route(web::delete().to(replace_testdocument)),
            )
            .service(
                web::resource("/testdocument/{key}").route(web::delete().to(replace_testdocument)),
            ),
    )
    .await;

    let test_data = serde_json::json!({
        "id": 8,
        "title": "Watanuki",
        "name": "",
    });
    let request = test::TestRequest::delete()
        .uri("/testdocument/8")
        .set_json(&test_data)
        .to_request();

    let response = app.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Check content type
    let hmap = response.headers();
    log::debug!("response: {:#?}", hmap);
    assert_eq!("application/json", hmap.get("content-type").unwrap());

    // Check payload
    let bdy = test::read_body(response).await;
    log::debug!("response: {:?}", bdy);
    assert_eq!(
        bdy,
        Bytes::from_static(
            b"{\"_key\":\"537130\",\"id\":8,\"number\":0,\"title\":\"NU\",\"name\":\"4242\"}"
        )
    );

    let request = test::TestRequest::delete()
        .uri("/parents/4242/testdocument/2")
        .set_json(&test_data)
        .to_request();

    let response = app.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Check content type
    let hmap = response.headers();
    log::debug!("response: {:#?}", hmap);
    assert_eq!("application/json", hmap.get("content-type").unwrap());

    // Check payload
    let bdy = test::read_body(response).await;
    log::debug!("response: {:?}", bdy);
    assert_eq!(
        bdy,
        Bytes::from_static(
            b"{\"_key\":\"537130\",\"id\":8,\"number\":0,\"title\":\"NU\",\"name\":\"4242\"}"
        )
    );

    let request = test::TestRequest::delete()
        .uri("/parents/5500/testdocument/2")
        .set_json(&test_data)
        .to_request();

    let response = app.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Check content type
    let hmap = response.headers();
    log::debug!("response: {:#?}", hmap);
    assert_eq!("application/json", hmap.get("content-type").unwrap());

    // Check payload
    let bdy = test::read_body(response).await;
    log::debug!("response: {:?}", bdy);
    assert_eq!(
        bdy,
        Bytes::from_static(
            b"{\"_key\":\"537130\",\"id\":8,\"number\":0,\"title\":\"NU\",\"name\":\"4242\"}"
        )
    );

    _m.assert();
}
