use mlua::{Lua, LuaSerdeExt as _};
use ratatui::{
    layout::Alignment,
    style::{Color, Style},
    symbols::border,
};
use serde::Deserialize;
use tmux_interface::Size;

use crate::{APP_NAME, args::Args, deserializers};

#[derive(Debug, Default)]
pub struct Config {
    _lua: Lua,
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
    #[serde(deserialize_with = "deserializers::alignment")]
    pub title_alignment: Alignment,
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
            title_alignment: Alignment::Center,
            title_style: Style::new(),
            item_style: Style::new(),
            item_match_style: Style::new().fg(Color::Blue),
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
    #[serde(deserialize_with = "deserializers::alignment")]
    pub title_alignment: Alignment,
    #[serde(deserialize_with = "deserializers::style")]
    pub title_style: Style,
    #[serde(deserialize_with = "deserializers::style")]
    pub pattern_style: Style,
    pub pattern_prefix: String,
    #[serde(deserialize_with = "deserializers::style")]
    pub pattern_prefix_style: Style,
    #[serde(skip)]
    pub stats_format: Option<mlua::Function>,
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
            title_alignment: Alignment::Center,
            title_style: Style::new(),
            pattern_style: Style::new(),
            pattern_prefix: String::from("> "),
            pattern_prefix_style: Style::new(),
            stats_format: None,
            stats_style: Style::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Keybinds {
    pub select_session: Vec<String>,
    pub last_session: Vec<String>,
}

impl Default for Keybinds {
    fn default() -> Self {
        Self {
            select_session: vec![String::from("C-s")],
            last_session: vec![String::from("w")],
        }
    }
}

impl Config {
    pub fn new(args: &Args) -> anyhow::Result<Self> {
        let path = args.config_file.clone().unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_default()
                .join(".config")
                .join(APP_NAME)
                .join("config.lua")
        });
        let code = match std::fs::read_to_string(path) {
            Ok(content) => content,
            Err(err) => {
                if err.kind() == std::io::ErrorKind::NotFound && args.config_file.is_none() {
                    return Ok(Self::default());
                }
                anyhow::bail!(err)
            }
        };

        let lua = Lua::new();
        let globals = lua.globals();
        let package = globals.get::<mlua::Table>("package")?;
        let loaded = package.get::<mlua::Table>("loaded")?;
        let module = match loaded.get(APP_NAME)? {
            mlua::Value::Table(module) => anyhow::Ok(module),
            mlua::Value::Nil => {
                let module = lua.create_table()?;
                loaded.set(APP_NAME, module.clone())?;
                anyhow::Ok(module)
            }
            other => anyhow::bail!(
                "failed to register '{APP_NAME}' module: 'package.loaded.{APP_NAME}' is already set to a value of type {}",
                other.type_name()
            ),
        }?;

        let deserialize_opts = mlua::DeserializeOptions::default().deny_unsupported_types(false);

        let mut session_selector_opt = None;
        let mut keybinds_opt = None;
        lua.scope(|scope| {
            let session_selector_fn = scope.create_function_mut(|lua, v: mlua::Value| {
                let mut session_selector =
                    lua.from_value_with::<SessionSelector>(v.clone(), deserialize_opts)?;
                session_selector.prompt.stats_format =
                    get_session_selector_prompt_stats_format(lua, &v)?;
                session_selector_opt = Some(session_selector);
                Ok(())
            })?;
            module.set("session_selector", session_selector_fn)?;

            let keybinds_fn = scope.create_function_mut(|lua, v: mlua::Value| {
                keybinds_opt = Some(lua.from_value_with(v, deserialize_opts)?);
                Ok(())
            })?;
            module.set("keybinds", keybinds_fn)?;

            lua.load(code).exec()
        })?;

        Ok(Self {
            _lua: lua,
            session_selector: session_selector_opt.unwrap_or_default(),
            keybinds: keybinds_opt.unwrap_or_default(),
        })
    }
}

fn get_session_selector_prompt_stats_format(
    lua: &Lua,
    v: &mlua::Value,
) -> mlua::Result<Option<mlua::Function>> {
    let Some(session_selector_table) = lua.convert::<Option<mlua::Table>>(v)? else {
        return Ok(None);
    };
    let Some(session_selector_prompt_table) =
        session_selector_table.get::<Option<mlua::Table>>("prompt")?
    else {
        return Ok(None);
    };
    session_selector_prompt_table.get("stats_format")
}
