use std::env;
use std::io::{self, BufRead, BufReader, Write};
use std::process::{Command, Stdio, exit};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();

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
