use std::{collections::HashMap, error::Error};

use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum BrewCommand<'a> {
    Update {
        envs: HashMap<&'static str, String>,
    },
    Outdated {
        envs: HashMap<&'static str, String>,
    },
    Upgrade {
        packages: Vec<&'a str>,
        envs: HashMap<&'static str, String>,
    },
    Cleanup {
        envs: HashMap<&'static str, String>,
    },
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
            BrewCommand::Upgrade { packages, envs: _ } => {
                let mut args = vec!["upgrade"];
                for &package_name in packages {
                    args.push(package_name);
                }
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
            BrewCommand::Upgrade { packages: _, envs } => envs.clone(),
            BrewCommand::Cleanup { envs } => envs.clone(),
        }
    }
}

#[derive(Debug, Error)]
pub enum BrewError {
    #[error("Error executing the brew command")]
    ExecutionFailed(String),
}

pub trait CommandExecutor {
    fn execute(&self, cmd: &BrewCommand) -> Result<String, BrewError>;
    fn envs(&self) -> HashMap<&'static str, String>;
}
