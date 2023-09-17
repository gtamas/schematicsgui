use std::path::Path;

use serde_json::*;
use std::fs::read_to_string;

use crate::settings_utils::SettingsUtils;

#[derive(Debug)]
pub struct Collection {
    data: Value,
}

#[derive(Debug)]
pub struct SchematicData {
    pub schema: String,
    pub description: String,
    pub private: bool,
    pub hidden: bool,
}

impl Collection {
    pub fn new(path: &str) -> Self {
        Collection {
            data: {
                let res: Value;
                if !Path::new(path).exists() {
                    res = from_str("{}").unwrap();
                } else {
                    res = Self::read(path);
                }
                res
            },
        }
    }
    pub fn schema_location(&self) -> &str {
        self.data["$schema"].as_str().unwrap_or("")
    }
    pub fn get_schematic(&self, name: &str) -> SchematicData {
        let empty_map = Map::default();
        let empty_obj = &Value::Object(empty_map);
        let list = self.list();
        let data = list.get(name).unwrap_or(empty_obj);
        SchematicData {
            schema: data["schema"].as_str().unwrap_or("").to_owned(),
            description: data["description"].as_str().unwrap_or("").to_owned(),
            hidden: data["hidden"].as_bool().unwrap_or(false),
            private: data["private"].as_bool().unwrap_or(false),
        }
    }
    fn list(&self) -> Map<String, Value> {
        let settings = SettingsUtils::new().read();
        let empty = Map::default();
        self.data["schematics"]
            .as_object()
            .unwrap_or(&empty)
            .to_owned()
            .into_iter()
            .filter(|a| {
                if settings.show_private == false {
                    return !(a.1["private"] == true);
                }
                true
            })
            .filter(|a| {
                if settings.show_hidden == false {
                    return !(a.1["hidden"] == true);
                }
                true
            })
            .collect()
    }

    pub fn list_schematic_names(&self) -> Vec<String> {
        let mut result = Vec::<String>::new();
        let list = self.list();
        let keys = list.keys().map(|x| x.to_owned());

        for key in keys {
            result.push(key.to_owned())
        }
        result
    }
    pub fn has_schematic(&self, schematic: &str) -> bool {
        let result = match self.list_schematic_names().iter().find(|x| *x == schematic) {
            Some(_) => true,
            None => false,
        };
        result
    }

    pub fn read_str(path: &str) -> String {
        match read_to_string(Path::new(path)) {
            Ok(str) => str.to_string(),
            Err(err) => panic!("Invalid UTF8 input: {:?}", err),
        }
    }

    pub fn read(path: &str) -> Value {
        let contents: Vec<u8> = match std::fs::read(path) {
            Ok(vec) => vec,
            Err(err) => panic!("{}, {:?}", path, err),
        };
        match from_str(match &std::str::from_utf8(&contents) {
            Ok(str) => &str.to_owned(),
            Err(err) => panic!("Invalid UTF8 input: {:?}", err),
        }) {
            Ok(json) => json,
            Err(err) => panic!("Invalid JSON: {:?}!", err),
        }
    }
}
