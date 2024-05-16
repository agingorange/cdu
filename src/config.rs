use std::env;
use std::fmt;
use std::fs;
use std::io::Write;
use std::net::Ipv4Addr;
use std::path::PathBuf;

use anyhow::Context;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

const CONFIG_DIR_LOCAL: &str = ".";
const CONFIG_DIR_DOCKER: &str = "/config";
const CONFIG_FILE: &str = "cdu.toml";

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub outside_ip: Option<Ipv4Addr>,
    pub cloudflare_ip: Option<Ipv4Addr>,
    pub last_updated: DateTime<Utc>,
    pub save_dir: PathBuf,
    pub file_name: String,
}

impl Default for Config {
    fn default() -> Self {
        let config_dir = if env::var("DOCKER_RUNTIME").is_ok() {
            CONFIG_DIR_DOCKER
        } else {
            CONFIG_DIR_LOCAL
        };

        Self {
            outside_ip: None,
            cloudflare_ip: None,
            last_updated: Utc::now(),
            save_dir: PathBuf::from(config_dir),
            file_name: String::from(CONFIG_FILE),
        }
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Config {{ outside_ip: {}, cloudflare_ip: {}, last_updated: {}, save_dir: {}, file_name: {} }}",
            self.outside_ip
                .map_or_else(|| String::from("None"), |ip| ip.to_string()),
            self.cloudflare_ip
                .map_or_else(|| String::from("None"), |ip| ip.to_string()),
            self.last_updated,
            self.save_dir.display(),
            self.file_name
        )
    }
}

impl Config {
    /// Loads the configuration file if it exists.
    /// The file won't exist on the first run, and we log a message in that case, as it could be
    /// an error if it's not the first run.
    ///
    /// # Errors
    ///
    /// Returns an error if the file exists but cannot be read or parsed.
    pub fn load(&mut self) -> anyhow::Result<()> {
        let config_path = self.save_dir.join(&self.file_name);
        let config_dir = config_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Failed to get config directory: {config_path:?}"))?;

        // Check if there are any issues with the config directory
        config_dir.metadata().map_err(|e| {
            anyhow::anyhow!("Problem with config directory: {config_dir:?}. Error: {e:?}")
        })?;

        if config_path.exists() {
            // If the file exists, proceed with loading
            let file_content = fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read file: {config_path:?}"))?;
            let config: Self = toml::from_str(&file_content)
                .with_context(|| format!("Failed to parse JSON from file: {config_path:?}"))?;
            log::debug!("Loaded config from: {} ({})", config_path.display(), config);

            self.outside_ip = config.outside_ip;
            self.cloudflare_ip = config.cloudflare_ip;
            self.last_updated = config.last_updated;
        } else {
            // If the file does not exist, do nothing and keep the current Config
            log::debug!("Config file does not exist: {config_path:?}");
        }

        Ok(())
    }

    /// Saves the configuration to a file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be created or written to.
    pub fn save(&self) -> anyhow::Result<()> {
        let config_path = self.save_dir.join(&self.file_name);
        let config_toml = toml::to_string_pretty(self)
            .with_context(|| format!("Failed to serialize Config to TOML: {:?}", &config_path))?;
        let mut file = fs::File::create(&config_path).map_err(|e| {
            anyhow::anyhow!(
                "Failed to create file: {:?}. Error: {:?}, Error kind: {:?}",
                config_path,
                e,
                e.kind()
            )
        })?;

        log::debug!("config: {}", self);

        file.write_all(config_toml.as_bytes())
            .with_context(|| format!("Failed to write to file: {config_path:?}"))?;

        log::debug!("Config saved to: {config_path:?}");

        Ok(())
    }
}

#[test]
fn test_load() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join(CONFIG_FILE);
    let mut config = Config {
        save_dir: dir.path().to_path_buf(),
        file_name: String::from(CONFIG_FILE),
        ..Default::default()
    };

    // Test with a non-existent file
    let result = config.load();
    assert!(
        result.is_ok(),
        "Expected Ok(()) when loading non-existent file, got {result:?}"
    );

    // Test with a valid file
    let file_content = r#"
        outside_ip = "1.2.3.4"
        cloudflare_ip = "1.2.3.4"
        last_updated = "2024-03-10T13:54:04.032435Z"
        save_dir = "/config"
        file_name = "cdu.toml"
    "#;
    fs::write(&file_path, file_content).unwrap();
    let result = config.load();
    assert!(result.is_ok(), "Expected successful load, got {result:?}");

    // Test with an invalid IP address
    let file_content = r#"
        outside_ip = "invalid ip"
        cloudflare_ip = "invalid ip"
        last_updated = "2024-03-10T13:54:04.032435Z"
        save_dir = "/config"
        file_name = "cdu.toml"
    "#;
    fs::write(&file_path, file_content).unwrap();
    let result = config.load();
    assert!(
        result.is_err(),
        "Expected error when loading file with invalid IP, got {result:?}"
    );
}

#[test]
fn test_save() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join(CONFIG_FILE);
    let config = Config {
        save_dir: dir.path().to_path_buf(),
        file_name: String::from(CONFIG_FILE),
        ..Default::default()
    };

    // Test with a valid file
    let result = config.save();
    assert!(result.is_ok(), "Expected successful save, got {result:?}");

    // Test with a read-only file
    let file = fs::File::create(&file_path).unwrap();
    let metadata = file.metadata().unwrap();
    let mut permissions = metadata.permissions();
    permissions.set_readonly(true);
    fs::set_permissions(&file_path, permissions).unwrap();
    let result = config.save();
    assert!(
        result.is_err(),
        "Expected error when saving to read-only file, got {result:?}"
    );
}
