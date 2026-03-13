use dirs;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use toml;

#[derive(Debug, Serialize, Deserialize)]
pub enum GameSwitcher {
    #[serde(rename = "Undefined")]
    Undefined,
    #[serde(rename = "HK")]
    HollowKnight,
    #[serde(rename = "SK")]
    SilkSong,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub game_switcher: GameSwitcher,
    pub hollow_knight_path: PathBuf,
    pub silk_song_path: PathBuf,
    pub index_path: PathBuf,
}

impl Config {
    fn create_default() -> Result<String, Box<dyn std::error::Error>> {
        let home = dirs::home_dir().expect("Unable to get home directory");
        let config_dir = home.join(".config/vespa"); // use lowercase consistently
        fs::create_dir_all(&config_dir)?; // ✅ make sure folder exists

        let config = Config {
            game_switcher: GameSwitcher::SilkSong,
            hollow_knight_path: home.join("code/private-git/yato/Vespa/resources/HK"),
            silk_song_path: home.join("code/private-git/yato/Vespa/resources/SK"),
            index_path: config_dir.join("index.json"),
        };

        let serialized = toml::to_string(&config)?;
        fs::write(config_dir.join("config.toml"), &serialized)?; // write after directory exists
        Ok(serialized)
    }

    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let home = dirs::home_dir().expect("Unable to get home directory");
        let config_path = home.join(".config/vespa/config.toml"); // match create_default()

        // If file doesn't exist, create default
        let serialized = fs::read_to_string(&config_path).or_else(|_| Config::create_default())?;
        let config: Config = toml::from_str(&serialized)?;
        Ok(config)
    }
}
