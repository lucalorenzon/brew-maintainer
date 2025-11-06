use std::fs::{self};
use std::process::Command;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, fmt};

fn init_logging() {
    // Choose log directory (works for both architectures)
    let log_dir = if fs::metadata("/opt/homebrew/var/log").is_ok() {
        "/opt/homebrew/var/log"
    } else {
        "/usr/local/var/log"
    };

    // let log_path = format!("{}/brew-maintainer.log", log_dir);

    // Make sure directory exists
    let _ = fs::create_dir_all(log_dir);

    // Initialize tracing subscriber
    let file_appender = tracing_appender::rolling::daily(log_dir, "brew-maintainer.log");
    let (file_writer, _guard) = tracing_appender::non_blocking(file_appender);

    let stdout_layer = fmt::layer().with_target(false).with_writer(std::io::stdout);
    let file_layer = fmt::layer()
        .with_target(false)
        .with_ansi(false)
        .with_writer(file_writer);

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // keep guard alive for the life of the program
    tracing_subscriber::registry()
        .with(filter)
        .with(stdout_layer)
        .with(file_layer)
        .init();

    info!("brew-maintainer logging initialized");
}

fn main() {
    init_logging();

    info!("=== Brew Maintenance Started ===>|");

    let steps = [
        ("brew update", vec!["update"]),
        ("brew upgrade", vec!["upgrade", "--quiet"]),
        ("brew cleanup", vec!["cleanup"]),
    ];

    for (desc, args) in steps {
        info!("Running: brew {}", args.join(" "));
        let output = Command::new("brew").args(args).output();

        match output {
            Ok(out) => {
                if !out.status.success() {
                    info!(
                        "❌ Step `{}` failed.\nError:\n{}",
                        desc,
                        String::from_utf8_lossy(&out.stderr)
                    );
                } else {
                    info!("✅ Step `{}` completed.", desc);
                }
            }
            Err(e) => info!("⚠️  Failed to run `{}`: {}", desc, e),
        }
    }

    info!("|<============= Run complete.");
}
