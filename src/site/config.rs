use super::SiteConfig;
use anyhow::Result;
use std::path::Path;

impl SiteConfig {
    pub fn load() -> Result<Self> {
        Self::load_from("site.toml")
    }
    
    pub fn load_from(path: &str) -> Result<Self> {
        // Try to load from specified path, otherwise use defaults
        let config_path = Path::new(path);
        if config_path.exists() {
            let config_str = std::fs::read_to_string(config_path)?;
            let config: SiteConfig = toml::from_str(&config_str)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_str = toml::to_string_pretty(self)?;
        std::fs::write("site.toml", config_str)?;
        Ok(())
    }
}
