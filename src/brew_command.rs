use std::collections::HashMap;

use chrono::Duration;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum BrewCommand<'a> {
    Update { envs: HashMap<&'static str, String> },
    Outdated { envs: HashMap<&'static str, String> },
    Upgrade { package_name: &'a str, envs: HashMap<&'static str, String> },
    Cleanup { envs: HashMap<&'static str, String> },
}

impl<'a> BrewCommand<'a> {
    // Helper to convert to CLI args
    pub fn to_args(&self) -> Vec<&'a str> {
        match self {
            BrewCommand::Update { envs: _ } => {
                vec!["update"]
            }
            BrewCommand::Outdated { envs: _ } => {
                vec!["outdated", "--json"]
            }
            BrewCommand::Upgrade { package_name, envs: _ } => {
                let mut args = vec!["upgrade"];
                args.push(package_name);
                args
            }
            BrewCommand::Cleanup { envs: _ } => {
                vec!["cleanup"]
            }
        }
    }

    pub fn to_env(&self) -> HashMap<&'static str, String> {
        match self {
            BrewCommand::Update { envs } => envs.clone(),
            BrewCommand::Outdated { envs } => envs.clone(),
            BrewCommand::Upgrade { package_name: _, envs } => envs.clone(),
            BrewCommand::Cleanup { envs } => envs.clone(),
        }
    }
}

#[derive(Debug, Error)]
pub enum BrewError {
    #[error("Error executing the brew command")]
    ExecutionFailed(String),
    #[error("Error Input request cannot be fulfilled")]
    InputRequested,
    #[error("Error command takes more than the timeout requested")]
    Timeout,
}

pub trait CommandExecutor {
    fn execute(&self, cmd: &BrewCommand) -> Result<String, BrewError>;
    fn envs(&self) -> HashMap<&'static str, String>;
    async fn execute_with_timeout<'a>(&self, cmd: &BrewCommand<'a>, timeout: Duration) -> Result<(), BrewError>;
}
