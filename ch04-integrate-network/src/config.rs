
pub mod dotenv {

    pub fn config(key: String) -> Option<String>  {
        let value = std::env::var(key);
        match value {
            Ok(v) => Some(v),
            Err(_) => None,
        }
    }

    pub fn get_groq_key() -> Option<String> {
        return config(String::from("GROQ_API_KEY"));
    }
}

pub mod toml {
    use serde::Deserialize;
    use std::fs;
    use toml;

    #[derive(Deserialize)]
    pub struct Settings {
        settings: SettingsData,
    }
    
    #[derive(Deserialize)]
    struct SettingsData {
        GROQ_API_KEY: Option<String>,
    }
    
    pub fn config() -> Settings {
        let config_content = fs::read_to_string("config.toml").expect("Failed to read config file");
        match toml::de::from_str(&config_content) {
            Ok(settings) => settings,
            Err(e) => panic!("Failed to parse config file: {}", e),
        }
        // return toml::de::from_str(&config_content).expect("Failed to parse config file");
    }

    pub fn get_groq_key() -> Option<String> {
        let settings = config();
        return settings.settings.GROQ_API_KEY;
    }
    
}
