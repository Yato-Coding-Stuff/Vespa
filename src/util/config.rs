use dirs;
use serde::{Deserialize, Serialize};
use std::{fmt, fs, path::PathBuf};
use toml;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameSwitcher {
    #[serde(rename = "HK")]
    HollowKnight,
    #[serde(rename = "SK")]
    SilkSong,
}

impl fmt::Display for GameSwitcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            GameSwitcher::HollowKnight => "Hollow Knight",
            GameSwitcher::SilkSong => "Silk Song",
        };
        write!(f, "{}", name)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub game_switcher: GameSwitcher,
    pub hollow_knight_path: PathBuf,
    pub default_hollow_knight_profile: String,
    pub silk_song_path: PathBuf,
    pub default_silk_song_profile: String,
    pub index_path: PathBuf,
}

impl Config {
    fn create_default() -> Result<String, Box<dyn std::error::Error>> {
        let home = dirs::home_dir().expect("Unable to get home directory");
        let config_dir = home.join(".config/vespa");
        fs::create_dir_all(&config_dir)?;

        let config = Config {
            game_switcher: GameSwitcher::SilkSong,
            hollow_knight_path: home.join("code/git/yato/Vespa/resources/HK"),
            default_hollow_knight_profile: "default".to_string(),
            silk_song_path: home.join("code/git/yato/Vespa/resources/SK"),
            default_silk_song_profile: "default".to_string(),
            index_path: config_dir.join("index.json"),
        };

        let serialized = toml::to_string(&config)?;
        fs::write(config_dir.join("config.toml"), &serialized)?;
        Ok(serialized)
    }

    pub fn config_dir() -> PathBuf {
        let home = dirs::home_dir().expect("Unable to get home directory");
        home.join(".config/vespa")
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::config_path();

        // If file doesn't exist, create default
        let serialized = fs::read_to_string(&config_path).or_else(|_| Config::create_default())?;
        let config: Config = toml::from_str(&serialized)?;
        Ok(config)
    }
}
