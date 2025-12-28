//! Adapted from <https://github.com/tokio-rs/axum/blob/f84105ae8b078109987b089c47febc3b544e6b80/axum/src/test_helpers/test_client.rs>

use axum::extract::Request;
use axum::response::Response;
use axum::serve;
use std::convert::Infallible;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower::make::Shared;
use tower_service::Service;

fn spawn_service<S>(svc: S) -> std::io::Result<SocketAddr>
where
    S: Service<Request, Response = Response, Error = Infallible> + Clone + Send + 'static,
    S::Future: Send,
{
    let std_listener = std::net::TcpListener::bind("127.0.0.1:0")?;
    std_listener.set_nonblocking(true)?;
    let listener = TcpListener::from_std(std_listener)?;
    let addr = listener.local_addr()?;
    tokio::spawn(async move { serve(listener, Shared::new(svc)).await.expect("server error") });
    Ok(addr)
}

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
        let addr = spawn_service(svc).expect("failed to spawn service");
        let client = reqwest::Client::default();
        TestClient { addr, client }
    }

    pub fn post(&self, url: &str) -> reqwest::RequestBuilder {
        let url = format!("http://{}{}", self.addr, url);
        self.client.post(&url)
    }
}
