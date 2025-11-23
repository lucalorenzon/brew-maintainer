use std::process::Command as StdCommand;
use std::time::Duration as StdDuration;
use std::{
    collections::HashMap,
    env,
    process::Stdio,
    sync::mpsc::{Sender, channel},
    thread,
};
use tokio::process::Child as TokioChild;
use tokio::process::Command as TokioCommand;
use tracing::info;

use crate::brew_command::{BrewCommand, BrewError, CommandExecutor};

pub struct RealBrewCommand;

impl CommandExecutor for RealBrewCommand {
    fn execute(&self, cmd: &BrewCommand) -> Result<String, BrewError> {
        let args = cmd.to_args();
        let env_map = cmd.to_env();
        info!("executing: brew {:?}", args.join(" "));

        let output =
            StdCommand::new("brew").envs(&env_map).args(&args).output().map_err(|e| BrewError::ExecutionFailed(e.to_string()))?;

        if output.status.success() {
            String::from_utf8(output.stdout).map_err(|e| BrewError::ExecutionFailed(e.to_string()))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(BrewError::ExecutionFailed(stderr.to_string()))
        }
    }
    fn envs(&self) -> HashMap<&'static str, String> {
        let mut envs: HashMap<&'static str, String> = HashMap::new();
        if let Ok(home) = env::var("HOME") {
            envs.insert("HOME", home);
        }
        if let Ok(path) = env::var("PATH") {
            envs.insert("PATH", path);
        }
        envs
    }

    async fn execute_with_timeout<'a>(&self, cmd: &BrewCommand<'a>, timeout: chrono::Duration) -> Result<(), BrewError> {
        let std_timeout = StdDuration::from_millis(timeout.num_milliseconds().max(0) as u64);
        let args = cmd.to_args();
        let env_map = cmd.to_env();
        info!("executing: brew {:?}", args.join(" "));
        let mut child = spawn_brew_process(args, env_map)?;
        let child_id = child.id().ok_or(BrewError::ExecutionFailed("No PID".to_string()))?;
        info!("executing with PID {:?}", child_id);
        let (error_tx, error_rx) = channel();
        let (event_tx, event_rx) = channel();

        // Spawn monitoring threads for stdout/stderr
        let stdout = child.stdout.take().unwrap();
        let error_tx_stdout = error_tx.clone();
        tokio::spawn(async move {
            monitor_async_output(stdout, error_tx_stdout).await;
        });

        let stderr = child.stderr.take().unwrap();
        let error_tx_stderr = error_tx.clone();
        tokio::spawn(async move {
            monitor_async_output(stderr, error_tx_stderr).await;
        });

        // Spawn completion monitor thread
        let completion_thread = spawn_completion_monitor(child, event_tx.clone());

        // Spawn timeout thread
        let timeout_thread = spawn_timeout_monitor(std_timeout, event_tx);

        // Main thread blocks waiting for first event
        let result = loop {
            // Check error channel (input detection) - this has priority
            if let Ok(error) = error_rx.try_recv() {
                kill_process_by_pid(child_id);
                break Err(error);
            }

            // Block on event channel (completion or timeout)
            match event_rx.recv() {
                Ok(ProcessEvent::Error(error)) => {
                    // Timeout occurred
                    kill_process_by_pid(child_id);
                    break Err(error);
                }
                Ok(ProcessEvent::Completed(Ok(status))) if status.success() => {
                    // Process completed successfully
                    break Ok(());
                }
                Ok(ProcessEvent::Completed(Ok(status))) => {
                    // Process completed with error
                    break Err(BrewError::ExecutionFailed(format!("Process exited with code: {:?}", status.code())));
                }
                Ok(ProcessEvent::Completed(Err(e))) => {
                    // Error waiting for process
                    break Err(BrewError::ExecutionFailed(e.to_string()));
                }
                Err(_) => {
                    // Channel closed unexpectedly
                    break Err(BrewError::ExecutionFailed("Event channel closed".to_string()));
                }
            }
        };

        // Cleanup: ensure process is killed if still running
        kill_process_by_pid(child_id);
        cleanup_threads(vec![completion_thread, timeout_thread]);

        result
    }
}

fn is_waiting_for_input(line: &str) -> bool {
    let line_lower = line.to_lowercase();

    let patterns = [
        "y/n",
        "(y/n)",
        "[y/n]",
        "yes/no",
        "(yes/no)",
        "[yes/no]",
        "press enter",
        "continue?",
        "proceed?",
        "password:",
        "passphrase:",
        "are you sure",
        "do you want",
        "would you like",
    ];

    patterns.iter().any(|pattern| line_lower.contains(pattern))
}

fn spawn_brew_process(args: Vec<&str>, envs: HashMap<&str, String>) -> Result<TokioChild, BrewError> {
    TokioCommand::new("brew")
        .args(args)
        .envs(envs)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| BrewError::ExecutionFailed(e.to_string()))
}

/// Spawns a tokio task that monitors an async stream for input requests
async fn monitor_async_output<R: tokio::io::AsyncRead + Unpin>(stream: R, tx: Sender<BrewError>) {
    use tokio::io::AsyncBufReadExt;

    let reader = tokio::io::BufReader::new(stream);
    let mut lines = reader.lines();

    while let Ok(Some(line)) = lines.next_line().await {
        if is_waiting_for_input(&line) {
            let _ = tx.send(BrewError::InputRequested);
            break;
        }
    }
}

fn spawn_completion_monitor(child: tokio::process::Child, tx: Sender<ProcessEvent>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async { child.wait_with_output().await });

        let event = match result {
            Ok(output) => ProcessEvent::Completed(Ok(output.status)),
            Err(e) => ProcessEvent::Completed(Err(e)),
        };

        let _ = tx.send(event);
    })
}

fn spawn_timeout_monitor(timeout: StdDuration, tx: Sender<ProcessEvent>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        thread::sleep(timeout);
        let _ = tx.send(ProcessEvent::Error(BrewError::Timeout));
    })
}

enum ProcessEvent {
    Error(BrewError),
    Completed(Result<std::process::ExitStatus, std::io::Error>),
}

fn cleanup_threads(threads: Vec<thread::JoinHandle<()>>) {
    for thread in threads {
        let _ = thread.join();
    }
}

fn kill_process_by_pid(pid: u32) {
    #[cfg(unix)]
    {
        use nix::sys::signal::{Signal, kill};
        use nix::unistd::Pid;
        let _ = kill(Pid::from_raw(pid as i32), Signal::SIGKILL);
    }

    #[cfg(windows)]
    {
        // On Windows, the kill() call in kill_child_process should suffice
        let _ = pid; // Suppress unused variable warning
    }
}
