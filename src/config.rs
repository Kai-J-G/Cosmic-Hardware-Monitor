//! Persisted settings, stored through `cosmic-config`.

use cosmic::cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, CosmicConfigEntry};
use serde::{Deserialize, Serialize};

use crate::hardware::types::TemperatureUnit;

pub const APP_ID: &str = "io.github.kai_j_g.CosmicHardwareMonitor";
pub const CONFIG_VERSION: u64 = 1;

/// Whether the popup follows the desktop theme or is pinned to one appearance.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, Default)]
pub enum ThemePreference {
    #[default]
    System,
    Dark,
    Light,
}

impl ThemePreference {
    /// The theme to apply, or `None` when the desktop's own theme should win.
    pub fn theme(self) -> Option<cosmic::Theme> {
        let mut theme = match self {
            Self::System => return None,
            Self::Dark => cosmic::theme::system_dark(),
            Self::Light => cosmic::theme::system_light(),
        };
        // Applet popups blur whatever is behind them.
        theme.transparent = true;
        Some(theme)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, CosmicConfigEntry)]
#[version = 1]
pub struct HardwareMonitorConfig {
    pub fahrenheit: bool,
    pub refresh_interval_secs: u64,
    pub theme_pref: ThemePreference,
}

impl HardwareMonitorConfig {
    pub fn unit(&self) -> TemperatureUnit {
        if self.fahrenheit {
            TemperatureUnit::Fahrenheit
        } else {
            TemperatureUnit::Celsius
        }
    }
}

impl Default for HardwareMonitorConfig {
    fn default() -> Self {
        Self {
            fahrenheit: false,
            refresh_interval_secs: 2,
            theme_pref: ThemePreference::System,
        }
    }
}
