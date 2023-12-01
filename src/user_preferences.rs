use std::{error::Error, fs::{File, self}, io::{BufReader, BufWriter}};

use serde::{Serialize, Deserialize};

use anyhow::Result;

use crate::temperature_unit::TemperatureUnit;

const MAX_KNOWN_PREFERENCES_VERSION: u32 = 1;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub preferences_version: u32,
    pub auto_open_camera: bool,
    pub temperature_unit: TemperatureUnit,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            preferences_version: 1,
            auto_open_camera: true,
            temperature_unit: TemperatureUnit::Celsius,
        }
    }
}

impl UserPreferences {
    pub fn load() -> Result<Self> {
        let mut path = dirs::config_dir().unwrap();
        path.push("thermal-viewer");
        path.push("preferences.json");
        if !path.exists() {
            return Ok(Self::default());
        }
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let prefs: UserPreferences = serde_json::from_reader(reader)?;
        if prefs.preferences_version > MAX_KNOWN_PREFERENCES_VERSION {
            return Err(anyhow::anyhow!("Unknown preferences version {}", prefs.preferences_version));
        }
        Ok(prefs)
    }

    pub fn save(&self) -> Result<()> {
        let mut path = dirs::config_dir().unwrap();
        path.push("thermal-viewer");
        if !path.exists() {
            fs::create_dir_all(&path)?;
        }
        path.push("preferences.json");
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }
}
