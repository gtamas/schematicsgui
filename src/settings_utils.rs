use serde::{Deserialize, Serialize};
use std::fs::{read_to_string, write};
use std::path::{Path, PathBuf};

pub struct SettingsUtils;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SettingsData {
    pub node_location: String,
    pub runner_location: String,
    pub schematics_collection: String,
    pub schematics_package: String,
    pub show_private: bool,
    pub show_hidden: bool,
    pub google_runner: bool,
    pub mbh_runner: bool,
    pub custom_runner: bool,
}

impl SettingsUtils {
    pub fn new() -> Self {
        SettingsUtils {}
    }

    pub fn exists(&self) -> bool {
        let config_dir = self.get_config_dir();
        let path = Path::new(&config_dir).join("./settings.toml");
        path.exists()
    }

    fn get_config_dir(&self) -> PathBuf {
        let home_dir = match std::env::var_os("HOME") {
            None => std::env::current_dir().unwrap().as_os_str().to_owned(),
            Some(s) => s,
        };
        Path::new(&home_dir).join("./schematics-gui").to_owned()
    }

    pub fn init(&self) -> () {
        let config_dir: PathBuf = self.get_config_dir();

        if !config_dir.exists() {
            match std::fs::create_dir(config_dir) {
                Ok(s) => s,
                Err(err) => panic!("Could not create settings dir! {}", err),
            }
        }
    }

    pub fn write(&self, model: &SettingsData) {
        let toml = toml::to_string(&model).unwrap();
        let config_dir = self.get_config_dir();
        let path = Path::new(&config_dir).join("./settings.toml");
        match write(path.as_os_str(), toml) {
            Ok(s) => s,
            Err(err) => panic!("Could not save settings! {}", err),
        }
    }

    pub fn read(&self) -> SettingsData {
        let config_dir = self.get_config_dir();
        let path = Path::new(&config_dir).join("./settings.toml");
        let contents = match read_to_string(path) {
            Ok(data) => data,
            Err(err) => panic!("Could not read settings! {}", err),
        };
        let settings: SettingsData = toml::from_str(&contents).unwrap();
        settings
    }
}