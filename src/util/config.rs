use dirs;
use serde::{Deserialize, Serialize};
use std::{
    fmt, fs,
    path::{Path, PathBuf},
};
use thiserror::Error;
use toml;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameSwitcher {
    #[serde(rename = "HK")]
    HollowKnight,
    #[serde(rename = "SK")]
    SilkSong,
}

impl GameSwitcher {
    pub fn display_name(&self) -> &'static str {
        match self {
            GameSwitcher::HollowKnight => "Hollow Knight",
            GameSwitcher::SilkSong => "Silk Song",
        }
    }

    pub fn profile_dir_name(&self) -> &'static str {
        match self {
            GameSwitcher::HollowKnight => "HK",
            GameSwitcher::SilkSong => "SK",
        }
    }
}

impl fmt::Display for GameSwitcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
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
        Self::create_default_in_dir(&Self::config_dir())
    }

    fn create_default_in_dir(config_dir: &Path) -> Result<String, ConfigError> {
        let home = dirs::home_dir().expect("Unable to get home directory");
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
        Self::load_from_path(&Self::config_path())
    }

    pub fn load_from_path(config_path: &Path) -> Result<Self, ConfigError> {
        let config_dir = config_path
            .parent()
            .expect("Config path should have a parent directory");

        // If file doesn't exist, create default
        let serialized =
            fs::read_to_string(config_path).or_else(|_| Self::create_default_in_dir(config_dir))?;
        let config: Config = toml::from_str(&serialized)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        self.save_to_path(&Self::config_path())
    }

    pub fn save_to_path(&self, config_path: &Path) -> Result<(), ConfigError> {
        let serialized = toml::to_string(&self)?;
        fs::write(config_path, &serialized)?;
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

    pub fn get_game_path(&self, game: &GameSwitcher) -> &PathBuf {
        match game {
            GameSwitcher::HollowKnight => &self.hollow_knight_path,
            GameSwitcher::SilkSong => &self.silk_song_path,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Config, GameSwitcher};
    use tempfile::tempdir;

    fn sample_config() -> Config {
        Config {
            game_switcher: GameSwitcher::SilkSong,
            sk_default_profile: Some("sk-default".to_string()),
            hk_default_profile: Some("hk-default".to_string()),
            hollow_knight_path: "/games/hk".into(),
            silk_song_path: "/games/sk".into(),
            index_path: "/config/index.json".into(),
        }
    }

    #[test]
    fn game_switcher_helpers_return_expected_names() {
        assert_eq!(GameSwitcher::HollowKnight.display_name(), "Hollow Knight");
        assert_eq!(GameSwitcher::SilkSong.display_name(), "Silk Song");
        assert_eq!(GameSwitcher::HollowKnight.profile_dir_name(), "HK");
        assert_eq!(GameSwitcher::SilkSong.profile_dir_name(), "SK");
    }

    #[test]
    fn save_and_load_round_trip_through_custom_path() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        let config = sample_config();

        config.save_to_path(&config_path).unwrap();
        let loaded = Config::load_from_path(&config_path).unwrap();

        assert_eq!(loaded.game_switcher, GameSwitcher::SilkSong);
        assert_eq!(
            loaded.get_default_profile(&GameSwitcher::SilkSong),
            Some("sk-default".to_string())
        );
        assert_eq!(
            loaded.get_default_profile(&GameSwitcher::HollowKnight),
            Some("hk-default".to_string())
        );
        assert_eq!(
            loaded.hollow_knight_path,
            sample_config().hollow_knight_path
        );
    }

    #[test]
    fn load_from_missing_path_creates_default_config() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let loaded = Config::load_from_path(&config_path).unwrap();

        assert_eq!(loaded.game_switcher, GameSwitcher::SilkSong);
        assert_eq!(loaded.get_default_profile(&GameSwitcher::SilkSong), None);
        assert!(config_path.exists());
    }

    #[test]
    fn get_default_profile_reads_correct_slot() {
        let config = sample_config();

        assert_eq!(
            config.get_default_profile(&GameSwitcher::SilkSong),
            Some("sk-default".to_string())
        );
        assert_eq!(
            config.get_default_profile(&GameSwitcher::HollowKnight),
            Some("hk-default".to_string())
        );
    }

    #[test]
    fn get_game_path_reads_correct_slot() {
        let config = sample_config();

        assert_eq!(
            config.get_game_path(&GameSwitcher::SilkSong),
            &config.silk_song_path
        );
        assert_eq!(
            config.get_game_path(&GameSwitcher::HollowKnight),
            &config.hollow_knight_path
        );
    }
}
