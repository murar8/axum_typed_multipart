//! Adapted from <https://github.com/tokio-rs/axum/blob/axum-v0.7.1/axum/src/test_helpers/test_client.rs>
//!
//! This module can be replaced by <https://github.com/cloudwalk/axum-test-helper>
//! once support for axum v0.7 is added.

use axum::extract::Request;
use axum::response::Response;
use axum::serve;
use bytes::Bytes;
use http::header::{HeaderName, HeaderValue};
use http::StatusCode;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::str::FromStr;
use tokio::net::TcpListener;
use tower::make::Shared;
use tower_service::Service;

pub struct TestClient {
    client: reqwest::Client,
    addr: SocketAddr,
}

impl TestClient {
    pub fn new<S>(svc: S) -> Self
    where
        S: Service<Request, Response = Response, Error = Infallible> + Clone + Send + 'static,
        S::Future: Send,
    {
        let std_listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        std_listener.set_nonblocking(true).unwrap();
        let listener = TcpListener::from_std(std_listener).unwrap();

        let addr = listener.local_addr().unwrap();
        println!("Listening on {addr}");

        tokio::spawn(async move { serve(listener, Shared::new(svc)).await.expect("server error") });

        let client =
            reqwest::Client::builder().redirect(reqwest::redirect::Policy::none()).build().unwrap();

        TestClient { client, addr }
    }

    pub fn get(&self, url: &str) -> RequestBuilder {
        RequestBuilder { builder: self.client.get(format!("http://{}{}", self.addr, url)) }
    }

    pub fn head(&self, url: &str) -> RequestBuilder {
        RequestBuilder { builder: self.client.head(format!("http://{}{}", self.addr, url)) }
    }

    pub fn post(&self, url: &str) -> RequestBuilder {
        RequestBuilder { builder: self.client.post(format!("http://{}{}", self.addr, url)) }
    }

    #[allow(dead_code)]
    pub fn put(&self, url: &str) -> RequestBuilder {
        RequestBuilder { builder: self.client.put(format!("http://{}{}", self.addr, url)) }
    }

    #[allow(dead_code)]
    pub fn patch(&self, url: &str) -> RequestBuilder {
        RequestBuilder { builder: self.client.patch(format!("http://{}{}", self.addr, url)) }
    }
}

pub struct RequestBuilder {
    builder: reqwest::RequestBuilder,
}

impl RequestBuilder {
    pub async fn send(self) -> TestResponse {
        TestResponse { response: self.builder.send().await.unwrap() }
    }

    pub fn body(mut self, body: impl Into<reqwest::Body>) -> Self {
        self.builder = self.builder.body(body);
        self
    }

    pub fn json<T>(mut self, json: &T) -> Self
    where
        T: serde::Serialize,
    {
        self.builder = self.builder.json(json);
        self
    }

    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
    {
        self.builder = self.builder.header(key, value);
        self
    }

    #[allow(dead_code)]
    pub fn multipart(mut self, form: reqwest::multipart::Form) -> Self {
        self.builder = self.builder.multipart(form);
        self
    }
}

#[derive(Debug)]
pub struct TestResponse {
    response: reqwest::Response,
}

impl TestResponse {
    #[allow(dead_code)]
    pub async fn bytes(self) -> Bytes {
        self.response.bytes().await.unwrap()
    }

    pub async fn text(self) -> String {
        self.response.text().await.unwrap()
    }

    #[allow(dead_code)]
    pub async fn json<T>(self) -> T
    where
        T: serde::de::DeserializeOwned,
    {
        self.response.json().await.unwrap()
    }

    pub fn status(&self) -> StatusCode {
        StatusCode::from_u16(self.response.status().as_u16()).unwrap()
    }

    pub fn headers(&self) -> http::HeaderMap {
        // reqwest still uses http 0.2 so have to convert into http 1.0
        let mut headers = http::HeaderMap::new();
        for (key, value) in self.response.headers() {
            let key = http::HeaderName::from_str(key.as_str()).unwrap();
            let value = http::HeaderValue::from_bytes(value.as_bytes()).unwrap();
            headers.insert(key, value);
        }
        headers
    }

    pub async fn chunk(&mut self) -> Option<Bytes> {
        self.response.chunk().await.unwrap()
    }

    pub async fn chunk_text(&mut self) -> Option<String> {
        let chunk = self.chunk().await?;
        Some(String::from_utf8(chunk.to_vec()).unwrap())
    }
}
