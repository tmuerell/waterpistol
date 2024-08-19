use assets::static_handler;
use axum::extract::State;
use axum::http::{header, StatusCode, Uri};
use controller::{get_config, get_testruns, run_test, update_visibility_status, upload_archive};
use figment::providers::{Format, Serialized, Yaml};
use figment::Figment;
use models::config::AppConfig;
use tokio::io::AsyncReadExt;

use axum::routing::{patch, post};
use axum::{response::IntoResponse, routing::get, Router};
use clap::Parser;

use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tokio::fs::File;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use color_eyre::Result;

pub mod assets;
pub mod controller;
pub mod error;

// Setup the command line interface with clap.
#[derive(Parser, Debug)]
#[clap(name = "waterpistol", about = "A UI for running gatling tests!")]
struct Opt {
    /// set the log level
    #[clap(short = 'l', long = "log", default_value = "debug")]
    log_level: String,

    /// set the listen addr
    #[clap(short = 'a', long = "addr", default_value = "::1")]
    addr: String,

    /// set the listen port
    #[clap(short = 'p', long = "port", default_value = "8080")]
    port: u16,

    /// set the gatling dir to use
    #[clap(long = "data-dir")]
    data_dir: String,
}

const TESTSUITE_NAME : &str = "main";  // TODO: Make this configurable...

pub struct AppState {
    pub data_dir: PathBuf,
    pub result_dir: PathBuf,
    pub app_config: AppConfig,
}

async fn simulations_handler(uri: Uri, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/').to_string();

    if path.starts_with("simulations/") {
        path = path.replace("simulations/", "");
    }

    let p = state.result_dir.join(&path);

    if !p.exists() {
        return (StatusCode::NOT_FOUND, "Not found").into_response();
    }

    let p = if p.is_dir() { p.join("index.html") } else { p };

    if p.exists() {
        let mut buf: Vec<u8> = vec![];
        let mut f = File::open(&p).await.unwrap();
        f.read_to_end(&mut buf).await.unwrap();
        let mime = mime_guess::from_path(&p).first_or_octet_stream();
        (StatusCode::OK, [(header::CONTENT_TYPE, mime.as_ref())], buf).into_response()
    } else {
        return (StatusCode::NOT_FOUND, "Not found").into_response();
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let opt = Opt::parse();

    // Setup logging & RUST_LOG from args
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", format!("{},hyper=info,mio=info", opt.log_level))
    }
    // enable console logging
    tracing_subscriber::fmt::init();

    let config: AppConfig = Figment::from(Serialized::defaults(AppConfig::default()))
        .merge(Yaml::file("waterpistol.yml"))
        .extract()?;

    let testsuite_dir = PathBuf::from(&opt.data_dir);

    let shared_state = Arc::new(AppState {
        data_dir: testsuite_dir.clone(),
        result_dir: testsuite_dir.join(TESTSUITE_NAME).join("target/gatling"),
        app_config: config,
    });

    let app = Router::new()
        .route("/api/upload", post(upload_archive))
        .route("/api/testruns", get(get_testruns))
        .route("/api/testruns/:name", patch(update_visibility_status))
        .route("/api/run", post(run_test))
        .route("/api/config", get(get_config))
        .route("/simulations/*path", get(simulations_handler))
        .fallback_service(get(static_handler))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
        .with_state(shared_state);

    let sock_addr = SocketAddr::from((
        IpAddr::from_str(opt.addr.as_str()).unwrap_or(IpAddr::V6(Ipv6Addr::LOCALHOST)),
        opt.port,
    ));

    log::info!("listening on http://{}", sock_addr);

    axum_server::bind(sock_addr)
        .serve(app.into_make_service())
        .await
        .expect("Unable to start server");

    Ok(())
}
