use anyhow::{Context, Result};
use chrono::Duration;
use tracing::info;

use crate::{
    brew_command::{BrewCommand, BrewError, CommandExecutor},
    formulae::{OutdatedPackages, Package},
};

pub struct BrewMaintainer<'b, E: CommandExecutor> {
    executor: &'b E,
}

impl<'b, E: CommandExecutor> BrewMaintainer<'b, E> {
    pub fn new(executor: &'b E) -> Self {
        Self { executor }
    }

    pub fn update_reference_repositories(&self) -> Result<String, BrewError> {
        self.executor.execute(&BrewCommand::Update { envs: self.executor.envs() })
    }

    pub fn find_outdated_packages(&self) -> Result<OutdatedPackages, BrewError> {
        let outdated_json = self.executor.execute(&BrewCommand::Outdated { envs: self.executor.envs() })?;
        let output: OutdatedPackages = serde_json::from_str(outdated_json.as_str()).expect("error on parsing");
        Ok(output)
    }

    pub async fn upgrade_packages_with_timeout<'a>(
        &self, outdated_packages: &'a OutdatedPackages, timeout: Duration,
    ) -> Result<Vec<&'a Package>, BrewError> {
        let mut failed_upgrade: Vec<&'a Package> = vec![];
        for package in outdated_packages.iter() {
            if let Err(_) = self
                .executor
                .execute_with_timeout(
                    &BrewCommand::Upgrade { package_name: package.name.as_str(), envs: self.executor.envs() },
                    timeout,
                )
                .await
            {
                failed_upgrade.push(package);
            }
        }
        Ok(failed_upgrade)
    }

    pub fn cleanup(&self) -> Result<String, BrewError> {
        self.executor.execute(&BrewCommand::Cleanup { envs: self.executor.envs() })
    }
}

