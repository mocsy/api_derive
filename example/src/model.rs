use crate::actors::{Created, CreatedActor};
use api_derive::{derive_db_fields, Create, Delete, Fetch, GetAll, Replace, Update};
use arangoq::*;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[cfg(test)]
use schemars::JsonSchema;

/// Represents a `Document` in the `posts` document collection
///
/// This struct is using serde attributes to skip phone if not specified,
/// as a way to strip Option<> away from the struct for more ergonomic use.
#[derive_db_fields]
#[derive(
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    Clone,
    Default,
    ArangoBuilder,
    GetAll,
    Fetch,
    Create,
    Update,
    Replace,
    Delete,
    Validate,
)]
#[cfg_attr(test, derive(JsonSchema))]
pub struct Post {
    #[author]
    #[serde(default)]
    #[validate(non_control_character, email)]
    pub author: String,

    #[validate(non_control_character, length(min = 2, max = 300))]
    pub title: String,

    #[validate(non_control_character, length(min = 2, max = 3000))]
    pub content: String,

    #[serde(default)]
    #[validate(non_control_character, url)]
    pub image: String,
}
