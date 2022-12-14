use async_std::task::block_on;
#[cfg(feature = "sqlx")]
use async_std::sync::Arc;
use serde::{Serialize};
use crate::{
    utils::{range::RANGE_MAP_SIZE, buffer::Buffer, string::unescaped}, server::{Ohkami, consume_buffer}, prelude::{Response, Result}, components::{json::Json, headers::HeaderKey}
};

pub use crate::{
    components::{
        method::Method,
        status::Status,
    },
    response::body::Body,
};


pub trait ExpectedResponse {fn as_response(self) -> Result<Response>;}
impl ExpectedResponse for Response {fn as_response(self) -> Result<Response> {Err(self)}}
impl ExpectedResponse for Result<Response> {fn as_response(self) -> Result<Response> {self}}

pub trait Test {
    /// Asserts that the server returns expected response to the request.
    fn assert_to_res<R: ExpectedResponse>(&self, request: &Request, expected: R);
    /// Asserts that the server DOESN'T return the response to the request.
    fn assert_not_to_res<R: ExpectedResponse>(&self, request: &Request, expected: R);
    /// Performs one-time handling and returns a `Response`
    fn oneshot_res(&self, request: &Request) -> Response;
    /// Performs one-time handling and asserts the response includes a response body of `application/json`. If so, returns the `JSON`.
    fn oneshot_json<J: for <'j> Json<'j>>(&self, request: &Request) -> J;
} impl Test for Ohkami {
    fn assert_to_res<R: ExpectedResponse>(&self, request: &Request, expected_response: R) {
        let actual_response = block_on(async {
            consume_buffer(
                request.into_request_buffer().await,
                &self.router,

                #[cfg(feature = "sqlx")]
                Arc::clone(&self.pool)
            ).await
        });
        assert_eq!(actual_response, expected_response.as_response())
    }
    fn assert_not_to_res<R: ExpectedResponse>(&self, request: &Request, expected_response: R) {
        let actual_response = block_on(async {
            consume_buffer(
                request.into_request_buffer().await,
                &self.router,

                #[cfg(feature = "sqlx")]
                Arc::clone(&self.pool)
            ).await
        });
        assert_ne!(actual_response, expected_response.as_response())
    }
    fn oneshot_res(&self, request: &Request) -> Response {
        match block_on(async {
            consume_buffer(
                request.into_request_buffer().await,
                &self.router,

                #[cfg(feature = "sqlx")]
                Arc::clone(&self.pool)
            ).await
        }) {
            Ok(res) => res,
            Err(res) => res,
        }
    }
    fn oneshot_json<J: for <'j> Json<'j>>(&self, request: &Request) -> J {
        match block_on(async {
            consume_buffer(
                request.into_request_buffer().await,
                &self.router,
                #[cfg(feature = "sqlx")]
                Arc::clone(&self.pool)
            ).await
        }) {
            Ok(res) => res,
            Err(res) => res,
        }.body_json()
    }
}


#[allow(unused)]
pub struct Request {
    method:  Method,
    uri:     &'static str,
    query:   [Option<(&'static str, &'static str)>; RANGE_MAP_SIZE],
    headers: String,
    body:    Option<String>,
} impl Request {
    pub fn new(method: Method, uri: &'static str) -> Self {
        Self {
            method,
            uri,
            query:   [None, None, None, None],
            headers: String::new(),
            body:    None,
        }
    }
    pub fn query(mut self, key: &'static str, value: &'static str) -> Self {
        let index = 'index: {
            for (i, q) in self.query.iter().enumerate() {
                if q.is_none() {break 'index i}
            }
            panic!("Current ohkami can't handle more than {RANGE_MAP_SIZE} query params");
        };
        self.query[index] = Some((key, value));
        self
    }
    pub fn header<K: HeaderKey>(mut self, key: K, value: &'static str) -> Self {
        self.headers += key.as_key_str();
        self.headers += ": ";
        self.headers += value;
        self.headers += "\r\n";
        self
    }
    pub fn body<S: Serialize>(mut self, body: S) -> Self {
        let body = unescaped(serde_json::to_string(&body).expect("can't serialize given body as a JSON"));
        self.body = Some(body);
        self
    }

    #[allow(unused)]
    pub(crate) async fn into_request_buffer(&self) -> Buffer {
        let request_uri = {
            let mut uri = self.uri.to_owned();
            if self.query[0].is_some() {
                for (i, query) in self.query.iter().enumerate() {
                    match query {
                        None => break,
                        Some((key, value)) => {
                            uri.push(if i==0 {'?'} else {'&'});
                            uri += key;
                            uri.push('=');
                            uri += value;
                        },
                    }
                }
            };
            uri
        };
        let request_str = {
            let mut raw_request = format!(
"{} {} HTTP/1.1
{}",
                self.method,
                request_uri,
                self.headers,
            );
            if let Some(body) = &self.body {
                raw_request.push('\n');
                raw_request += &body
            }
            raw_request
        };
        Buffer::from_http_request_str(request_str).await
    }
}