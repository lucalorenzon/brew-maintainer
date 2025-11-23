use std::{collections::HashMap, env};

use anyhow::{Context, Result};
use chrono::Duration;
use tracing::info;

use crate::{
    brew_command::{BrewCommand, BrewError, CommandExecutor},
    formulae::{OutdatedPackages, Package},
};

pub struct BrewMaintainer<E: CommandExecutor> {
    executor: E,
}

impl<E: CommandExecutor> BrewMaintainer<E> {
    pub fn new(executor: E) -> Self {
        Self { executor }
    }

    pub fn update_reference_repositories(&self) -> Result<String, BrewError> {
        self.executor.execute(&BrewCommand::Update {
            envs: self.executor.envs(),
        })
    }

    pub fn find_outdated_packages(&self) -> Result<OutdatedPackages, BrewError> {
        let outdated_json = self.executor.execute(&BrewCommand::Outdated {
            envs: self.executor.envs(),
        })?;
        let output: OutdatedPackages =
            serde_json::from_str(outdated_json.as_str()).expect("error on parsing");
        Ok(output)
    }
}

pub fn run_maintenance<E: CommandExecutor>(brew_maintainer: &BrewMaintainer<E>) -> Result<()> {
    let output = brew_maintainer
        .update_reference_repositories()
        .context("\u{274c} Failed to update reference repositories")?;
    info!("output: {}", output);
    info!("\u{2705} brew update done");
    let outdated_packages = brew_maintainer
        .find_outdated_packages()
        .context("\u{274c} Failed in finding outdated packages")?;
    info!("outdated:packages: \n{}", outdated_packages);
    info!("\u{2705} brew outdated done");
    // let failed_packages = brew_maintainer
    //     .upgrade_packages_with_timeout(outdatated_packages, Duration::minutes(30))
    //     .context("\u{274c} Failure occurred while upgrading packages")?;
    // info!("\u{2705} brew upgrade done");
    // info!("failed upgrade: {:?}", failed_packages);
    // brew_maintainer
    //     .cleanup()
    //     .context("\u{274c} Failed to cleanup")?;
    // info!("\u{2705} brew cleanup done");
    Ok(())
}
