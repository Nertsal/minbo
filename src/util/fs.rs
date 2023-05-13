use color_eyre::eyre::Context;

pub fn read_to_string(path: impl AsRef<std::path::Path>) -> color_eyre::Result<String> {
    let path = path.as_ref();
    let s =
        std::fs::read_to_string(path).wrap_err_with(|| format!("Failed to read from {path:?}"))?;
    Ok(s)
}

/// Reads from a file and attempts to parse its contents from toml format.
pub fn read_toml<T: serde::de::DeserializeOwned>(
    path: impl AsRef<std::path::Path>,
) -> color_eyre::Result<T> {
    let content = read_to_string(path)?;
    let result = toml::from_str(&content)?;
    Ok(result)
}
