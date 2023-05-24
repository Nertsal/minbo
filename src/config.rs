use std::{collections::HashMap, path::PathBuf};

use color_eyre::eyre::Context;
use serde::Deserialize;

#[derive(Default)]
pub struct Config {
    /// Path to the config directory.
    pub path: PathBuf,
    pub commands: SimpleCommands,
}

#[derive(Deserialize)]
pub struct SimpleCommands {
    #[serde(default)]
    pub cooldown: f64,
    pub commands: HashMap<String, String>,
}

impl Default for SimpleCommands {
    fn default() -> Self {
        Self {
            cooldown: 30.0,
            commands: Default::default(),
        }
    }
}

impl Config {
    /// Loads the config from the given folder.
    pub fn load(path: impl AsRef<std::path::Path>) -> color_eyre::Result<Self> {
        let path = path.as_ref().to_owned();

        let commands =
            read_or_default(path.join("commands.toml")).wrap_err("when loading client secret")?;

        Ok(Self { path, commands })
    }
}

/// Read from file and use default if file does not exist.
/// If file exists but cannot be read, then report the error.
fn read_or_default<T: serde::de::DeserializeOwned + Default>(
    path: impl AsRef<std::path::Path>,
) -> color_eyre::Result<T> {
    let path = path.as_ref();
    let content = match std::fs::read_to_string(path) {
        Ok(v) => v,
        Err(err) => match err.kind() {
            std::io::ErrorKind::NotFound => {
                log::info!("File at {:?} not found, using default", path);
                return Ok(T::default());
            }
            _ => return Err(err).wrap_err("when opening file"),
        },
    };

    // Parse normally and report errors
    let result = toml::from_str(&content).wrap_err("when parsing toml")?;
    Ok(result)
}
