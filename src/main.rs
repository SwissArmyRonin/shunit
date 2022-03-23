use std::{env, error, fs, process, time};

use log::*;
use model::{Property, TestCase, TestError, TestSuite};
use structopt::StructOpt;

use crate::model::Properties;

#[macro_use]
extern crate yaserde_derive;

mod model;

#[derive(StructOpt, Debug)]
#[structopt()]
struct Opt {
    /// Silence all output
    #[structopt(short = "q", long)]
    quiet: bool,

    /// Verbose mode (-v, -vv, -vvv, -vvvv). The levels are warnings, informational, debugging, and trace message.
    #[structopt(short = "v", long, parse(from_occurrences))]
    verbose: usize,

    /// Timestamp (sec, ms, ns, none)
    #[structopt(short = "t", long = "timestamp")]
    ts: Option<stderrlog::Timestamp>,

    // Test scripts
    scripts: Vec<String>,
}

fn main() {
    let opt = Opt::from_args();

    stderrlog::new()
        .module(module_path!())
        .module("irs") // Log the lib as well
        .quiet(opt.quiet)
        .verbosity(opt.verbose)
        .timestamp(opt.ts.unwrap_or(stderrlog::Timestamp::Off))
        .init()
        .unwrap();

    let mut error_count = 0;

    let start = time::Instant::now();
    let testcases: Vec<TestCase> = opt
        .scripts
        .iter()
        .map(|script_name| run_script(script_name))
        .filter(|res| match res {
            Ok(_) => true,
            Err(err) => {
                error_count += 1;
                error!("Failed to process a script: {:?}", err);
                false
            }
        })
        .map(|tc| tc.unwrap())
        .collect();
    let duration = start.elapsed();

    let failure_count = testcases
        .iter()
        .map(|tc| match tc.error {
            Some(_) => 1,
            None => 0,
        })
        .reduce(|acc, elem| acc + elem)
        .unwrap_or(0);

    let properties: Vec<Property> = env::vars()
        .map(|(name, value)| Property { name, value })
        .collect();

    let testsuite = TestSuite {
        testcases,
        errors: error_count,
        failures: failure_count,
        time: duration.as_secs_f32(),
        tests: opt.scripts.len() as u32,
        name: env::var("PWD").unwrap_or("Unknown".to_string()),
        properties: Properties { properties },
        ..Default::default()
    };

    let yaserde_cfg = yaserde::ser::Config {
        perform_indent: true,
        ..Default::default()
    };

    info!(
        "{}",
        yaserde::ser::to_string_with_config(&testsuite, &yaserde_cfg)
            .ok()
            .unwrap()
    );
}

/// Run a test script and output a `TestCase`object with the result.
fn run_script(script_name: &str) -> Result<TestCase, Box<dyn std::error::Error>> {
    let absolute_path = fs::canonicalize(script_name)?;
    let start = time::Instant::now();
    let output = process::Command::new(script_name).output();
    let duration = start.elapsed();
    let text_output;

    let error = match output {
        Ok(result) => {
            text_output = std::str::from_utf8(result.stderr.as_slice())
                .unwrap_or("Unable to parse handler output");
            let code = result.status.code().unwrap_or(-1);
            match result.status.success() {
                true => None,
                false => Some(TestError {
                    body: text_output.to_string(),
                    message: format!("Exit code: {code}"),
                    error_type: "non_zero_exit".to_string(),
                }),
            }
        }
        Err(error) => Some(TestError {
            body: format!("{:?}", error),
            message: format!("{:?}", error),
            error_type: "failed_to_run".to_string(),
        }),
    };

    let classname = absolute_path.into_os_string().into_string().unwrap();
    let name = script_name.to_string();
    let time = duration.as_secs_f32();

    Ok(TestCase {
        classname,
        name,
        time,
        error,
    })
}

#[cfg(test)]
mod test {
    use log::info;

    use crate::run_script;

    #[test]
    fn run_failing_script() {
        let result = run_script("./test/bad_apple.sh").expect("Should return a TestCase");
        assert!(result.error.is_some());
        println!("{:?}", result);
    }

    #[test]
    fn run_ok_script() {
        let result = run_script("./test/im_ok.sh").expect("Should return a TestCase");
        assert!(result.error.is_none());
        println!("{:?}", result);
    }

    // #[test]
    // fn test_serialization() {
    //     use std::fs;
    //     use yaserde::de::from_str;
    //     let filename = "test/JUnit.xml";
    //     let content = fs::read_to_string(filename).expect("something went wrong reading the file");
    //     let junit_result: TestSuite = from_str(&content).unwrap();

    //     let yaserde_cfg = yaserde::ser::Config {
    //         perform_indent: true,
    //         ..Default::default()
    //     };

    //     debug!(
    //         "{}",
    //         yaserde::ser::to_string_with_config(&junit_result, &yaserde_cfg)
    //             .ok()
    //             .unwrap()
    //     );
    // }
}
