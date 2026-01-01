use anyhow::{Context, Result};
use gtk4_layer_shell::Edge;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::{debug, info, warn};

const DEFAULT_DISPLAY_TIMEOUT_MS: u64 = 2000;

const DEFAULT_BUBBLE_TIMEOUT_MS: u64 = 10000;

const DEFAULT_MAX_KEYS: usize = 5;

const DEFAULT_MARGIN: i32 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DisplayMode {
    #[default]
    Keystroke,
    Bubble,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Position {
    TopLeft,
    TopCenter,
    TopRight,
    BottomLeft,
    #[default]
    BottomCenter,
    BottomRight,
}

impl Position {
    pub fn layer_shell_edges(self) -> Vec<(Edge, bool)> {
        match self {
            Position::TopLeft => vec![
                (Edge::Top, true),
                (Edge::Left, true),
                (Edge::Bottom, false),
                (Edge::Right, false),
            ],
            Position::TopCenter => vec![
                (Edge::Top, true),
                (Edge::Left, false),
                (Edge::Bottom, false),
                (Edge::Right, false),
            ],
            Position::TopRight => vec![
                (Edge::Top, true),
                (Edge::Left, false),
                (Edge::Bottom, false),
                (Edge::Right, true),
            ],
            Position::BottomLeft => vec![
                (Edge::Top, false),
                (Edge::Left, true),
                (Edge::Bottom, true),
                (Edge::Right, false),
            ],
            Position::BottomCenter => vec![
                (Edge::Top, false),
                (Edge::Left, false),
                (Edge::Bottom, true),
                (Edge::Right, false),
            ],
            Position::BottomRight => vec![
                (Edge::Top, false),
                (Edge::Left, false),
                (Edge::Bottom, true),
                (Edge::Right, true),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub display_mode: DisplayMode,

    pub position: Position,

    pub display_timeout_ms: u64,

    pub bubble_timeout_ms: u64,

    pub max_keys: usize,

    pub margin: i32,

    pub show_modifiers: bool,

    pub all_keyboards: bool,

    pub font_scale: f64,

    pub opacity: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            display_mode: DisplayMode::Keystroke,
            position: Position::BottomCenter,
            display_timeout_ms: DEFAULT_DISPLAY_TIMEOUT_MS,
            bubble_timeout_ms: DEFAULT_BUBBLE_TIMEOUT_MS,
            max_keys: DEFAULT_MAX_KEYS,
            margin: DEFAULT_MARGIN,
            show_modifiers: true,
            all_keyboards: true,
            font_scale: 1.0,
            opacity: 0.9,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read config: {:?}", config_path))?;

            let config: Self =
                toml::from_str(&content).with_context(|| "Failed to parse config file")?;

            info!("Loaded configuration from {:?}", config_path);
            Ok(config)
        } else {
            debug!("No config file found, using defaults");
            Ok(Self::default())
        }
    }

    #[allow(dead_code)]
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config dir: {:?}", parent))?;
        }

        let content = toml::to_string_pretty(self).context("Failed to serialize config")?;

        fs::write(&config_path, content)
            .with_context(|| format!("Failed to write config: {:?}", config_path))?;

        info!("Saved configuration to {:?}", config_path);
        Ok(())
    }

    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir().context("Could not determine config directory")?;

        Ok(config_dir.join("keystroke").join("config.toml"))
    }

    #[allow(dead_code)]
    pub fn create_default_if_missing() -> Result<()> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            let default = Self::default();
            default.save()?;
            info!("Created default configuration at {:?}", config_path);
        }

        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        if self.display_timeout_ms < 100 {
            warn!(
                "display_timeout_ms is very low ({}ms)",
                self.display_timeout_ms
            );
        }

        if self.max_keys == 0 {
            anyhow::bail!("max_keys must be greater than 0");
        }

        if self.font_scale <= 0.0 {
            anyhow::bail!("font_scale must be positive");
        }

        if !(0.0..=1.0).contains(&self.opacity) {
            anyhow::bail!("opacity must be between 0.0 and 1.0");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.display_timeout_ms, DEFAULT_DISPLAY_TIMEOUT_MS);
        assert_eq!(config.max_keys, DEFAULT_MAX_KEYS);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_position_edges() {
        let pos = Position::BottomCenter;
        let edges = pos.layer_shell_edges();
        assert!(!edges.is_empty());
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let serialized = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();
        assert_eq!(config.display_timeout_ms, deserialized.display_timeout_ms);
    }
}
