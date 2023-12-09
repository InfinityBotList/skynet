use once_cell::sync::Lazy;
use poise::serenity_prelude::UserId;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Write};

use crate::Error;

/// Global config object
pub static CONFIG: Lazy<Config> = Lazy::new(|| Config::load().expect("Failed to load config"));

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub token: String,
    pub proxy_url: String,
    pub frontend_url: String,
    pub owners: Vec<UserId>,
    pub client_id: String,
    pub client_secret: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_url: String::from(""),
            token: String::from(""),
            client_id: String::from(""),
            client_secret: String::from(""),
            proxy_url: String::from("http://127.0.0.1:3219"),
            frontend_url: String::from("https://skynet.infinitybots.gg"),
            owners: vec![
                UserId::new(728871946456137770),
                UserId::new(510065483693817867),
            ],
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, Error> {
        // Delete config.yaml.sample if it exists
        if std::path::Path::new("config.yaml.sample").exists() {
            std::fs::remove_file("config.yaml.sample")?;
        }

        // Create config.yaml.sample
        let mut sample = File::create("config.yaml.sample")?;

        // Write default config to config.yaml.sample
        sample.write_all(serde_yaml::to_string(&Config::default())?.as_bytes())?;

        // Open config.yaml
        let file = File::open("config.yaml");

        match file {
            Ok(file) => {
                // Parse config.yaml
                let cfg: Config = serde_yaml::from_reader(file)?;

                // Return config
                Ok(cfg)
            }
            Err(e) => {
                // Print error
                println!("config.yaml could not be loaded: {}", e);

                // Exit
                std::process::exit(1);
            }
        }
    }
}
