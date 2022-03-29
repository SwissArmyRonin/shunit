use crate::model::*;
use anyhow::anyhow;
use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use std::process::ExitStatus;
use std::process::Stdio;
use std::{env, fs, io, path, time};
use structopt::StructOpt;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

#[macro_use]
extern crate yaserde_derive;

mod model;

type LogLine = (DateTime<Utc>, String);

type ScriptResult = anyhow::Result<(ExitStatus, Vec<LogLine>, Vec<LogLine>)>;

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

    /// An optional target file to write the result to.
    #[structopt(short = "o", long)]
    output: Option<String>,

    /// Test scripts.
    scripts: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opt = Opt::from_args();
    let script_count = opt.scripts.len() as u32;

    stderrlog::new()
        .module(module_path!())
        .quiet(opt.quiet)
        .verbosity(opt.verbose)
        .timestamp(opt.ts.unwrap_or(stderrlog::Timestamp::Off))
        .init()?;

    let mut error_count = 0;
    let mut failure_count = 0;

    if opt.scripts.is_empty() {
        return Ok(());
    }

    let start = time::Instant::now();

    let mut stdout_messages: Vec<LogLine> = vec![];
    let mut stderr_messages: Vec<LogLine> = vec![];
    let mut testcases: Vec<TestCase> = vec![];

    for name in opt.scripts {
        let absolute_path = fs::canonicalize(&name)?;
        let classname = absolute_path
            .into_os_string()
            .into_string()
            .map_err(|os_string| {
                anyhow!("Unable to determine the absolute path for {:?}", os_string)
            })?;

        let duration = start.elapsed();
        let result = run_script(&name[..]).await;
        let time = duration.as_secs_f32();

        let error = match result {
            Ok((exit_code, stdout, stderr)) => {
                stdout_messages.extend(stdout.iter().cloned());
                stderr_messages.extend(stderr.iter().cloned());
                if exit_code.success() {
                    None
                } else {
                    failure_count += 1;
                    let body = join_and_sort(join_log_lines(&stdout), join_log_lines(&stderr));
                    let body: Vec<String> = body.into_iter().map(|line| line.1).collect();
                    let body = body.concat();
                    Some(TestError {
                        message: format!("Non-zero exit-code: {}", exit_code.code().unwrap_or(-1)),
                        error_type: String::from("Assertion failed"),
                        body,
                    })
                }
            }
            Err(error) => {
                error_count += 1;
                Some(TestError {
                    message: error.to_string(),
                    error_type: String::from("IO error"),
                    body: String::new(),
                })
            }
        };

        let testcase = TestCase {
            classname,
            name,
            time,
            error,
        };

        testcases.push(testcase);
    }

    let duration = start.elapsed();

    let properties: Vec<Property> = env::vars()
        .map(|(name, value)| Property { name, value })
        .collect();

    let system_out: Vec<String> = stdout_messages.into_iter().map(|line| line.1).collect();
    let system_err: Vec<String> = stderr_messages.into_iter().map(|line| line.1).collect();

    let testsuite = TestSuite {
        testcases,
        errors: error_count,
        failures: failure_count,
        time: duration.as_secs_f32(),
        tests: script_count,
        system_out: system_out.concat(),
        system_err: system_err.concat(),
        name: env::var("PWD").unwrap_or_else(|_| "Unknown".to_string()),
        properties: Properties { properties },
        ..Default::default()
    };

    let yaserde_cfg = yaserde::ser::Config {
        perform_indent: true,
        ..Default::default()
    };

    let out = opt.output;

    let mut out_writer = match out {
        Some(x) => {
            let path = path::Path::new(&x);
            Box::new(fs::File::create(&path).unwrap()) as Box<dyn io::Write>
        }
        None => Box::new(io::stdout()) as Box<dyn io::Write>,
    };

    let output = yaserde::ser::to_string_with_config(&testsuite, &yaserde_cfg)
        .map_err(|msg| anyhow!(msg))?;

    out_writer
        .write(output.as_bytes())
        .map_err(|err| anyhow!("Failed to output test result: {:?}", err))?;

    Ok(())
}

