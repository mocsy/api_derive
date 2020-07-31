use arangoq::arango_api::{GetByKey, GetByKeys, Update};
use arangoq::{ArangoConnection, Collection, CollectionType};
use serde::{de::DeserializeOwned, Serialize};

pub trait DbFields {
    fn _key(&self) -> String;
}

///
pub async fn load_by_key<Res>(
    key: String,
    conn: &ArangoConnection,
    coll: &str,
) -> Result<Res, String>
where
    Res: 'static + Serialize + DeserializeOwned + std::fmt::Debug + Send + Clone,
{
    let coll = Collection::new(coll, CollectionType::Document);
    let query = coll.get_by_key(key.clone());
    match query.try_exec::<Res>(conn).await {
        Ok(ar) => {
            if let Some(doc) = ar.result.first() {
                Ok(doc.clone())
            } else {
                Err(format!("Couldn't find Document for key:{}", key))
            }
        }
        Err(err) => Err(err.to_string()),
    }
}

pub async fn load_by_keys<Res>(keys: &[String], conn: &ArangoConnection, coll: &str) -> Vec<Res>
where
    Res: 'static + Serialize + DeserializeOwned + std::fmt::Debug + Send + Clone,
{
    let coll = Collection::new(coll, CollectionType::Document);
    let query = coll.get_by_keys(keys);
    match query.try_exec::<Res>(conn).await {
        Ok(ar) => ar.result,
        Err(err) => {
            log::error!("{}", err);
            vec![]
        }
    }
}

pub async fn update_in_db<R>(data: &R, conn: &ArangoConnection) -> Option<R>
where
    R: std::fmt::Debug + DeserializeOwned + Serialize + DbFields + Clone,
{
    let local_name = super::get_type_name::<R>(&data).to_lowercase();
    let db_coll = conn.context.collection_name(&local_name);
    let coll = Collection::new(db_coll.as_str(), CollectionType::Document);
    let query = coll.update(data._key(), data);
    match query.try_exec::<R>(&conn).await {
        Ok(ar) => {
            if let Some(doc) = ar.result.first() {
                Some(doc.clone())
            } else {
                None
            }
        }
        Err(_) => None,
    }
}
