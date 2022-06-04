# README

`shunit` runs a list of shell scripts and outputs the results in JUnit format for easy use in CI/CD systems.

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
