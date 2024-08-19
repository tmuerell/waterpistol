use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Local, Utc};
use error::Error;
use flate2::read::GzDecoder;
use log::warn;
use models::config::AppConfig;

use axum::response::IntoResponse;
use models::report::{GatlingReport, TestrunData, TestrunStatus, TestrunVisibilityStatus};
use models::{RunTestParam, Testrun, UpdateTestrunData, UploadTestsuite};
use tar::Archive;

use std::io::{BufReader, Write};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::{self, create_dir_all, read_dir, read_to_string, remove_dir_all, rename, File};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tracing::info;
use uuid::Uuid;

use color_eyre::Result;

use crate::{error, AppState, TESTSUITE_NAME};

async fn read_data_file(data_file: &PathBuf) -> error::Result<TestrunData> {
    let contents = fs::read(&data_file).await?;
    let contents = String::from_utf8_lossy(&contents);
    Ok(serde_json::from_str(&contents)?)
}

pub async fn get_config(State(state): State<Arc<AppState>>) -> error::Result<Json<AppConfig>> {
    Ok(Json(state.app_config.clone()))
}

pub async fn get_testruns(State(state): State<Arc<AppState>>) -> error::Result<Json<Vec<Testrun>>> {
    let mut res: Vec<Testrun> = vec![];

    let Ok(dir) = read_dir(&state.result_dir).await else {
        return Err(Error::NotFound);
    };

    let mut x = dir;
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
                    if data.visibility_status != TestrunVisibilityStatus::Hidden {
                        res.push(Testrun {
                            creation_date: datetime,
                            name: e.file_name().to_owned().to_string_lossy().to_string(),
                            progress: None,
                            data: Some(data),
                        })
                    }
                }
            }
            Ok(None) => break,
            Err(_) => break,
        }
    }

    res.sort();

    let mut x = read_dir(&state.data_dir).await.unwrap();
    loop {
        match x.next_entry().await {
            Ok(Some(e)) => {
                if e.path().is_dir()
                    && e.path()
                        .file_name()
                        .map(|e| e.to_string_lossy().starts_with("running-"))
                        .unwrap_or(false)
                {
                    let mut sum = None;
                    info!("{:?}", e.path());
                    let data_file = e.path().join("testrun-data.json");
                    {
                        let mut x = read_dir(&e.path()).await.unwrap();
                        loop {
                            match x.next_entry().await {
                                Ok(Some(e)) => {
                                    if e.path().is_dir() {
                                        let f = e.path().join("simulation.log");
                                        if f.exists() {
                                            sum = read_to_string(f).await.ok().map(|c| {
                                                c.lines()
                                                    .filter(|l| {
                                                        l.contains("USER") && l.contains("START")
                                                    })
                                                    .count()
                                                    as u64
                                            });
                                        }
                                    }
                                }
                                Ok(None) => break,
                                Err(_) => break,
                            }
                        }
                    }
                    if let Ok(d) = read_data_file(&data_file).await {
                        res.push(Testrun {
                            creation_date: "".into(),
                            name: e.file_name().to_owned().to_string_lossy().to_string(),
                            progress: sum,
                            data: Some(d),
                        })
                    }
                }
            }
            Ok(None) => break,
            Err(_) => break,
        }
    }

    res.reverse();
    Ok(Json(res))
}

pub async fn update_visibility_status(
    Path(name): Path<String>,
    State(state): State<Arc<AppState>>,
    param: Json<UpdateTestrunData>,
) -> error::Result<impl IntoResponse> {
    let x = PathBuf::from(&state.result_dir).join(name);

    if !x.exists() {
        return Err(Error::NotFound);
    }

    let data_file = x.join("testrun-data.json");

    if !x.exists() {
        return Err(Error::NotFound);
    }

    let mut d = read_data_file(&data_file).await?;
    d.visibility_status = param.visibility_status.as_ref().unwrap().clone();

    {
        let mut f = File::create(&data_file).await.unwrap();
        f.write_all(serde_json::to_string(&d).unwrap().as_bytes())
            .await
            .unwrap();
    }
    Ok((StatusCode::OK, "OK").into_response())
}

pub async fn run_test(
    State(state): State<Arc<AppState>>,
    test_param: Json<RunTestParam>,
) -> impl IntoResponse {
    tokio::spawn(async move {
        let app_config = &state.app_config;

        info!("Starting simulation");

        let uuid = Uuid::new_v4();
        let uuid = format!("{}", uuid);

        let target_test_dir = state.result_dir.join(&uuid);
        let temp_test_dir = state.result_dir.join(&format!("running-{}", uuid));

        create_dir_all(&temp_test_dir).await.unwrap();

        {
            let data = TestrunData {
                datum: None,
                status: TestrunStatus::Running,
                custom_params: test_param.custom_params.clone(),
                statistics: None,
                ..Default::default()
            };

            let mut f = File::create(temp_test_dir.join("testrun-data.json"))
                .await
                .unwrap();
            f.write_all(serde_json::to_string(&data).unwrap().as_bytes())
                .await
                .unwrap();
        }

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

        cmd.current_dir(&state.data_dir.join(TESTSUITE_NAME));

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
                            ..Default::default()
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

pub async fn upload_archive(
    State(state): State<Arc<AppState>>,
    upload: Json<UploadTestsuite>,
) -> error::Result<impl IntoResponse> {
    let f = state.data_dir.join("tempfile.tar.gz");

    let file_name = TESTSUITE_NAME;
    let mime_type = upload.mime_type.clone();
    let unpack_dir = state.data_dir.join(TESTSUITE_NAME);

    if mime_type == "application/gzip" || mime_type == "application/x-gzip" {
    {
        let mut file = File::create(&f).await?;
        file.write(&upload.data).await?;
        file.flush().await?;
    }

    {
        let tar_gz = std::fs::File::open(&f)?;
        let tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);
        archive
        .entries()?
        .filter_map(|e| e.ok())
        .map(|mut entry| -> Result<PathBuf> {
            let path = entry.path()?;
            let mut components = path.components();
            components.next();
            let path = unpack_dir.join(components.as_path());
            entry.unpack(&path)?;
            Ok(path)
        })
        .filter_map(|e| e.ok())
        .for_each(|x| println!("> {}", x.display()));
    }

    let _ = fs::remove_file(&f).await;

    Ok((StatusCode::OK, "").into_response())
} else {
    Ok((StatusCode::UNPROCESSABLE_ENTITY, format!("Archive was not a proper archive. Mime type was {}, need application/gzip.", mime_type)).into_response())
}
}
