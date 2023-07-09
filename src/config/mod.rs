use std::path::PathBuf;

use figment::{
    providers::{Format, Yaml},
    Figment,
};
use once_cell::sync::Lazy;
use ratatui::{style::Style, symbols::border};
use serde::Deserialize;
use tmux_interface::Size;

use crate::{consts::CONFIG_DIR_PATH, deserializers, util::PathDisplayPretty};

pub const CONFIG_DEFAULT: &str = include_str!("./config.default.yaml");
static CONFIG_FILE_PATH: Lazy<PathBuf> = Lazy::new(|| CONFIG_DIR_PATH.join("config.yaml"));

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub session_selector: SessionSelector,
    pub keybinds: Keybinds,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SessionSelector {
    #[serde(deserialize_with = "deserializers::size")]
    pub width: Size,
    #[serde(deserialize_with = "deserializers::size")]
    pub height: Size,
    pub scrolloff: usize,
    pub inverted: bool,
    pub results: SessionSelectorResults,
    pub prompt: SessionSelectorPrompt,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SessionSelectorResults {
    #[serde(deserialize_with = "deserializers::style")]
    pub style: Style,
    #[serde(deserialize_with = "deserializers::border_set")]
    pub border: border::Set,
    #[serde(deserialize_with = "deserializers::style")]
    pub border_style: Style,
    pub title: String,
    #[serde(deserialize_with = "deserializers::style")]
    pub title_style: Style,
    #[serde(deserialize_with = "deserializers::style")]
    pub item_style: Style,
    #[serde(deserialize_with = "deserializers::style")]
    pub item_match_style: Style,
    #[serde(deserialize_with = "deserializers::style")]
    pub selection_style: Style,
    pub selection_prefix: String,
    #[serde(deserialize_with = "deserializers::style")]
    pub selection_prefix_style: Style,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SessionSelectorPrompt {
    #[serde(deserialize_with = "deserializers::style")]
    pub style: Style,
    #[serde(deserialize_with = "deserializers::border_set")]
    pub border: border::Set,
    #[serde(deserialize_with = "deserializers::style")]
    pub border_style: Style,
    pub title: String,
    #[serde(deserialize_with = "deserializers::style")]
    pub title_style: Style,
    #[serde(deserialize_with = "deserializers::style")]
    pub pattern_style: Style,
    pub pattern_prefix: String,
    #[serde(deserialize_with = "deserializers::style")]
    pub pattern_prefix_style: Style,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Keybinds {
    pub session_selector: Vec<String>,
    pub last_session: Vec<String>,
}

impl Config {
    pub fn new() -> anyhow::Result<Self> {
        Figment::from(Yaml::string(CONFIG_DEFAULT))
            .merge(Yaml::file(CONFIG_FILE_PATH.as_path()))
            .extract()
            .map_err(|err| anyhow::format_err!("failed to load config: {err}"))
    }

    pub fn copy_default() -> anyhow::Result<()> {
        match std::fs::metadata(CONFIG_FILE_PATH.as_path()) {
            Ok(_) => {
                return Err(anyhow::format_err!(
                    "config already exists in {}",
                    CONFIG_FILE_PATH.display_pretty()
                ))
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => (),
            Err(err) => return Err(err.into()),
        }

        if let Some(parent) = CONFIG_FILE_PATH.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(CONFIG_FILE_PATH.as_path(), CONFIG_DEFAULT)?;
        println!("copied config to {}", CONFIG_FILE_PATH.display_pretty());

        Ok(())
    }
}
