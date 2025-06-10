use std::env;
use std::io::{self, BufRead, BufReader, IsTerminal, Write};
use std::process::{Command, Stdio, exit};
use strip_ansi_escapes::strip_str;

const GRAY: &str = "\x1b[2m";
const RED: &str = "\x1b[91;1m";
const GREEN: &str = "\x1b[92;1m";

/// Validate `--display` flag and insert required defaults.
fn validate_and_add_flags(args: &mut Vec<String>) {
    let has_display = args.iter().any(|a| a.starts_with("--display"));

    if let Some(flag) = args.iter().find(|a| a.starts_with("--display=")) {
        if !flag.ends_with("inline") {
            eprintln!("difftw only supports --display=inline");
            exit(1);
        }
    }

    if !has_display {
        args.insert(0, "--display=inline".to_string());
    }

    // Force difft to always output colors so that our parser works correctly.
    args.insert(0, "--color=always".to_string());
}

enum ColorSetting {
    Auto,
    Always,
    Never,
}

/// Determine the desired output color setting for the wrapper.
/// The `--color` command line argument has precedence over the `DFT_COLOR`
/// environment variable. Allowed values are `always`, `auto` and `never`.
fn parse_color_setting(args: &mut Vec<String>) -> ColorSetting {
    if let Some(pos) = args.iter().position(|a| a.starts_with("--color=")) {
        let arg = args.remove(pos);
        return match &arg[8..] {
            "always" => ColorSetting::Always,
            "auto" => ColorSetting::Auto,
            "never" => ColorSetting::Never,
            other => {
                eprintln!("Invalid value for --color: {}", other);
                exit(1);
            }
        };
    }

    if let Ok(var) = env::var("DFT_COLOR") {
        return match var.as_str() {
            "always" => ColorSetting::Always,
            "auto" => ColorSetting::Auto,
            "never" => ColorSetting::Never,
            other => {
                eprintln!("Invalid value for DFT_COLOR: {}", other);
                exit(1);
            }
        };
    }

    ColorSetting::Auto
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

    // Determine wrapper color setting and remove the flag from difft args
    let color_setting = parse_color_setting(&mut args);

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
    let strip_color = match color_setting {
        ColorSetting::Always => false,
        ColorSetting::Never => true,
        ColorSetting::Auto => !std::io::stdout().is_terminal(),
    };

    for line_res in reader.lines() {
        let line = line_res?;
        process_line(&line, &mut handle, strip_color)?;
    }

    let status = child.wait()?;
    exit(status.code().unwrap_or_default());
}
