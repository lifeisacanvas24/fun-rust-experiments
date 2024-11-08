
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{Read, Write};
use color_eyre::Result;

#[derive(Serialize, Deserialize)]
pub enum StorageType {
    Json,
    Sqlite,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub storage_type: StorageType,
}

impl Config {
    pub fn new() -> Self {
        Self {
            storage_type: StorageType::Json,
        }
    }

    pub fn load() -> Result<Self> {
        let mut file = File::open("config.json")?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let config: Config = serde_json::from_str(&contents)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let json = serde_json::to_string(self)?;
        let mut file = File::create("config.json")?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }
}
