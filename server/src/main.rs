use axum::body::{boxed, Full};
use axum::extract::State;
use axum::http::{header, StatusCode, Uri};
use axum::Json;
use chrono::{DateTime, Local, Utc};
use figment::providers::{Format, Serialized, Yaml};
use figment::Figment;
use log::warn;
use models::config::AppConfig;
use tokio::io::AsyncReadExt;

use axum::response::Response;
use axum::routing::post;
use axum::{response::IntoResponse, routing::get, Router};
use clap::Parser;
use models::report::{GatlingReport, TestrunData, TestrunStatus};
use models::{RunTestParam, Testrun};

use rust_embed::RustEmbed;
use std::io::BufReader;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tokio::fs::{self, read_dir, remove_dir_all, rename, File};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::info;
use uuid::Uuid;

use color_eyre::Result;

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
    #[clap(long = "testsuite-dir")]
    testsuite_dir: String,
}

struct AppState {
    pub testsuite_dir: PathBuf,
    pub result_dir: PathBuf,
    pub app_config: AppConfig,
}

#[derive(RustEmbed)]
#[folder = "../dist/"]
struct DistAsset;

pub struct StaticFile<T>(pub T);

impl<T> IntoResponse for StaticFile<T>
where
    T: Into<String>,
{
    fn into_response(self) -> Response {
        let path = self.0.into();

        match DistAsset::get(path.as_str()) {
            Some(content) => {
                let body = boxed(Full::from(content.data));
                let mime = mime_guess::from_path(path).first_or_octet_stream();
                Response::builder()
                    .header(header::CONTENT_TYPE, mime.as_ref())
                    .body(body)
                    .unwrap()
            }
            None => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(boxed(Full::from("404")))
                .unwrap(),
        }
    }
}

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/').to_string();

    if path == "" {
        path = "index.html".to_string();
    }
    info!("Request for {}", path);

    StaticFile(path)
}

