mod brew_command;
mod formulae;
mod logging;
mod maintenance_command;
mod service;

use crate::{
    logging::init_logging,
    maintenance_command::RealBrewCommand,
    service::{BrewMaintainer, run_maintenance},
};
use anyhow::Result;
use chrono::Local;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    init_logging();
    let start_time = Local::now();
    info!("=== Brew Maintenance Started at {} ===>|", start_time);
    let command = BrewMaintainer::new(&RealBrewCommand);

    match run_maintenance(&command).await {
        Ok(_) => info!("|<============= Run complete."),
        Err(e) => info!("|<============= Run failed: {}", e),
    }
    let end_time = Local::now();
    let duration = end_time - start_time;
    info!("=== Brew Maintenance Finished at {} taking {} ===>|", end_time, duration);
    Ok(())
}
