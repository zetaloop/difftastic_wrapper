use std::env;
use std::io::{self, BufRead, BufReader, Write};
use std::process::{Command, Stdio, exit};

/// Search for a flag within the argument list. Returns `None` if the flag is not
/// present. Returns `Some(Some(value))` if the flag is present with a value and
/// `Some(None)` if the flag is present but no value follows.
fn find_flag_value(args: &[String], flag: &str) -> Option<Option<String>> {
    for i in 0..args.len() {
        if args[i] == flag {
            return Some(args.get(i + 1).cloned());
        }

        let with_eq = format!("{}=", flag);
        if let Some(v) = args[i].strip_prefix(&with_eq) {
            return Some(Some(v.to_string()));
        }
    }
    None
}

fn main() -> io::Result<()> {
    // Collect all arguments provided to the wrapper except the binary name
    let mut args: Vec<String> = env::args().skip(1).collect();

    // Check the provided flags and ensure the required ones are present
    let display_mode = find_flag_value(&args, "--display");
    let color_mode = find_flag_value(&args, "--color");

    if let Some(Some(d)) = &display_mode {
        if d != "inline" {
            eprintln!("difftw only supports --display=inline");
            exit(1);
        }
    }

    if let Some(Some(c)) = &color_mode {
        if c != "always" {
            eprintln!("difftw only supports --color=always");
            exit(1);
        }
    }

    // Prepend missing required flags
    let mut prefix = Vec::new();
    if display_mode.is_none() {
        prefix.push("--display=inline".to_string());
    }
    if color_mode.is_none() {
        prefix.push("--color=always".to_string());
    }
    if !prefix.is_empty() {
        args.splice(0..0, prefix);
    }

    let mut child = Command::new("difft")
        .args(&args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("Could not start difft process");

    let stdout = child.stdout.take().expect("Could not capture stdout");

    let reader = BufReader::new(stdout);
    let stdout_lock = io::stdout();
    let mut handle = stdout_lock.lock();

    for line_res in reader.lines() {
        let line = line_res?;
        // Split the line into leading spaces and the rest
        let stripped = line.trim_start();
        let leading = &line[..line.len() - stripped.len()];

        if stripped.starts_with("\x1b[2m") {
            // Gray line numbers -> one space
            writeln!(handle, " {}", line)?;
        } else if stripped.starts_with("\x1b[91;1m") {
            // Red line numbers -> red -
            let tail = &stripped["\x1b[91;1m".len()..];
            write!(handle, "{}\x1b[91;1m-{}\n", leading, tail)?;
        } else if stripped.starts_with("\x1b[92;1m") {
            // Green line numbers -> green +
            let tail = &stripped["\x1b[92;1m".len()..];
            write!(handle, "\x1b[92;1m+{}{}\n", leading, tail)?;
        } else {
            writeln!(handle, "{}", line)?;
        }
    }

    let status = child.wait()?;
    exit(status.code().unwrap_or_default());
}
