use fs::read_to_string;
use std::net::SocketAddr;

use std::fs;
use std::path::Path;
use directories::ProjectDirs;
use bytes::Bytes;
use clap::Parser;
use http_body_util::{BodyExt, Full};
use hyper::body::{Body, Incoming};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::tokio::TokioIo;
use regex::Regex;
use sonic_rs::{pointer, JsonContainerTrait, JsonValueTrait};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};

use once_cell::sync::Lazy;
fn path_exists<P: AsRef<Path>>(path: P) -> bool {
    return path.as_ref().exists();
}

fn default_filters() -> Vec<Regex> {
    let re = vec![
        // GTUBE
        Regex::new(
            r"XJS\*C4JDBQADN1\.NSBN3\*2IDNEN\*GTUBE-STANDARD-ANTI-UBE-TEST-ACTIVITYPUB\*C\.34X",
        ).unwrap(),
        // Sorry, @ap12, but they are using your name in spam
        Regex::new(r"mastodon-japan\.net\/@ap12").unwrap(),
        // Would you kindly stop spamming in Korean?
        Regex::new(r"한국괴물군").unwrap(),
        // Fucking discord.
        Regex::new(r"discord.gg\/ctkpaarr").unwrap(),
    ];
    return re;
}

static FILTERS: Lazy<Arc<Vec<Regex>>> = Lazy::new(|| {
    if let Some(config) = ProjectDirs::from("org","Interstellar-Relay-Community","BudaeJjigae") {
        if path_exists(config.config_dir().join("filters.txt")) {
            let re = read_to_string(config.config_dir().join("filters.txt"))
                .unwrap()
                .split('\n')
                .map(|s| Regex::new(s).unwrap()).by_ref().collect::<Vec<_>>();

            Arc::new(re)
        } else {
            eprintln!("filters.txt not found. Using default filters.");
            Arc::new(default_filters())
        }
    } else {
        eprintln!("Could not find config directory, using default filters.");
        Arc::new(default_filters())
    }
});

#[derive(Parser, Clone, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    backend: String,
    #[arg(long, default_value = "5")]
    max_new_reply_cnt: i32,
    #[arg(long, default_value = "true")]
    silent_mode: bool,

}

// An async function that consumes a request, does nothing with it and returns a
// response.
async fn hello(
    req: Request<Incoming>,
    args: &Args,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    let max = req.body().size_hint().upper().unwrap_or(u64::MAX);
    if max > 1024 * 1024 * 2 {
        let mut resp = Response::new(Full::new(Bytes::from("Request too big!")));
        *resp.status_mut() = hyper::StatusCode::PAYLOAD_TOO_LARGE;
        return Ok(resp);
    }

    let status_code = match args.silent_mode {
        true => StatusCode::CREATED,
        false => StatusCode::IM_A_TEAPOT,
    };

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

    // Extract object
    let object = body.pointer(pointer!["object"]);

    // Extract content
    if let Some(content_str) = object.pointer(pointer!["content"]).as_str() {
        // Extract ID
        let object_id_raw = object.pointer(pointer!["id"]);
        let object_id = object_id_raw.as_str().unwrap_or("Unknown ID");

        for re in FILTERS.iter() {
            if let Some(_) = re.captures(content_str) {
                // Spam!!
                tracing::info!("Spam killed - RegExp: {} => {}", object_id, content_str);
                let mut response = Response::new(Full::new(Bytes::from("Spam is not allowed.")));
                *(response.status_mut()) = status_code;

                return Ok(response);
            }
        }

        if let Some(reply_to) = object.pointer(pointer!["inReplyTo"]) {
            if reply_to.is_null() {
                if let Some(tags) = object.pointer(pointer!["tag"]).as_array() {
                    let mut mention_cnt = 0;
                    for tag in tags.iter() {
                        if let Some(tag_type) = tag.pointer(pointer!["type"]).as_str() {
                            if tag_type == "Mention" {
                                mention_cnt += 1;
                            }
                        }
                    }

                    if mention_cnt >= args.max_new_reply_cnt {
                        // Spam!!
                        tracing::info!(
                            "Spam killed - Too many mention: {} => {}",
                            object_id,
                            content_str
                        );
                        let mut response =
                            Response::new(Full::new(Bytes::from("Spam is not allowed.")));
                        *(response.status_mut()) = status_code;

                        return Ok(response);
                    }
                }
            }
        }
    }

    tracing::info!("Spam filter passed. Relaying to the server.");

    // TODO: Make it configurable
    let stream = TcpStream::connect(&args.backend).await.unwrap();
    let io = TokioIo::new(stream);
    tracing::debug!("Handshaking..");
    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;
    tracing::debug!("Handshake done.");

    tokio::task::spawn(async move {
        if let Err(e) = conn.await {
            tracing::error!("Connection failed: {:?}", e);
        }
    });

    let req = Request::from_parts(parts, Full::new(body_bytes));
    tracing::debug!("Sending req!");
    let res = sender.send_request(req).await.unwrap();

    let (resp_parts, resp_body) = res.into_parts();
    let payload = resp_body.collect().await?.to_bytes();

    tracing::debug!("Returning!");

    Ok(Response::from_parts(resp_parts, Full::new(payload)))
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt().init();

    let args = Arc::new(Args::parse());

    // This address is localhost
    let addr: SocketAddr = ([0, 0, 0, 0], 3000).into();

    // Bind to the port and listen for incoming TCP connections
    let listener = TcpListener::bind(addr).await?;
    println!("Listening on http://{}", addr);

    loop {
        let (tcp, _) = listener.accept().await?;
        let io = TokioIo::new(tcp);

        let args = args.clone();

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(|req| hello(req, &args)))
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}