pub async fn run_maintenance<'a, E: CommandExecutor>(brew_maintainer: &BrewMaintainer<'a, E>) -> Result<()> {
    let output = brew_maintainer.update_reference_repositories().context("\u{274c} Failed to update reference repositories")?;
    info!("output: {}", output);
    info!("\u{2705} brew update done");
    let outdated_packages = brew_maintainer.find_outdated_packages().context("\u{274c} Failed in finding outdated packages")?;
    info!("outdated:packages: \n{}", outdated_packages);
    info!("\u{2705} brew outdated done");
    let failed_packages = brew_maintainer
        .upgrade_packages_with_timeout(&outdated_packages, Duration::minutes(5))
        .await
        .context("\u{274c} Failure occurred while upgrading packages")?;
    info!("failed upgrade: {:?}", failed_packages);
    info!("\u{2705} brew upgrade done");
    let output = brew_maintainer.cleanup().context("\u{274c} Failed to cleanup")?;
    info!("output: {}", output);
    info!("\u{2705} brew cleanup done");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
        time::Duration as StdDuration,
    };

    use crate::{
        brew_command::{BrewCommand, BrewError, CommandExecutor},
        service::BrewMaintainer,
    };

    #[test]
    fn should_run_brew_update_command_with_success_when_no_update_are_present() {
        let expected_output = "";
        let mock = MockBrewCommand::new().with_execute_response(Ok(expected_output.to_string()));
        let system_under_test = BrewMaintainer::new(&mock);
        let output = system_under_test.update_reference_repositories();
        assert!(output.is_ok());
        assert_eq!(expected_output, output.unwrap_or_default().as_str());
        mock.assert_call_count(1);
        mock.assert_command_called(&["update"]);
    }

    #[test]
    fn should_run_brew_update_command_with_success_when_update_are_present() {
        let expected_output = "Already up-to-date.";
        let mock = MockBrewCommand::new().with_execute_response(Ok(expected_output.to_string()));
        let system_under_test = BrewMaintainer::new(&mock);
        let output = system_under_test.update_reference_repositories();
        assert!(output.is_ok());
        assert_eq!(expected_output, output.unwrap_or_default().as_str());
        mock.assert_call_count(1);
        mock.assert_command_called(&["update"]);
    }

    pub struct MockBrewCommand {
        /// Captured commands that were executed
        pub captured_commands: Arc<Mutex<Vec<CapturedCommand>>>,
        /// Configured responses for execute()
        pub execute_responses: Arc<Mutex<Vec<Result<String, BrewError>>>>,
        /// Configured responses for execute_with_timeout()
        pub timeout_responses: Arc<Mutex<Vec<Result<(), BrewError>>>>,
        /// Simulated delay before returning (for timeout testing)
        pub simulated_delay: Option<StdDuration>,
    }

    #[derive(Debug, Clone)]
    pub struct CapturedCommand {
        pub command: String,
        pub args: Vec<String>,
        pub envs: HashMap<String, String>,
        pub timeout: Option<Duration>,
    }

    impl MockBrewCommand {
        fn new() -> Self {
            Self {
                captured_commands: Arc::new(Mutex::new(Vec::new())),
                execute_responses: Arc::new(Mutex::new(Vec::new())),
                timeout_responses: Arc::new(Mutex::new(Vec::new())),
                simulated_delay: None,
            }
        }

        pub fn with_execute_response(self, response: Result<String, BrewError>) -> Self {
            self.execute_responses.lock().unwrap().push(response);
            self
        }
        pub fn with_timeout_response(self, response: Result<(), BrewError>) -> Self {
            self.timeout_responses.lock().unwrap().push(response);
            self
        }
        pub fn with_delay(mut self, delay: StdDuration) -> Self {
            self.simulated_delay = Some(delay);
            self
        }
        pub fn get_captured_commands(&self) -> Vec<CapturedCommand> {
            self.captured_commands.lock().unwrap().clone()
        }
        pub fn assert_command_called(&self, expected_args: &[&str]) {
            let captured = self.captured_commands.lock().unwrap();
            let found = captured.iter().any(|cmd| cmd.args == expected_args.iter().map(|s| s.to_string()).collect::<Vec<_>>());
            assert!(found, "Command with args {:?} was not called. Captured: {:?}", expected_args, *captured);
        }
        pub fn assert_call_count(&self, expected: usize) {
            let captured = self.captured_commands.lock().unwrap();
            assert_eq!(captured.len(), expected, "Expected {} calls, got {}", expected, captured.len());
        }
    }

    impl CommandExecutor for MockBrewCommand {
        fn execute(&self, cmd: &BrewCommand) -> std::result::Result<String, BrewError> {
            let args = cmd.to_args();
            let env_map = cmd.to_env();

            // Capture the command
            self.captured_commands.lock().unwrap().push(CapturedCommand {
                command: "brew".to_string(),
                args: args.into_iter().map(|arg| arg.to_owned()).collect(),
                envs: env_map.iter().map(|(k, v)| (k.to_string(), v.clone())).collect(),
                timeout: None,
            });

            // Return configured response or default success
            let mut responses = self.execute_responses.lock().unwrap();
            if !responses.is_empty() { responses.remove(0) } else { Ok("Mock output".to_string()) }
        }

        fn envs(&self) -> HashMap<&'static str, String> {
            let mut envs = HashMap::new();
            envs.insert("HOME", "/mock/home".to_string());
            envs.insert("PATH", "/mock/path".to_string());
            envs
        }

        async fn execute_with_timeout<'a>(&self, cmd: &BrewCommand<'a>, timeout: Duration) -> std::result::Result<(), BrewError> {
            let args = cmd.to_args();
            let env_map = cmd.to_env();

            // Capture the command with timeout
            self.captured_commands.lock().unwrap().push(CapturedCommand {
                command: "brew".to_string(),
                args: args.into_iter().map(|arg| arg.to_owned()).collect(),
                envs: env_map.iter().map(|(k, v)| (k.to_string(), v.clone())).collect(),
                timeout: Some(timeout),
            });

            // Simulate delay if configured
            if let Some(delay) = self.simulated_delay {
                tokio::time::sleep(delay).await;
            }

            // Return configured response or default success
            let mut responses = self.timeout_responses.lock().unwrap();
            if !responses.is_empty() { responses.remove(0) } else { Ok(()) }
        }
    }
}
