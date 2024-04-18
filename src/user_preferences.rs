use std::{
    fs::{self, File},
    io::{BufReader, BufWriter},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

use anyhow::Result;

use crate::temperature::TemperatureUnit;

/// Denotes the maximum known version of the preferences file for this version of the application.
///
/// Version 1: Initial version.
/// Version 2: Added `captures_directory`.
const MAX_KNOWN_PREFERENCES_VERSION: u32 = 2;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UserPreferences {
    pub preferences_version: u32,
    pub temperature_unit: TemperatureUnit,
    pub auto_open_camera: bool,
    pub show_unsupported_cameras: bool,
    pub captures_directory: String,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            preferences_version: 1,
            temperature_unit: TemperatureUnit::Celsius,
            auto_open_camera: true,
            show_unsupported_cameras: false,
            captures_directory: dirs::picture_dir()
                .unwrap_or(dirs::home_dir().unwrap_or(PathBuf::from("./")))
                .join("Thermal Cat")
                .to_string_lossy()
                .to_string(),
        }
    }
}

impl UserPreferences {
    pub fn preferences_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap();
        path.push("thermal-viewer");
        path.push("preferences.json");
        path
    }
    pub fn load() -> Result<Self> {
        let path = Self::preferences_path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let prefs: UserPreferences = serde_json::from_reader(reader)?;
        if prefs.preferences_version > MAX_KNOWN_PREFERENCES_VERSION {
            return Err(anyhow::anyhow!(
                "Unknown preferences version {}. Please update the application or remove the preferences file.",
                prefs.preferences_version
            ));
        }

        let mut did_migration = false;

        let prefs = if prefs.preferences_version < 2 {
            did_migration = true;
            log::info!("Migrating preferences to version 2");
            UserPreferences {
                preferences_version: 2,
                captures_directory: Self::default().captures_directory,
                ..prefs
            }
        } else {
            prefs
        };

        // More migrations here...

        if did_migration {
            prefs.save()?;
        }

        Ok(prefs)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::preferences_path();
        let dir_path = path.parent().unwrap();
        if !dir_path.exists() {
            fs::create_dir_all(&dir_path)?;
        }

        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }
}
