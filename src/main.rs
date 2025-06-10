use atty::Stream;
use std::env;
use std::io::{self, BufRead, BufReader, Write};
use std::process::{Command, Stdio, exit};
use strip_ansi_escapes::strip_str;

const GRAY: &str = "\x1b[2m";
const RED: &str = "\x1b[91;1m";
const GREEN: &str = "\x1b[92;1m";

/// Validate `--display` and `--color` flags and insert defaults if missing.
fn validate_and_add_flags(args: &mut Vec<String>) {
    let has_display = args.iter().any(|a| a.starts_with("--display"));
    let has_color = args.iter().any(|a| a.starts_with("--color"));

    if let Some(flag) = args.iter().find(|a| a.starts_with("--display=")) {
        if !flag.ends_with("inline") {
            eprintln!("difftw only supports --display=inline");
            exit(1);
        }
    }

    if let Some(flag) = args.iter().find(|a| a.starts_with("--color=")) {
        if !flag.ends_with("always") {
            eprintln!("difftw only supports --color=always");
            exit(1);
        }
    }

    if !has_display {
        args.insert(0, "--display=inline".to_string());
    }
    if !has_color {
        args.insert(0, "--color=always".to_string());
    }
}

fn process_line(line: &str, handle: &mut impl Write, strip_color: bool) -> io::Result<()> {
    let trimmed = line.trim_start();
    let indent_len = line.len() - trimmed.len();
    let indent = &line[..indent_len];

    let mut output = if trimmed.starts_with(GRAY) {
        format!(" {}\n", line)
    } else if trimmed.starts_with(RED) {
        format!("{}{}-{}\n", indent, RED, &trimmed[RED.len()..])
    } else if trimmed.starts_with(GREEN) {
        format!("{}+{}{}\n", GREEN, indent, &trimmed[GREEN.len()..])
    } else {
        format!("{}\n", line)
    };

    if strip_color {
        output = strip_str(&output);
    }

    write!(handle, "{}", output)
}

fn main() -> io::Result<()> {
    // Collect all arguments provided to the wrapper except the binary name
    let mut args: Vec<String> = env::args().skip(1).collect();

    // Ensure required flags are valid and present
    validate_and_add_flags(&mut args);

    let mut child = Command::new("difft")
        .args(&args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("Could not start difft process");

    let stdout = child.stdout.take().expect("Could not capture stdout");

    let reader = BufReader::new(stdout);
    let mut handle = io::stdout();
    let strip_color = !atty::is(Stream::Stdout);

    for line_res in reader.lines() {
        let line = line_res?;
        process_line(&line, &mut handle, strip_color)?;
    }

    let status = child.wait()?;
    exit(status.code().unwrap_or_default());
}
