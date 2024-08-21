use std::collections::HashMap;

use report::TestrunVisibilityStatus;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::base64::Base64;

pub mod config;
pub mod report;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Testrun {
    pub creation_date: String,
    pub name: String,
    pub progress: Option<u64>,
    pub data: Option<report::TestrunData>,
}

impl Eq for Testrun {}

impl Ord for Testrun {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.creation_date.cmp(&other.creation_date)
    }
}

impl PartialEq for Testrun {
    fn eq(&self, other: &Self) -> bool {
        self.creation_date == other.creation_date
    }
}

impl PartialOrd for Testrun {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.creation_date.partial_cmp(&other.creation_date)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Testsuite {
    pub name: String,
}

#[derive(Deserialize, Serialize)]
pub struct RunTestParam {
    pub description: String,
    pub custom_params: HashMap<String, String>,
}

#[derive(Deserialize, Serialize)]
pub struct UpdateTestrunData {
    pub visibility_status: Option<TestrunVisibilityStatus>,
}

#[serde_as]
#[derive(Deserialize, Serialize)]
pub struct UploadTestsuite {
    pub file_name : String,
    pub mime_type: String,
    #[serde_as(as = "Base64")]
    pub data : Vec<u8>,
}

#[derive(Deserialize, Serialize, PartialEq)]
pub enum SystemStatus {
    Healthy,
    Unhealthy,
}

#[derive(Deserialize, Serialize)]
pub struct SystemStatusResponse {
    pub overall: SystemStatus,
    pub maven_output: Option<String>,
    pub java_version: Option<String>,
}