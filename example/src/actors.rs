use crate::model::Post;
use actix::{Actor, Context, Handler, Message, ResponseFuture};
use arangoq::ArangoConnection;
use serde::{Deserialize, Serialize};

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
    type Context = Context<Self>;
}

impl Handler<Created<Post>> for CreatedActor {
    type Result = ResponseFuture<Result<bool, ()>>;

    fn handle(&mut self, msg: Created<Post>, _: &mut Context<Self>) -> Self::Result {
        log::debug!("CreatedActor handle Post {:#?}", msg);
        Box::pin(futures::future::ok(false))
    }
}
