use dirs;
use serde::{Deserialize, Serialize};
use std::{fmt, fs, path::PathBuf};
use thiserror::Error;
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

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    TomlError(#[from] toml::ser::Error),
    #[error("Unable to load config: {0}")]
    LoadError(#[from] toml::de::Error),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub game_switcher: GameSwitcher,
    pub sk_default_profile: Option<String>,
    pub hk_default_profile: Option<String>,
    pub hollow_knight_path: PathBuf,
    pub silk_song_path: PathBuf,
    pub index_path: PathBuf,
}

impl Config {
    fn create_default() -> Result<String, ConfigError> {
        let home = dirs::home_dir().expect("Unable to get home directory");
        let config_dir = home.join(".config/vespa");
        fs::create_dir_all(&config_dir)?;

        let config = Config {
            game_switcher: GameSwitcher::SilkSong,
            sk_default_profile: None,
            hk_default_profile: None,
            hollow_knight_path: home.join("code/git/yato/Vespa/resources/HK"),
            silk_song_path: home.join("code/git/yato/Vespa/resources/SK"),
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

    pub fn load() -> Result<Self, ConfigError> {
        let config_path = Self::config_path();

        // If file doesn't exist, create default
        let serialized = fs::read_to_string(&config_path).or_else(|_| Config::create_default())?;
        let config: Config = toml::from_str(&serialized)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let serialized = toml::to_string(&self)?;
        fs::write(Self::config_path(), &serialized)?;
        Ok(())
    }

    pub fn set_default_profile(
        &mut self,
        game: &GameSwitcher,
        profile: String,
    ) -> Result<(), ConfigError> {
        match game {
            GameSwitcher::HollowKnight => self.hk_default_profile = Some(profile),
            GameSwitcher::SilkSong => self.sk_default_profile = Some(profile),
        }
        self.save()?;
        Ok(())
    }

    pub fn clear_default_profile(&mut self, game: &GameSwitcher) -> Result<(), ConfigError> {
        match game {
            GameSwitcher::HollowKnight => self.hk_default_profile = None,
            GameSwitcher::SilkSong => self.sk_default_profile = None,
        }
        self.save()?;
        Ok(())
    }

    pub fn get_default_profile(&self, game: &GameSwitcher) -> Option<String> {
        match game {
            GameSwitcher::HollowKnight => self.hk_default_profile.clone(),
            GameSwitcher::SilkSong => self.sk_default_profile.clone(),
        }
    }
}
