// src/main.rs
use chrono::Local;
use std::fs::OpenOptions;
use std::io::Write;
use std::process::Command;

fn main() {
    let timestamp = Local::now();
    let log_file = format!(
        "/usr/local/var/log/brew-maintainer-{}.log",
        timestamp.format("%Y%m%d-%H%M%S")
    );
    let mut log = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)
        .unwrap();

    writeln!(log, "=== Brew Maintenance Run at {} ===", timestamp).unwrap();

    let steps = [
        ("brew update", vec!["update"]),
        ("brew upgrade", vec!["upgrade", "--quiet"]),
        ("brew cleanup", vec!["cleanup"]),
    ];

    for (desc, args) in steps {
        writeln!(log, "\nRunning: brew {}", args.join(" ")).unwrap();
        let output = Command::new("brew").args(args).output();

        match output {
            Ok(out) => {
                if !out.status.success() {
                    writeln!(
                        log,
                        "❌ Step `{}` failed.\nError:\n{}",
                        desc,
                        String::from_utf8_lossy(&out.stderr)
                    )
                    .unwrap();
                } else {
                    writeln!(log, "✅ Step `{}` completed.", desc).unwrap();
                }
            }
            Err(e) => writeln!(log, "⚠️  Failed to run `{}`: {}", desc, e).unwrap(),
        }
    }

    writeln!(log, "\nRun complete. Log saved at {}", log_file).unwrap();
}
