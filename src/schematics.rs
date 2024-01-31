use std::path::Path;

use serde_json::*;
use std::fs::read_to_string;

use crate::settings_utils::SettingsData;

#[derive(Debug)]
pub struct Collection {
    data: Value,
    settings: SettingsData,
}

#[derive(Default, Debug, PartialEq)]
pub struct SchematicData {
    pub schema: String,
    pub description: String,
    pub private: bool,
    pub hidden: bool,
}

impl Collection {
    pub fn new(settings: SettingsData) -> Self {
        Collection {
            settings,
            data: from_str("{}").unwrap(),
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
            schema: data["schema"].as_str().unwrap_or_default().to_owned(),
            description: data["description"].as_str().unwrap_or_default().to_owned(),
            hidden: data["hidden"].as_bool().unwrap_or_default(),
            private: data["private"].as_bool().unwrap_or_default(),
        }
    }
    fn list(&self) -> Map<String, Value> {
        let empty = Map::default();
        self.data["schematics"]
            .as_object()
            .unwrap_or(&empty)
            .to_owned()
            .into_iter()
            .filter(|a| {
                if !self.settings.show_private {
                    return !(a.1["private"] == true);
                }
                true
            })
            .filter(|a| {
                if !self.settings.show_hidden {
                    return !(a.1["hidden"] == true);
                }
                true
            })
            .collect()
    }

    pub fn init(&mut self) -> &Collection {
        self.data = Self::read(&self.settings.schematics_collection);
        self
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
        self.list_schematic_names().iter().any(|x| x == schematic)
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
            Ok(str) => str.to_owned(),
            Err(err) => panic!("Invalid UTF8 input: {:?}", err),
        }) {
            Ok(json) => json,
            Err(err) => panic!("Invalid JSON: {:?}!", err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_collection() -> Collection {
        let settings = SettingsData::default();
        let mut collection = Collection::new(settings);
        collection.data = serde_json::from_str("{\"$schema\": \"location\", \"schematics\": {\"sc1\": {\"description\": \"sc1 desc\"}, \"sc2\": {}, \"sc3\": {\"private\": true}, \"sc4\": {\"hidden\": true}}}").unwrap();
        collection
    }

    #[test]
    fn schema_location_success() {
        let collection = get_collection();
        let loc = collection.schema_location();

        assert_eq!(loc, "location")
    }

    #[test]
    fn schema_location_fail_no_schema() {
        let mut collection = get_collection();

        collection.data = serde_json::from_str("{}").unwrap();
        let loc = collection.schema_location();

        assert_eq!(loc, "")
    }

    #[test]
    fn has_schematic_should_return_true() {
        let collection = get_collection();
        let result = collection.has_schematic("sc1");

        assert!(result)
    }

    #[test]
    fn has_schematic_should_return_false() {
        let collection = get_collection();
        let result = collection.has_schematic("none");

        assert!(!result)
    }

    #[test]
    fn list_schematic_names_success() {
        let collection = get_collection();
        let result = collection.list_schematic_names();

        assert_eq!(result, vec!["sc1", "sc2"])
    }

    #[test]
    fn list_success() {
        let collection = get_collection();
        let result = collection.list();

        assert_eq!(result.len(), 2);
        assert!(result.get("sc1").is_some());
        assert!(result.get("sc2").is_some());
    }

    #[test]
    fn list_private_hidden_not_shown() {
        let collection = get_collection();
        let result = collection.list();

        assert_eq!(result.len(), 2);
        assert!(result.get("sc3").is_none());
        assert!(result.get("sc4").is_none());
    }

    #[test]
    fn list_private_hidden_shown() {
        let mut collection = get_collection();
        collection.settings.show_private = true;
        collection.settings.show_hidden = true;
        let result = collection.list();

        assert_eq!(result.len(), 4);
        assert!(result.get("sc1").is_some());
        assert!(result.get("sc2").is_some());
        assert!(result.get("sc3").is_some());
        assert!(result.get("sc4").is_some());
    }

    #[test]
    fn get_schematic_success() {
        let collection = get_collection();
        let result = collection.get_schematic("sc1");

        assert_eq!(result, SchematicData {
          description: String::from("sc1 desc"),
          ..Default::default()
        });
    }

     #[test]
    fn get_schematic_not_found_returns_default() {
        let collection = get_collection();
        let result = collection.get_schematic("none");

        assert_eq!(result, SchematicData::default());
    }
}