/// Merge two log streams and sort the contents,
fn join_and_sort(stdout: Vec<LogLine>, stderr: Vec<LogLine>) -> Vec<LogLine> {
    let stdout = stdout[..].as_ref();
    let stderr = stderr[..].as_ref();
    let mut result = [stdout, stderr].concat();
    result.sort_by(|a, b| a.0.cmp(&b.0));
    result
}

/**
Join log messages so there is one message per line ending with a new line.

Log messages are intereleved so if a line was sliced into two messages, they become a single message,
with the timestamp from the first message.

- `messages` a vector of log messages and timestamps sorted in ascending order.
*/
fn join_log_lines(messages: &[(DateTime<Utc>, String)]) -> Vec<LogLine> {
    let mut joined_messages: Vec<LogLine> = vec![];
    let mut line: String = String::new();
    let mut first_ts: Option<DateTime<Utc>> = None;

    for (index, (ts, message)) in messages.iter().enumerate() {
        line.push_str(message);
        if first_ts.is_none() {
            first_ts = Some(*ts);
        }
        if message.ends_with('\n') || index == (messages.len() - 1) {
            joined_messages.push((first_ts.unwrap(), line));
            first_ts = None;
            line = String::new();
        }
    }

    joined_messages
}

// https://stackoverflow.com/questions/68173678/read-childstdout-without-blocking
// https://stackoverflow.com/questions/34611742/how-do-i-read-the-output-of-a-child-process-without-blocking-in-rust
async fn run_script(program: &str) -> ScriptResult {
    let mut child = Command::new(program)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("No stdout handle?"))?;
    let mut stdout = BufReader::new(stdout).lines();

    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow!("No stdout handle?"))?;
    let mut stderr = BufReader::new(stderr).lines();

    let mut stdout_vector: Vec<LogLine> = vec![];
    let mut stderr_vector: Vec<LogLine> = vec![];

    let handle = tokio::spawn(async move { child.wait().await });

    loop {
        let now: DateTime<Utc> = Utc::now();
        let stdout_line = stdout.next_line().await?;
        let stderr_line = stderr.next_line().await?;
        if stdout_line == None && stderr_line == None {
            break;
        }
        if let Some(line) = stdout_line {
            println!("{line}");
            stdout_vector.push((now, line));
        }
        if let Some(line) = stderr_line {
            eprintln!("{line}");
            stderr_vector.push((now, line));
        }
    }

    let exit_code = handle.await??;
    Ok((exit_code, stdout_vector, stderr_vector))
}

#[cfg(test)]
mod test {
    use crate::{join_log_lines, LogLine};
    use chrono::DateTime;
    use std::{str::FromStr, sync::Once};

    static INIT: Once = Once::new();

    pub fn setup() {
        INIT.call_once(|| {
            stderrlog::new()
                .module(module_path!())
                .verbosity(5)
                .init()
                .unwrap();
        });
    }

    #[test]
    fn test_join_log_lines() {
        setup();
        let ts1 = DateTime::from_str("2022-04-03 10:13:48 UTC").unwrap();
        let ts2 = DateTime::from_str("2022-04-03 10:13:49 UTC").unwrap();
        let ts3 = DateTime::from_str("2022-04-03 10:13:50 UTC").unwrap();
        let messages: Vec<LogLine> = vec![
            (ts1, "A".to_string()),
            (ts2, "B\n".to_string()),
            (ts3, "C".to_string()),
        ];
        let joined = join_log_lines(&messages);
        assert_eq!(joined.len(), 2);
        assert_eq!(joined[0], (ts1, "AB\n".to_string()));
        assert_eq!(joined[1], (ts3, "C".to_string()));
    }
}
