use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    ffi::OsString,
    fs,
    io::{self, Write},
    path::PathBuf,
};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Config {
    pub(crate) hf_cache: PathBuf,
    pub(crate) lmstudio_models: PathBuf,
}

/// User config dir, not the binary dir: `cargo install` may put the binary
/// somewhere the user cannot write to.
pub(crate) fn config_path() -> PathBuf {
    if let Some(path) = env_value("HF2LMS_CONFIG") {
        return PathBuf::from(path);
    }

    #[cfg(windows)]
    if let Some(base) = env_value("APPDATA") {
        return PathBuf::from(base).join("hf2lms").join("config.json");
    }
    #[cfg(windows)]
    if let Some(base) = env_value("USERPROFILE") {
        return PathBuf::from(base).join("AppData/Roaming/hf2lms/config.json");
    }
    if let Some(base) = env_value("XDG_CONFIG_HOME") {
        return PathBuf::from(base).join("hf2lms").join("config.json");
    }
    if let Some(home) = env_value("HOME") {
        return PathBuf::from(home).join(".config/hf2lms/config.json");
    }

    if let Some(dir) = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(PathBuf::from))
    {
        return dir.join("hf2lms.json");
    }
    PathBuf::from("hf2lms.json")
}

fn env_value(key: &str) -> Option<OsString> {
    std::env::var_os(key).filter(|v| !v.is_empty())
}

fn prompt_line(prompt: &str) -> Result<String> {
    print!("{prompt}");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn prompt_path(prompt: &str) -> Result<PathBuf> {
    loop {
        let input = prompt_line(prompt)?;
        if input.is_empty() {
            println!("  (path is required)");
            continue;
        }
        let path = PathBuf::from(input);
        if !path.exists() {
            let ans = prompt_line(&format!(
                "  {} does not exist. Use anyway? [y/N] ",
                path.display()
            ))?;
            if !matches!(ans.to_ascii_lowercase().as_str(), "y" | "yes") {
                continue;
            }
        }
        return Ok(path);
    }
}

fn prompted(override_path: Option<PathBuf>, prompt: &str) -> Result<PathBuf> {
    match override_path {
        Some(path) => {
            println!("  {prompt}: {}", path.display());
            Ok(path)
        }
        None => prompt_path(&format!("{prompt}: ")),
    }
}

pub(crate) fn load_or_create_config(hf: Option<PathBuf>, lms: Option<PathBuf>) -> Result<Config> {
    let path = config_path();

    if path.exists() {
        let data =
            fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
        return serde_json::from_str(&data).with_context(|| format!("parsing {}", path.display()));
    }

    println!("First run. Configure paths:");
    let config = Config {
        hf_cache: prompted(hf, "HF cache directory (the 'hub' folder)")?,
        lmstudio_models: prompted(lms, "LM Studio models directory")?,
    };

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
    }
    fs::write(&path, serde_json::to_string_pretty(&config)?)
        .with_context(|| format!("writing {}", path.display()))?;
    println!("Config saved to {}", path.display());

    Ok(config)
}
