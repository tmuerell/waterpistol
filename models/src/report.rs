use std::{io::{Read, BufRead, BufReader}, error::Error};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Default)]
pub enum TestrunStatus {
    #[default]
    Unknown,
    Running,
    Done,
}


#[derive(Serialize, Deserialize, PartialEq, Debug, Default)]
pub struct TestrunData {
    pub status : TestrunStatus,
    pub scenario : String,
    pub factor : u64,
    pub duration : u64,
    pub statistics : Option<GatlingReport>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct GatlingReport {
    pub name : String,
    pub version : String,
    pub requests_ok : u128,
    pub requests_nok : u128,
}

impl GatlingReport {
    pub fn from_file(stream : &mut dyn BufRead) -> Result<Self, Box<dyn Error>> {

        let rdr = BufReader::new(stream);

        let mut iter = rdr.lines();

        let header = iter.next().unwrap()?;
        let mut header = header.split('\t');

        let name = header.nth(4).unwrap();
        let version = header.nth(0).unwrap();
        let mut requests_ok = 0;
        let mut requests_nok = 0;

        for result in iter {
            // The iterator yields Result<StringRecord, Error>, so we check the
            // error here.
            let record = result?;
            let mut record = record.split('\t');

            let action = record.nth(0).unwrap();

            if action == "REQUEST" {
                let request_result = record.nth(4).unwrap();
                if request_result == "OK" {
                    requests_ok += 1;
                } else {
                    requests_nok += 1;
                }
            }
        }

        Ok(GatlingReport {
            name: name.to_string(),
            version: version.to_string(),
            requests_ok: requests_ok,
            requests_nok: requests_nok,
        })
}
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use crate::report::GatlingReport;

    #[test]
    fn it_works() {
        let string = "\
RUN\tSimulation\tsimulation\t1680605882911\tfoobar\t3.9.2
USER\tVisit Homepage\tSTART\t1680605883404
USER\tVisit Homepage\tSTART\t1680605883428
USER\tVisit Homepage\tSTART\t1680605883454
REQUEST\t\thome_page\t1680605883428\t1680605883518\tOK\t 
REQUEST\t\thome_page\t1680605883400\t1680605883512\tOK\t 
USER\tPrepaid User\tSTART\t1680605883562";

        let mut r = BufReader::new(string.as_bytes());

        let record = GatlingReport::from_file(&mut r).unwrap();

        let expected = GatlingReport {
            name: "foobar".to_string(),
            version: "3.9.2".to_string(),
            requests_ok: 2,
            requests_nok: 0
        };

        assert_eq!(record, expected);
    }
}