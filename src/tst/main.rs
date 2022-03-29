use std::process::{Command, Output};

use anyhow::Context;

fn main() {
    println!("{:?}", run("./test/slows.sh"));
}

fn run(program: &str) -> anyhow::Result<Output> {
    let _output = Command::new(program)
        .output()
        .with_context(|| format!("Failed to run {program}"))?;
    Ok(_output)
}
