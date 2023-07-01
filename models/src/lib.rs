use serde::{Deserialize, Serialize};

pub mod config;
pub mod report;

#[derive(Serialize, Deserialize, Debug)]
pub struct Testrun {
    pub creation_date: String,
    pub name: String,
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

#[derive(Deserialize, Serialize)]
pub struct RunTestParam {
    pub description: String,
    pub factor: u64,
    pub duration: u64,
    pub scenario: String,
    pub url: String,
}
