#![forbid(unsafe_code)]
pub mod db;

use actix_web::web::HttpResponse;
use arangoq::{ArangoConnection, ArangoQuery, ArangoResponse};
pub use db::*;
use futures::future::Either;
use serde::{de::DeserializeOwned, Serialize};

/// You can handle the ok case of a db query with handling Either::Left.
/// Either::Right is a HttpResponse with the error Result as json in response body.
/// Errors are returned as HttpResponses.
/// ```ignore
/// # let db_name = "test_db".to_owned();
/// # let connection = ArangoConnection::with_context(mockito::server_url(), db_name, reqwest::Client::new(), Context{ app_prefix: "api", },);
/// # use crate::model::organizer::*;
/// # let exist_query = Organizer::query_builder(qualified_name.as_str())
/// # .read()
/// # .filter()
/// # .name_eq(&new_org.name)
/// # .limit(1)
/// # .build();
/// match web_query_ok::<Organizer>(exist_query, &conn).await {
///   Either::Left(ar) => {},
///   Either::Right(err) => err,
/// }
/// ```
pub async fn web_query_ok<Res>(
    query: ArangoQuery,
    conn: &ArangoConnection,
) -> Either<ArangoResponse<Res>, HttpResponse>
where
    Res: 'static + Serialize + DeserializeOwned + std::fmt::Debug + Send,
{
    // let dbq = arangoq::DbQuery(query, std::marker::PhantomData::<Res>);
    // match conn.send(dbq).await {
    match query.try_exec::<Res>(conn).await {
        Ok(req_res) =>
            // match req_res {
            // Ok(ar) => {
                {
                let ar = req_res;
                if !ar.error {
                    Either::Left(ar)
                } else {
                    let msg = format!("Database Error:{} {}", ar.error_num, ar.error_message);
                    log::error!("{:#?} -> {}", ar, msg);
                    Either::Right(HttpResponse::InternalServerError().json(Err::<String, _>(msg)))
                }
                }
            // }
            // Err(err) => Either::Right(
            //     HttpResponse::InternalServerError().json(Err::<String, _>(err.to_string())),
            // ),
        // },
        Err(err) => Either::Right(
            HttpResponse::InternalServerError().json(Err::<String, _>(err.to_string())),
        ),
    }
}

pub fn get_type_name<T>(it_is: &T) -> String
where
    T: std::fmt::Debug,
{
    let nm = format!("{:?}", it_is);
    nm.split_whitespace().next().unwrap().to_owned()
}

#[cfg(test)]
mod tests {

    use super::get_type_name;

    #[derive(Debug)]
    struct Zed();

    #[derive(Debug)]
    struct SuperZed {
        my_name: String,
    }

    #[test]
    fn test_get_type_name() {
        let z = Zed();
        assert_eq!("Zed", get_type_name(&z));

        let sz = SuperZed {
            my_name: "My name is Zed!".to_owned(),
        };
        assert_eq!("SuperZed", get_type_name(&sz));
    }
}
