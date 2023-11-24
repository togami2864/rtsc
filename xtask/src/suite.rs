use std::{fs, io::Write, path::Path};

use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum TestResult {
    Success,
    Failure,
    Panic,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SuiteSummary {
    dir_name: String,
    total_count: f64,
    success: f64,
    failure: f64,
    panic: f64,
    coverage: f64,
}

impl SuiteSummary {
    pub fn new(dir_name: &str, total_count: f64, success: f64, failure: f64, panic: f64) -> Self {
        Self {
            dir_name: dir_name.to_string(),
            total_count,
            success,
            failure,
            panic,
            coverage: (success / total_count) * 100.0,
        }
    }

    pub fn show_and_write_summary<W: Write>(&self, writer: &mut W) {
        let previous_coverage = self.read_previous_run_coverage();
        let msg = format!(
            "{}: {} / {} ({:.2}% +{:.2}%)\n",
            self.dir_name,
            self.success,
            self.total_count,
            self.coverage,
            self.coverage - previous_coverage
        );
        writer
            .write_all(msg.as_bytes())
            .expect("Unable to write summary");
        self.write_summary();
    }

    pub fn write_summary(&self) {
        let json_output = serde_json::to_string_pretty(&self).unwrap();
        let path = format!(
            "{}/summary/{}.json",
            env!("CARGO_MANIFEST_DIR"),
            self.dir_name
        );
        fs::write(path, json_output).expect("Unable to write summary file");
    }

    pub fn read_previous_run_coverage(&self) -> f64 {
        let path = format!(
            "{}/summary/{}.json",
            env!("CARGO_MANIFEST_DIR"),
            self.dir_name
        );
        let summery = fs::read_to_string(path).expect("Unable to read file");
        let previous_summary: SuiteSummary = serde_json::from_str(&summery).unwrap();
        previous_summary.coverage()
    }

    pub fn coverage(&self) -> f64 {
        self.coverage
    }
}

pub trait TestSuite {
    fn new() -> Self
    where
        Self: Sized;
    fn run(&self) -> SuiteSummary;
    fn get_test_root(&self) -> &Path;
    fn write_success_cases(&self, _content: String) {}
}

pub trait Case {
    fn new(code: &str, filename: &str) -> Self
    where
        Self: Sized;
    fn run(&self) -> TestResult;
}
