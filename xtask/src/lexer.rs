use std::{
    ffi::OsStr,
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

use ansi_term::Colour::{Green, Purple, Red};
use rtsc_parser::run_lexer;
use tracing::info;
use walkdir::WalkDir;

use crate::{
    suite::{Case, SuiteSummary, TestResult, TestSuite},
    utils::remove_bom,
};

const FIXTURES_NAME: &str = "lexer";
const FIXTURES_DIR: &str = "tests/lexer";

#[derive(Debug, Default, Clone)]
pub struct LexerTestCase {
    filename: String,
    code: String,
}

impl Case for LexerTestCase {
    fn new(filename: &str, code: &str) -> Self
    where
        Self: Sized,
    {
        Self {
            filename: filename.to_owned(),
            code: remove_bom(code).to_owned(),
        }
    }

    fn run(&self) -> TestResult {
        match std::panic::catch_unwind(|| run_lexer(&self.code)) {
            Ok(res) => match res {
                Ok(_) => TestResult::Success,
                Err(_) => TestResult::Failure,
            },
            Err(_) => TestResult::Panic,
        }
    }
}

#[derive(Debug, Default)]
pub struct LexerTestSuite {
    dir_name: String,
    root: PathBuf,
}

impl TestSuite for LexerTestSuite {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            dir_name: FIXTURES_NAME.to_string(),
            root: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(FIXTURES_DIR),
        }
    }

    fn run(&self) -> crate::suite::SuiteSummary {
        let root = self.get_test_root();
        let cases = WalkDir::new(root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().extension() == Some(OsStr::new("ts"))
                    || (e.path().extension() == Some(OsStr::new("js")))
            })
            .map(|e| e.path().to_path_buf())
            .collect::<Vec<_>>();
        if cases.is_empty() {
            panic!("No test cases found");
        }

        let cases = cases
            .iter()
            .map(|c| {
                let mut file = File::open(c).unwrap();
                let mut contents = String::new();
                file.read_to_string(&mut contents).unwrap();
                LexerTestCase::new(c.to_str().unwrap(), &mut contents)
            })
            .collect::<Vec<_>>();

        let total_count = cases.len();

        let mut success = 0;
        let mut failure = 0;
        let mut panic = 0;

        let mut success_cases = String::new();
        for c in cases.iter() {
            match c.run() {
                TestResult::Success => {
                    success += 1;
                    info!("{}: {:?}", Green.bold().paint("PASS"), c.filename);
                    let case = c.filename.split("/tests").nth(1).unwrap();
                    success_cases.push_str(case);
                    success_cases.push('\n');
                }
                TestResult::Failure => {
                    failure += 1;
                    info!("{}: {:?}", Red.bold().paint("FAIL"), c.filename);
                }

                TestResult::Panic => {
                    panic += 1;
                    info!("{}: {:?}", Purple.bold().paint("PANIC"), c.filename);
                }
            }
        }
        self.write_success_cases(success_cases);
        SuiteSummary::new(
            &self.dir_name,
            total_count as f64,
            success as f64,
            failure as f64,
            panic as f64,
        )
    }

    fn get_test_root(&self) -> &std::path::Path {
        &self.root
    }

    fn write_success_cases(&self, content: String) {
        let path = format!(
            "{}/summary/{}.success.txt",
            env!("CARGO_MANIFEST_DIR"),
            self.dir_name
        );
        let mut file = File::create(path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }
}