async fn simulations_handler(uri: Uri, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/').to_string();

    if path.starts_with("simulations/") {
        path = path.replace("simulations/", "");
    }

    let p = state.result_dir.join(&path);

    if !p.exists() {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(boxed(Full::from("404")))
            .unwrap();
    }

    let p = if p.is_dir() { p.join("index.html") } else { p };

    if p.exists() {
        let mut buf: Vec<u8> = vec![];
        let mut f = File::open(&p).await.unwrap();
        f.read_to_end(&mut buf).await.unwrap();
        let mime = mime_guess::from_path(&p).first_or_octet_stream();
        Response::builder()
            .header(header::CONTENT_TYPE, mime.as_ref())
            .body(boxed(Full::from(buf)))
            .unwrap()
    } else {
        Response::builder()
            .status(404)
            .body(boxed(Full::from("Not found")))
            .unwrap()
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

    let shared_state = Arc::new(AppState {
        testsuite_dir: PathBuf::from(&opt.testsuite_dir),
        result_dir: PathBuf::from(&opt.testsuite_dir).join("target/gatling"),
        app_config: config,
    });

    let app = Router::new()
        .route("/api/testruns", get(get_testruns))
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

    axum::Server::bind(&sock_addr)
        .serve(app.into_make_service())
        .await
        .expect("Unable to start server");

    Ok(())
}

async fn read_data_file(data_file: &PathBuf) -> Result<TestrunData> {
    let contents = fs::read(&data_file).await?;
    let contents = String::from_utf8_lossy(&contents);
    Ok(serde_json::from_str(&contents)?)
}

async fn get_config(State(state): State<Arc<AppState>>) -> Json<AppConfig> {
    Json(state.app_config.clone())
}

async fn get_testruns(State(state): State<Arc<AppState>>) -> Json<Vec<Testrun>> {
    let mut res: Vec<Testrun> = vec![];
    let mut x = read_dir(&state.result_dir).await.unwrap();
    loop {
        match x.next_entry().await {
            Ok(Some(e)) => {
                if e.path().is_dir() {
                    let data_file = e.path().join("testrun-data.json");
                    let data: TestrunData = match read_data_file(&data_file).await {
                        Ok(df) => df,
                        Err(err) => {
                            warn!(
                                "Cannot read data file {:?} because of {:?}",
                                &data_file, err
                            );
                            let simulation_log_file = e.path().join("simulation.log");
                            let data = if simulation_log_file.exists() {
                                let f = std::fs::File::open(&simulation_log_file).unwrap();
                                let report = GatlingReport::from_file(&mut BufReader::new(&f)).ok();

                                TestrunData {
                                    statistics: report,
                                    ..Default::default()
                                }
                            } else {
                                Default::default()
                            };
                            {
                                let mut f = File::create(&data_file).await.unwrap();
                                f.write_all(serde_json::to_string(&data).unwrap().as_bytes())
                                    .await
                                    .unwrap();
                            }
                            data
                        }
                    };
                    let datetime: String = match e.metadata().await.and_then(|x| x.created()) {
                        Ok(t) => DateTime::<Local>::from(t).to_rfc3339(),
                        Err(_) => "1970-01-01T12:00:00".to_string(),
                    };
                    res.push(Testrun {
                        creation_date: datetime,
                        name: e.file_name().to_owned().to_string_lossy().to_string(),
                        data: Some(data),
                    })
                }
            }
            Ok(None) => break,
            Err(_) => break,
        }
    }
    res.sort();
    res.reverse();
    Json(res)
}

async fn run_test(
    State(state): State<Arc<AppState>>,
    test_param: Json<RunTestParam>,
) -> impl IntoResponse {
    tokio::spawn(async move {
        let app_config = &state.app_config;

        info!("Starting simulation");

        let uuid = Uuid::new_v4();
        let uuid = format!("{}", uuid);

        let target_test_dir = state.result_dir.join(&uuid);
        let temp_test_dir = state.testsuite_dir.join(&uuid);

        let mut cmd = Command::new("mvn");

        cmd.arg("gatling:test")
            .arg(format!(
                "-Dgatling.simulationClass={}",
                app_config.simulation.simulation_class
            ))
            .arg(format!(
                "-Dgatling.runDescription={}",
                &test_param.description
            ))
            .arg(format!(
                "-Dgatling.resultsFolder={}",
                &temp_test_dir.as_os_str().to_string_lossy()
            ));

        for param in &app_config.simulation.params {
            if let Some(v) = test_param.custom_params.get(&param.name) {
                cmd.arg(format!("-D{}={}", param.name, v));
            }
        }

        cmd.current_dir(&state.testsuite_dir);

        let output = cmd.status().await.unwrap();

        info!(?output, "Output");

        let mut x = read_dir(&temp_test_dir).await.unwrap();

        loop {
            match x.next_entry().await {
                Ok(Some(e)) => {
                    if e.path().is_dir() {
                        rename(e.path(), &target_test_dir).await.unwrap();

                        let report = {
                            let f = std::fs::File::open(target_test_dir.join("simulation.log"))
                                .unwrap();
                            GatlingReport::from_file(&mut BufReader::new(&f)).unwrap()
                        };

                        let data = TestrunData {
                            datum: PathBuf::from(target_test_dir.join("simulation.log"))
                                .metadata()
                                .and_then(|m| m.created())
                                .map(|t| DateTime::<Utc>::from(t))
                                .ok(),
                            status: TestrunStatus::Done,
                            custom_params: test_param.custom_params.clone(),
                            statistics: Some(report),
                        };

                        {
                            let mut f = File::create(target_test_dir.join("testrun-data.json"))
                                .await
                                .unwrap();
                            f.write_all(serde_json::to_string(&data).unwrap().as_bytes())
                                .await
                                .unwrap();
                        }

                        break;
                    }
                }
                Ok(None) => break,
                Err(_) => break,
            }
        }

        remove_dir_all(&temp_test_dir).await.unwrap();

        info!("Simulation finished.")
    });

    "Ok"
}
