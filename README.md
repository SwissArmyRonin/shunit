# README

`shunit` runs a list of shell scripts and outputs the results in JUnit format 
for easy use in CI/CD systems.

## Usage

```
$ cargo install shunit
$ shunit --help
USAGE:
    shunit [FLAGS] [OPTIONS] [scripts]...

FLAGS:
    -h, --help       Prints help information
    -q, --quiet      Silence all output
    -V, --version    Prints version information
    -v, --verbose    Verbose mode (-v, -vv, -vvv, -vvvv). The levels are warnings, informational, debugging, and trace
                     message

OPTIONS:
    -o, --output <output>    An optional target file to write the result to
    -t, --timestamp <ts>     Timestamp (sec, ms, ns, none)

ARGS:
    <scripts>...    Test scripts
```

## Example

To run the tests in the [test](test) folder and generate an JUnit compatible XML
output, run:

```shell
shunit -o shunit.xml test/*
```

This will generate a JUnit compatible output file called shunit.xml. The file 
glob will expand to every script in the test directory, so the final suite will
contain the results of running all 4 files (including the one that isn't a script).

_The exit code from shunit is 0 if all tests succeeded, and 1 otherwise._
