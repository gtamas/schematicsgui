use serde::{Deserialize, Serialize};
use std::fs::{read_to_string, write};
use std::path::{Path, PathBuf};

pub struct SettingsUtils;

impl Default for SettingsUtils {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Runner {
    Google,
    MBH,
    Custom,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SettingsData {
    pub runner: Runner,
    pub node_binary: String,
    pub runner_location: String,
    pub schematics_collection: String,
    pub schematics_package: String,
    pub show_private: bool,
    pub show_hidden: bool,
}

impl Default for SettingsData {
    fn default() -> Self {
        SettingsData {
            node_binary: String::default(),
            runner_location: String::default(),
            schematics_collection: String::default(),
            schematics_package: String::default(),
            show_private: false,
            show_hidden: false,
            runner: Runner::Google,
        }
    }
}

impl SettingsUtils {
    pub fn new() -> Self {
        SettingsUtils {}
    }

    pub fn exists(&self) -> bool {
        let config_dir = Self::get_config_dir();
        let path = Path::new(&config_dir).join("./settings.toml");
        path.exists()
    }

    pub fn get_config_dir() -> PathBuf {
        let home_dir = match std::env::var_os("HOME") {
            None => std::env::current_dir().unwrap().as_os_str().to_owned(),
            Some(s) => s,
        };
        Path::new(&home_dir).join("schematics-gui").to_owned()
    }

    pub fn init(&self) {
        let config_dir: PathBuf = Self::get_config_dir();

        if !config_dir.exists() {
            match std::fs::create_dir(config_dir) {
                Ok(s) => s,
                Err(err) => panic!("Could not create settings dir! {}", err),
            }
        }
    }

    pub fn write(&self, model: &SettingsData) {
        let toml = toml::to_string(&model).unwrap();
        let config_dir = Self::get_config_dir();
        let path = Path::new(&config_dir).join("./settings.toml");
        match write(path.as_os_str(), toml) {
            Ok(s) => s,
            Err(err) => panic!("Could not save settings! {}", err),
        }
    }

    pub fn read(&self) -> SettingsData {
        let config_dir = Self::get_config_dir();
        let path = Path::new(&config_dir).join("./settings.toml");
        let contents = match read_to_string(path) {
            Ok(data) => data,
            Err(err) => panic!("Could not read settings! {}", err),
        };
        let settings: SettingsData = toml::from_str(&contents).unwrap();
        settings
    }
}

#[cfg(test)]
mod tests {
    use std::env::{remove_var, set_var, temp_dir};

    use super::*;

    #[test]
    fn get_config_dir_without_home() {
        remove_var("HOME");

        let current_dir = std::env::current_dir().unwrap();
        let dir = SettingsUtils::get_config_dir();
        assert_eq!(dir, current_dir.join("schematics-gui"));
    }

    #[test]
    fn get_config_dir_with_home() {
        set_var("HOME", "test");

        let dir = SettingsUtils::get_config_dir();
        assert_eq!(dir, PathBuf::new().join("test").join("schematics-gui"));
    }

    #[test]
    #[should_panic(expected = "Could not create settings dir!")]
    fn init_fail() {
        set_var("HOME", "/tmp/no/such/dir");

        let settings = SettingsUtils::default();
        settings.init();
    }

    #[test]
    fn init_success() {
        set_var("HOME", temp_dir());

        let settings = SettingsUtils::default();
        settings.init();
        let cfg_dir = SettingsUtils::get_config_dir();
        assert!(cfg_dir.exists())
    }

    #[test]
    fn exists_fail() {
        set_var("HOME", "/tmp/no/dir");

        let settings = SettingsUtils::default();

        let result = settings.exists();
        assert!(!result);
    }

    #[test]
    fn exists_success() {
        set_var("HOME", temp_dir());

        let settings = SettingsUtils::default();
        let data = SettingsData::default();
        settings.init();
        settings.write(&data);

        let result = settings.exists();
        assert!(result);
    }

    #[test]
    #[should_panic(expected = "Could not save settings!")]
    fn write_fail() {
        set_var("HOME", "/tmp/no/dir");

        let settings = SettingsUtils::default();
        let data = SettingsData::default();
        settings.write(&data);
    }

    #[test]
    fn write_success() {
        set_var("HOME", temp_dir());

        let settings = SettingsUtils::default();
        let data = SettingsData::default();
        settings.write(&data);
        let cfg_dir = SettingsUtils::get_config_dir();
        assert!(cfg_dir.join("settings.toml").exists())
    }

    #[test]
    #[should_panic(expected = "Could not read settings!")]
    fn read_fail() {
        set_var("HOME", "/tmp/no/dir");

        let settings = SettingsUtils::default();
        settings.read();
    }

    #[test]
    fn read_success() {
        set_var("HOME", temp_dir());

        let settings = SettingsUtils::default();
        let data = SettingsData::default();
        settings.init();
        settings.write(&data);
        let loaded = settings.read();
        assert_eq!(loaded, data)
    }
}
