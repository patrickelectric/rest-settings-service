use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use chrono;
use hex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha1::{Digest, Sha1};
use toml;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Header {
    pub name: String,
    pub modified: bool,
    pub hash: String, // File's sha1
    pub date: String, // ISO 8601 / RFC 3339 date & time format.
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Content {
    pub header: Header,
    // This needs to be done with toml, otherwise it fails to generate pretty string
    pub settings: Option<toml::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SettingsManager {
    pub path: String,
    pub settings: serde_json::Map<String, serde_json::Value>,
}

impl Default for SettingsManager {
    fn default() -> Self {
        SettingsManager {
            path: format!("~/.config/{}", env!("CARGO_PKG_NAME")),
            settings: serde_json::Map::new(),
        }
    }
}

impl SettingsManager {
    /// Create a new SettingsManager object with a proper initialization
    pub fn new(path: Option<String>) -> Self {
        let mut this = SettingsManager::default();
        if path.is_some() {
            this.path = path.unwrap();
        }
        let _ = this.init();
        this.load();
        return this;
    }

    /// Do the object initialization
    fn init(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(self.get_default_folder())
    }

    /// Get path that contains the default configuration files
    pub fn get_default_folder(&self) -> PathBuf {
        Path::new(&self.path).join("default")
    }

    /*
    pub fn read_setting(&self, name: &str) -> Option<toml::Value> {
        if !self.settings.contains_key(name) {
            return None;
        }

        if !name.contains("/") {
            return Some(self.settings[name]);
        }
        return self.settings;
    }*/

    /// TODO: return something
    pub fn push(&mut self, mut content: Content) {
        //let item = &self.settings[&content.header.name];
        if self.settings.contains_key(&content.header.name) {
            println!("Item already exist: {}", content.header.name);
            return;
        }
        content.header.date = chrono::Local::now().to_string();
        content.header.modified = false;

        let mut hasher = Sha1::new();
        hasher.input(toml::to_string_pretty(&content).unwrap_or_else(|error| {
            panic!("Failed to parse content to create hash: {:#?}\n{:#?}", error, content)
        }));
        content.header.hash = hex::encode(hasher.result());

        self.settings.insert(content.header.name.clone(), serde_json::to_value(&content).unwrap_or_else(|error| {
            panic!("Failed to parse content to json: {:#?}\n{:#?}", error, content)
        }));
    }

    /// Load all settings available in the manager path
    pub fn load(&mut self) {
        let files = std::fs::read_dir(&self.path).unwrap();
        let files = files
            .filter_map(Result::ok)
            .filter(|file| match file.path().extension() {
                Some(extension) => extension.to_str() == Some("toml"),
                None => false,
            });

        for file in files {
            let mut contents = String::new();
            let mut file = File::open(file.path())
                .unwrap_or_else(|error| panic!("Failed to open settings file: {:#?}", error));
            file.read_to_string(&mut contents)
                .unwrap_or_else(|error| panic!("Failed to read settings file: {:#?}", error));

            if contents.is_empty() {
                println!("File '{:#?}' is empty.", file);
                return;
            }

            let content: Content = toml::from_str(&contents)
                .unwrap_or_else(|error| panic!("Failed to update settings vector: {:#?}\n{:#?}\n{:#?}", error, &file, &contents));
            self.settings.insert(content.header.name.clone(), serde_json::to_value(&content).unwrap_or_else(|error| {
                panic!("Failed to parse content to json: {:#?}\n{:#?}", error, content)
            }));
        }
    }

    /// Save all settings available in the manager path
    pub fn save(&self) {
        for (name, setting) in self.settings.clone() {
            // Create file name for the settings file
            let mut file_name = Path::new(&self.path).join(&name);
            file_name.set_extension("toml");

            // Do the same thing for the default file
            let mut default_file_name = self.get_default_folder().join(&name);
            default_file_name.set_extension("toml");

            // Create settings file if it exist or not, we are going to rewrite the data
            let mut file = File::create(&file_name).unwrap_or_else(|error| {
                panic!("Failed to create settings file: {:#?}\n{:#?}", error, file_name);
            });

            // Create the string that will be in the file
            let toml_string = toml::to_string_pretty(&serde_json::from_value::<Content>(setting).unwrap())
                .unwrap_or_else(|error| panic!("Failed to parse settings to toml: {:#?}", error));

            let _ = file.write_all(&toml_string.as_bytes());

            // By default, we only create the default settings file if it does not exist yet
            if !default_file_name.exists() {
                let mut default_file = File::create(default_file_name).unwrap_or_else(|error| {
                    panic!("Failed to create default file: {:#?}", error);
                });
                default_file
                    .write_all(&toml_string.as_bytes())
                    .unwrap_or_else(|error| {
                        panic!("Failed to populate default file: {:#?}", error);
                    });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_path() -> String {
        std::env::temp_dir().to_str().unwrap().to_string()
    }

    #[test]
    fn simple() {
        println!("Test safe..");
        save();
        println!("Test load..");
        load();
    }

    fn save() {
        let mut settings_manager = SettingsManager::new(Some(create_path()));
        settings_manager.settings = serde_json::Map::new();

        println!("Check settings path: {}", settings_manager.path);
        assert!(Path::new(&settings_manager.path).exists());

        // Create fake settings
        let json_example = r#"
            {
                "name": "John Doe",
                "age": 43,
                "address": {
                    "street": "10 Downing Street",
                    "city": "London"
                },
                "phones": [
                    "+44 1234567",
                    "+44 2345678"
                ]
            }
        "#;
        let json_example: serde_json::Value = serde_json::from_str(json_example).unwrap();
        let toml_example = toml::Value::try_from(&json_example).unwrap();

        // Create a fake service with our fake settings
        let mut content = Content::default();
        content.header.name = "test".to_string();
        content.settings = Some(toml_example);
        settings_manager.push(content);
        settings_manager.save();

        //&serde_json::from_value::<Content>(settings_manager.settings["test"]).unwrap()
        let test_settings = serde_json::from_value::<Content>(settings_manager.settings["test"].clone()).unwrap();
        // Check file
        let content_toml_string =
            toml::to_string_pretty(&test_settings).unwrap();

        let mut file_name = Path::new(&settings_manager.path).join("test");
        file_name.set_extension("toml");
        println!(
            "Check if settings file exist and content matches: {:?}",
            file_name
        );
        assert!(file_name.exists());

        let mut file_content = String::new();
        let mut file = File::open(file_name).unwrap();
        file.read_to_string(&mut file_content).unwrap();

        assert_eq!(file_content, content_toml_string);
    }

    fn load() {
        let settings_manager = SettingsManager::new(Some(create_path()));
        let settings = &settings_manager.settings["test"]["settings"];
        //let settings = content.settings;

        println!("Check test file contents.. {:#?}", settings);
        assert_eq!(settings["name"].as_str().unwrap(), "John Doe");
        assert_eq!(settings["age"].as_i64().unwrap(), 43);
        assert_eq!(settings["address"]["city"].as_str().unwrap(), "London");
        assert_eq!(settings["phones"][1].as_str().unwrap(), "+44 2345678");
    }
}
