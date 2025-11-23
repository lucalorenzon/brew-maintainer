use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::formulae;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OutdatedPackages {
    pub formulae: Vec<Package>,
    pub casks: Vec<Package>,
}

impl From<&OutdatedPackages> for String {
    fn from(output: &OutdatedPackages) -> Self {
        let formulae_str = output.formulae.iter().map(|p| format!("{\n}", p)).collect();
        formulae_str
    }
}
impl Display for OutdatedPackages {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(self))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    installed_versions: Vec<String>,
    current_version: String,
    pinned: bool,
    #[serde(default)]
    pinned_version: Option<String>,
}

impl Display for Package {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} => available: {} | installed: {} | pinned: {} | pinned-version: {:?}",
            &self.name,
            &self.current_version,
            &self.installed_versions.join(", "),
            &self.pinned,
            &self.pinned_version
        )
    }
}
