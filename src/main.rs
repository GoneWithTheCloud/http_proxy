use anyhow::*;

use std::net::SocketAddr;
use std::sync::Arc;
use hyper::{Body, Client, Request, Server};
use hyper::http::HeaderValue;
use hyper::service::{make_service_fn, service_fn};

fn proxy_crate(req: &mut Request<Body>) -> Result<()> {
    for key in &["content-length", "accept-encoding", "content-encoding", "transfer-encoding", "host"] {
        req.headers_mut().remove(*key);
    }
    // req.headers_mut().insert("host", HeaderValue::from_static("api.openai.com"));
    println!("request {:?}", req);

    let uri = req.uri();
    let uri_string = match uri.query() {
        Some(query_item) => format!("https://api.openai.com{}?{}", uri.path(), query_item),
        None => format!("https://api.openai.com{}", uri.path())
    };

    *req.uri_mut() = uri_string.parse().context("Parsing URI error")?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let https = hyper_rustls::HttpsConnector::with_native_roots();
    let client: Client<_, hyper::Body> = Client::builder().build(https);

    let client: Arc<Client<_, hyper::Body>> = Arc::new(client);

    let addr: SocketAddr = SocketAddr::from(([0, 0, 0, 0], 80));
    let make_svc = make_service_fn(move |_conn| {
        let client = Arc::clone(&client);
        async move {
            Ok::<_>(service_fn(
                move |mut req| {
                    let client = Arc::clone(&client);
                    async move {
                        println!("proxy {} {}", req.method(), req.uri().path());
                        proxy_crate(&mut req)?;
                        let response = client.request(req).await;
                        println!("body {:?}", &response);
                        response.context("response ")
                    }
                }
            ))
        }
    });

    let _server = Server::bind(&addr).serve(make_svc).await.context(" Run Proxy Server");

    Ok::<()>(())
}