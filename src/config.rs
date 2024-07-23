use std::ops::Deref;

use mlua::{Lua, LuaSerdeExt as _};
use ratatui::{style::Style, symbols::border};
use serde::Deserialize;
use tmux_interface::Size;

use crate::{
    consts::{CONFIG_FILE_PATH, PKG_NAME},
    deserializers,
};

#[derive(Debug, Default)]
pub struct Config {
    inner: Inner,
    pub on_session_created: Option<mlua::OwnedFunction>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Inner {
    pub session_selector: SessionSelector,
    pub keybinds: Keybinds,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct SessionSelector {
    #[serde(deserialize_with = "deserializers::size")]
    pub width: Size,
    #[serde(deserialize_with = "deserializers::size")]
    pub height: Size,
    pub scrolloff: usize,
    pub inverted: bool,
    pub paths: SessionSelectorPaths,
    pub results: SessionSelectorResults,
    pub prompt: SessionSelectorPrompt,
}

impl Default for SessionSelector {
    fn default() -> Self {
        Self {
            width: Size::Size(48),
            height: Size::Size(16),
            scrolloff: 4,
            inverted: false,
            paths: SessionSelectorPaths::default(),
            results: SessionSelectorResults::default(),
            prompt: SessionSelectorPrompt::default(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct SessionSelectorPaths {
    pub truncate_home_dir: bool,
    pub home_dir_symbol: String,
    pub trailing_slash: bool,
}

impl Default for SessionSelectorPaths {
    fn default() -> Self {
        Self {
            truncate_home_dir: true,
            home_dir_symbol: String::from("~"),
            trailing_slash: false,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
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

impl Default for SessionSelectorResults {
    fn default() -> Self {
        Self {
            style: Style::new(),
            border: border::ROUNDED,
            border_style: Style::new(),
            title: String::from(" Results "),
            title_style: Style::new(),
            item_style: Style::new(),
            item_match_style: Style::new(),
            selection_style: Style::new(),
            selection_prefix: String::from("> "),
            selection_prefix_style: Style::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
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
    pub stats_template: String,
    #[serde(deserialize_with = "deserializers::style")]
    pub stats_style: Style,
}

impl Default for SessionSelectorPrompt {
    fn default() -> Self {
        Self {
            style: Style::new(),
            border: border::ROUNDED,
            border_style: Style::new(),
            title: String::from(" Sessions "),
            title_style: Style::new(),
            pattern_style: Style::new(),
            pattern_prefix: String::from("> "),
            pattern_prefix_style: Style::new(),
            stats_template: String::from("{results}/{sessions}"),
            stats_style: Style::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Keybinds {
    pub session_selector: Vec<String>,
    pub last_session: Vec<String>,
}

impl Default for Keybinds {
    fn default() -> Self {
        Self {
            session_selector: vec![String::from("C-s"), String::from("M-s")],
            last_session: vec![String::from("w")],
        }
    }
}

impl Config {
    pub fn new() -> anyhow::Result<Self> {
        let code = match std::fs::read_to_string(CONFIG_FILE_PATH.as_path()) {
            Ok(content) => content,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(err) => anyhow::bail!(err),
        };

        let lua = Lua::new();
        let globals = lua.globals();
        let package = globals.get::<_, mlua::Table>("package")?;
        let loaded = package.get::<_, mlua::Table>("loaded")?;
        let module = match loaded.get(PKG_NAME)? {
                mlua::Value::Table(module) => anyhow::Ok(module),
                mlua::Value::Nil => {
                    let module = lua.create_table()?;
                    loaded.set(PKG_NAME, module.clone())?;
                    anyhow::Ok(module)
                }
                other => anyhow::bail!(
                    "failed to register '{PKG_NAME}' module: 'package.loaded.{PKG_NAME}' is already set to a value of type {}",
                    other.type_name()
                ),
            }?;

        let mut inner_opt = None;
        let mut on_session_created = None;
        lua.scope(|scope| {
            let config_fn = scope.create_function_mut(|lua, inner_val: mlua::Value| {
                inner_opt = Some(lua.from_value::<Inner>(inner_val)?);
                Ok(())
            })?;
            module.set("config", config_fn)?;
            let on_session_created_fn =
                scope.create_function_mut(|_, on_session_created_val: mlua::OwnedFunction| {
                    on_session_created = Some(on_session_created_val);
                    Ok(())
                })?;
            module.set("on_session_created", on_session_created_fn)?;
            lua.load(code).exec()
        })?;

        Ok(Self {
            inner: inner_opt.unwrap_or_default(),
            on_session_created,
        })
    }
}

impl Deref for Config {
    type Target = Inner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
