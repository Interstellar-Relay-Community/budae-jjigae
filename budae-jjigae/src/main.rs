use std::convert::Infallible;
use std::net::SocketAddr;

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper::body::{Body, Incoming};
use hyper_util::rt::tokio::{TokioIo, TokioTimer};
use sonic_rs::{JsonValueTrait, pointer, Value};
use tokio::net::{TcpListener, TcpStream};
use regex::Regex;

// An async function that consumes a request, does nothing with it and returns a
// response.
async fn hello(req: Request<Incoming>) -> Result<Response<Full<Bytes>>, hyper::Error> {
    let max = req.body().size_hint().upper().unwrap_or(u64::MAX);
    if max > 1024 * 1024 * 2 {
        let mut resp = Response::new(Full::new(Bytes::from("Request too big!")));
        *resp.status_mut() = hyper::StatusCode::PAYLOAD_TOO_LARGE;
        return Ok(resp);
    }

    let (parts, incoming) = req.into_parts();

    let body_bytes = incoming.collect().await?.to_bytes();

    let body: sonic_rs::Value = match sonic_rs::from_slice(&body_bytes) {
        Ok(x) => x,
        Err(e) => {
            let mut response = Response::new(Full::new(Bytes::from("bad!")));
            *(response.status_mut()) = StatusCode::BAD_REQUEST;

            return Ok(response);
        }
    };

    if let Some(content) = body.pointer(pointer!["content"]) {
        if let Some(content_str) = content.as_str() {
            // TODO: Move it outside of this function
            let re = Regex::new(r"mastodon-japan\.net\/@ap12").unwrap();

            if let Some(_) = re.captures(content_str) {
                // Spam!!
                tracing::info!("Spam killed: {}", content_str);
                let mut response = Response::new(Full::new(Bytes::from("bad!")));
                *(response.status_mut()) = StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS;

                return Ok(response);
            }
        }
    }

    let stream = TcpStream::connect("localhost:3001").await.unwrap();
    let io = TokioIo::new(stream);
    tracing::info!("Requesting..");
    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;
    tracing::info!("Request done.");

    tokio::task::spawn(async move {
        if let Err(e) = conn.await {
            tracing::error!("Connection failed: {:?}", e);
        }
    });

    let req = Request::from_parts(parts, Full::new(body_bytes));
    tracing::info!("Sending req!");
    let res = sender.send_request(req).await.unwrap();

    let (resp_parts, resp_body) = res.into_parts();
    let payload = resp_body.collect().await?.to_bytes();

    tracing::info!("Returning!");

    Ok(Response::from_parts(resp_parts, Full::new(payload)))
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt().init();

    // This address is localhost
    let addr: SocketAddr = ([127, 0, 0, 1], 3000).into();

    // Bind to the port and listen for incoming TCP connections
    let listener = TcpListener::bind(addr).await?;
    println!("Listening on http://{}", addr);
    loop {
        let (tcp, _) = listener.accept().await?;
        let io = TokioIo::new(tcp);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(hello))
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}
