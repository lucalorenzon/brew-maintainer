use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "formulae")]
pub struct Package {
    pub name: String,
    installed_versions: Vec<String>,
    current_version: String,
    pinned: bool,
    pinned_version: String,
}

impl Display for Package {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}(available:{}): |installed: {}|pinned: {}|pinned-version: {}|",
            &self.name,
            &self.current_version,
            &self.installed_versions.join(", "),
            &self.pinned,
            self.pinned_version
        )
    }
}
