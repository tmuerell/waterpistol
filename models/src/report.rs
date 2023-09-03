use std::{
    collections::HashMap,
    error::Error,
    io::{BufRead, BufReader},
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Default, Clone, Eq)]
pub enum TestrunStatus {
    #[default]
    Unknown,
    Running,
    Done,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Default, Clone, Eq)]
pub struct TestrunData {
    pub datum: Option<DateTime<Utc>>,
    pub status: TestrunStatus,
    pub custom_params: HashMap<String, String>,
    pub statistics: Option<GatlingReport>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Eq)]
pub struct GatlingReport {
    pub name: String,
    pub version: String,
    pub requests_ok: u64,
    pub requests_nok: u64,
    pub request_stats: Vec<RequestStats>,
    pub user_stats: Vec<UserStats>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Eq)]
pub struct RequestStats {
    pub name: String,
    pub avg: u64,
    pub max: u64,
    pub min: u64,
    pub p95: u64,
    pub count: u64,
    pub errors: Vec<RequestErrorStats>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Eq)]
pub struct UserStats {
    pub name: String,
    pub count: u64,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Eq)]
pub struct RequestErrorStats {
    pub name: String,
    pub count: u64,
}

impl GatlingReport {
    pub fn from_file(stream: &mut dyn BufRead) -> Result<Self, Box<dyn Error>> {
        let rdr = BufReader::new(stream);

        let mut iter = rdr.lines();

        let header = iter.next().ok_or("No header")??;
        let mut header = header.split('\t');

        let name = header.nth(4).ok_or("No name")?;
        let version = header.nth(0).ok_or("No version")?;
        let mut requests_ok = 0;
        let mut requests_nok = 0;

        let mut user_count: HashMap<String, u64> = HashMap::new();
        let mut request_map: HashMap<String, Vec<u64>> = HashMap::new();
        let mut error_map: HashMap<String, HashMap<String, u64>> = HashMap::new();

        for result in iter {
            // The iterator yields Result<StringRecord, Error>, so we check the
            // error here.
            let record = result?;
            let mut record = record.split('\t');

            let action = record.nth(0).unwrap();

            if action == "REQUEST" {
                let _ = record.next().ok_or("missing field _")?;
                let name = record.next().ok_or("missing field name")?;
                let start: u64 = record.next().ok_or("missing field start")?.parse()?;
                let end: u64 = record.next().ok_or("missing field end")?.parse()?;
                let delta = end - start;
                let request_result = record.next().ok_or("missing field request_result")?;
                if request_result == "OK" {
                    requests_ok += 1;
                } else {
                    requests_nok += 1;
                    error_map
                        .entry(name.to_string())
                        .and_modify(|m| {
                            m.entry(request_result.to_string())
                                .and_modify(|c| *c += 1)
                                .or_insert(1);
                        })
                        .or_insert_with(|| {
                            let mut m = HashMap::new();
                            m.insert(request_result.to_string(), 1);
                            m
                        });
                }
                request_map
                    .entry(name.to_string())
                    .and_modify(|v| v.push(delta))
                    .or_insert_with(|| vec![delta]);
            }
            if action == "USER" {
                let journey = record.next().ok_or("missing field journey")?;
                let status = record.next().ok_or("missing field status")?;
                if status == "START" {
                    user_count
                        .entry(journey.to_string())
                        .and_modify(|c| *c += 1)
                        .or_insert(1);
                }
            }
        }

        let request_stats: Vec<RequestStats> = request_map
            .iter_mut()
            .map(|(k, v)| {
                v.sort();

                let mut aiter = v.iter();
                let mut max = *aiter.next().unwrap();
                let mut min = max;
                let mut sum = max;
                let l = v.len();

                for a in aiter {
                    if *a > max {
                        max = *a;
                    }
                    if *a < min {
                        min = *a;
                    }
                    sum += a;
                }

                let pi = (l * 95) / 100;
                let p95 = *v.iter().nth(pi).unwrap_or(&0);

                RequestStats {
                    name: k.to_string(),
                    min: min,
                    max: max,
                    avg: sum / l as u64,
                    p95: p95,
                    count: l as u64,
                    errors: error_map
                        .get(k)
                        .map(|e| {
                            e.into_iter()
                                .map(|(k, v)| RequestErrorStats {
                                    name: k.to_string(),
                                    count: *v,
                                })
                                .collect()
                        })
                        .unwrap_or_default(),
                }
            })
            .collect();

        Ok(GatlingReport {
            name: name.to_string(),
            version: version.to_string(),
            requests_ok: requests_ok,
            requests_nok: requests_nok,
            request_stats: request_stats,
            user_stats: user_count
                .into_iter()
                .map(|(k, v)| UserStats { name: k, count: v })
                .collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use crate::report::{GatlingReport, RequestErrorStats, RequestStats, UserStats};

    #[test]
    fn it_works() {
        let string = "\
RUN\tSimulation\tsimulation\t1680605882911\tfoobar\t3.9.2
USER\tVisit Homepage\tSTART\t1680605883404
USER\tVisit Homepage\tSTART\t1680605883428
USER\tVisit Homepage\tSTART\t1680605883454
REQUEST\t\thome_page\t1680605883428\t1680605883518\tOK\t 
REQUEST\t\thome_page\t1680605883400\t1680605883512\tOK";

        let mut r = BufReader::new(string.as_bytes());

        let record = GatlingReport::from_file(&mut r).unwrap();

        let expected = GatlingReport {
            name: "foobar".to_string(),
            version: "3.9.2".to_string(),
            requests_ok: 2,
            requests_nok: 0,
            request_stats: vec![RequestStats {
                name: "home_page".into(),
                avg: 101,
                max: 112,
                min: 90,
                p95: 112,
                count: 2,
                errors: vec![],
            }],
            user_stats: vec![UserStats {
                name: "Visit Homepage".into(),
                count: 3,
            }],
        };

        assert_eq!(record, expected);
    }

    #[test]
    fn it_works_with_errors() {
        let string = "\
RUN\tSimulation\tsimulation\t1680605882911\tfoobar\t3.9.2
USER\tVisit Homepage\tSTART\t1680605883404
USER\tVisit Homepage\tSTART\t1680605883428
USER\tVisit Homepage\tSTART\t1680605883454
REQUEST\t\thome_page\t1680605883428\t1680605883518\tBLAH\t 
REQUEST\t\thome_page\t1680605883400\t1680605883512\tOK";

        let mut r = BufReader::new(string.as_bytes());

        let record = GatlingReport::from_file(&mut r).unwrap();

        let expected = GatlingReport {
            name: "foobar".to_string(),
            version: "3.9.2".to_string(),
            requests_ok: 1,
            requests_nok: 1,
            request_stats: vec![RequestStats {
                name: "home_page".into(),
                avg: 101,
                max: 112,
                min: 90,
                p95: 112,
                count: 2,
                errors: vec![RequestErrorStats {
                    name: "BLAH".into(),
                    count: 1,
                }],
            }],
            user_stats: vec![UserStats {
                name: "Visit Homepage".into(),
                count: 3,
            }],
        };

        assert_eq!(record, expected);
    }
}
