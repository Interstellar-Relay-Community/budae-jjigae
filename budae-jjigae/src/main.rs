use std::sync::Arc;

use tokio::net::TcpStream;

use http_body_util::{BodyExt, Full};
use hyper_util::rt::TokioIo;

use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::Router;

use sonic_rs::{pointer, JsonValueTrait};

use tracing_subscriber::EnvFilter;

use clap::Parser;

mod mrf;

#[derive(Parser, Clone, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    backend: String,
    #[arg(long, default_value = "3000")]
    listen_port: u16,
    #[arg(long, default_value = "5")]
    max_new_reply_cnt: i32,
    #[arg(long)]
    spam_in_teapot: bool,
    #[arg(long)]
    silent_mode: bool,
    #[arg(long, help = "deprecated. Doesn't do anything for now.")]
    enable_hc: bool,
    #[arg(
        long,
        help = "Log every object in log to help spam filter debug. INTRODUCES PRIVACY ISSUE"
    )]
    peeping_tom: bool,
}

#[derive(Clone)]
struct AppState {
    args: Arc<Args>,
    filter_config: Arc<mrf_policy::FilterConfig>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    tracing::info!(
        "ActivityPub Spam Filter v{} ~ Budae Jjigae (Johnson Tang Edition) ~ ",
        env!("CARGO_PKG_VERSION")
    );

    let args = Arc::new(Args::parse());

    let app_state = AppState {
        args: args.clone(),
        filter_config: Arc::new(mrf_policy::FilterConfig {
            max_new_reply_cnt: args.max_new_reply_cnt,
        }),
    };

    let app = Router::new()
        // handle inboxes
        .route("/healthcheck", get(healthcheck))
        .route("/*path", post(handler))
        //.route("/inbox", post(handler))
        //.route("/users/:user/inbox", post(handler))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", args.listen_port))
        .await
        .unwrap();

    tracing::info!("Listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

async fn healthcheck() -> Result<Response, StatusCode> {
    Ok(Response::new("Health OK!".into()))
}

async fn handler(
    State(app_state): State<AppState>,
    req: Request,
) -> Result<impl IntoResponse, StatusCode> {
    let args = app_state.args;
    let filter_config = app_state.filter_config;

    let (parts, body) = req.into_parts();

    // TODO: Remove unwrap
    let body_bytes = body.into_data_stream().collect().await.unwrap().to_bytes();

    let body: sonic_rs::Value = match sonic_rs::from_slice(&body_bytes) {
        Ok(x) => x,
        Err(e) => {
            tracing::warn!("Failed to process body: {:?}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    if args.peeping_tom {
        tracing::info!("Request body: {}", body.to_string());
    }

    // Extract object
    let object = match body.pointer(pointer!["type"]).as_str() {
        Some("Announce") => match body.pointer(pointer!["object"]).as_str() {
            Some(obj_ref) => {
                tracing::warn!("Announce support is not fully implemented!");
                tracing::warn!("If something goes wrong, please report to Interstellar Team!");
                tracing::warn!("Announce object reference: {}", obj_ref);
                None
            }
            None => body.pointer(pointer!["object", "object"]),
        },
        Some("Create") => body.pointer(pointer!["object"]),
        Some("Update") => body.pointer(pointer!["object"]),
        Some("Delete") => None,
        Some("Follow") => None,
        Some("Like") => None,
        Some("Block") => None,
        Some("Undo") => None,
        Some("View") => None,
        Some("Add") => None,
        Some("Remove") => None,
        Some(x) => {
            tracing::warn!("Unknown type: {}. Please report to Interstellar Team!", x);
            tracing::warn!("Payload: {}", body.to_string());
            body.pointer(pointer!["object"])
        }
        None => {
            tracing::warn!("Cannot determine activity type!");
            tracing::warn!("Report this activity to Interstellar Team!");
            tracing::warn!("Payload: {}", body.to_string());
            Some(&body)
        }
    };

    if let Some(obj) = object {
        match mrf::filter(obj, &filter_config).await {
            Ok(_) => {}
            Err(_) => {
                return match args.spam_in_teapot {
                    true => Err(StatusCode::IM_A_TEAPOT),
                    false => Err(StatusCode::CREATED),
                }
            }
        }
    }

    if !args.silent_mode {
        tracing::info!("Passed all filter. Relaying message to the backend.");
    }

    // TODO: Pool using bb-8
    let stream = TcpStream::connect(&args.backend)
        .await
        .map_err(|e| {
            tracing::error!("Failed to do open socket: {:?}", e);
            e
        })
        .unwrap(); // TODO: FIXME

    let io = TokioIo::new(stream);
    tracing::debug!("Handshaking..");
    let (mut sender, conn) = hyper::client::conn::http1::handshake(io)
        .await
        .map_err(|e| {
            tracing::error!("Failed to do handshake: {:?}", e);
            e
        })
        .unwrap(); // TODO: FIXME

    tracing::debug!("Handshake done.");

    tokio::task::spawn(async move {
        if let Err(e) = conn.await {
            tracing::error!("Connection failed: {:?}", e);
        }
    });

    let req = Request::from_parts(parts, Full::new(body_bytes));
    tracing::debug!("Sending req!");
    let res = sender
        .send_request(req)
        .await
        .map_err(|e| {
            tracing::error!("Failed to send request: {:?}", e);
            e
        })
        .unwrap();

    let (resp_parts, resp_body) = res.into_parts();
    let payload = resp_body
        .collect()
        .await
        .map_err(|e| {
            tracing::error!("Failed to collect response body: {:?}", e);
            e
        })
        .unwrap()
        .to_bytes();

    tracing::debug!("Response: {:?}. {:?}", resp_parts, payload);

    //let resp = Response::new("");

    Ok(resp_parts.status)

    //Ok(Response::from_parts(resp_parts, "".into()))
}
