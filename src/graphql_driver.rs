use actix::prelude::*;
use futures::future::Future;
use serde_json;

use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;

use std;
use super::AppState;

use actix_web::{http, server, middleware, App, Path, State, HttpRequest, HttpResponse,
                HttpMessage, AsyncResponder, FutureResponse, Error};

use graphql_schema::{Schema, create_schema};

#[derive(Serialize, Deserialize)]
pub struct GraphQLData(GraphQLRequest);

impl Message for GraphQLData {
    type Result = Result<String, Error>;
}

pub struct GraphQLExecutor {
    schema: std::sync::Arc<Schema>,
}

impl GraphQLExecutor {
    pub fn new(schema: std::sync::Arc<Schema>) -> GraphQLExecutor {
        GraphQLExecutor { schema: schema }
    }
}

impl Actor for GraphQLExecutor {
    type Context = SyncContext<Self>;
}

impl Handler<GraphQLData> for GraphQLExecutor {
    type Result = Result<String, Error>;

    fn handle(&mut self, msg: GraphQLData, _: &mut Self::Context) -> Self::Result {
        let res = msg.0.execute(&self.schema, &());
        let res_text = serde_json::to_string(&res)?;
        Ok(res_text)
    }
}

pub fn graphiql(_req: HttpRequest<AppState>) -> Result<HttpResponse, Error> {
    let html = graphiql_source("/graphql");
    Ok(
        HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html),
    )
}

pub fn graphql(req: HttpRequest<AppState>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let executor = req.state().executor.clone();
    req.json()
        .from_err()
        .and_then(move |val: GraphQLData| {
            executor.send(val).from_err().and_then(|res| match res {
                Ok(user) => Ok(
                    HttpResponse::Ok()
                        .header(http::header::CONTENT_TYPE, "application/json")
                        .body(user),
                ),
                Err(_) => Ok(HttpResponse::InternalServerError().into()),
            })
        })
        .responder()
}

