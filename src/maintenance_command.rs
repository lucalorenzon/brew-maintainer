use std::{collections::HashMap, env, process::Command};

use tracing::info;

use crate::brew_command::{BrewCommand, BrewError, CommandExecutor};

pub struct RealBrewCommand;

impl CommandExecutor for RealBrewCommand {
    fn execute(&self, cmd: &BrewCommand) -> Result<String, BrewError> {
        let args = cmd.to_args();
        let env_map = cmd.to_env();
        info!("executing: brew {:?}", args.join(" "));

        let output = Command::new("brew")
            .envs(&env_map)
            .args(&args)
            .output()
            .map_err(|e| BrewError::ExecutionFailed(e.to_string()))?;

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
}
