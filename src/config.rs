use cosmic::cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, CosmicConfigEntry};
use serde::{Deserialize, Serialize};

pub const APP_ID: &str = "com.github.neojakey.cosmic-ext-temptyle";
pub const CONFIG_VERSION: u64 = 1;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, Default)]
pub enum ThemePreference {
    #[default]
    System,
    Dark,
    Light,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, CosmicConfigEntry)]
#[version = 1]
pub struct TempTyleConfig {
    pub fahrenheit: bool,
    pub refresh_interval_secs: u64,
    pub theme_pref: ThemePreference,
}

impl Default for TempTyleConfig {
    fn default() -> Self {
        Self {
            fahrenheit: false,
            refresh_interval_secs: 2,
            theme_pref: ThemePreference::System,
        }
    }
}
