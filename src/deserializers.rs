use std::path::PathBuf;

use glob::Pattern;
use ratatui::{
    style::{Color, Modifier, Style},
    symbols::border,
};
use serde::{de::Visitor, Deserialize, Deserializer};
use tmux_interface::Size;

use crate::util::PathDisplayPretty;

pub fn size<'de, D>(deserializer: D) -> Result<Size, D::Error>
where
    D: Deserializer<'de>,
{
    struct SizeVisitor;

    impl<'de> Visitor<'de> for SizeVisitor {
        type Value = Size;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("an unsigned integer or a string")
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            #[allow(clippy::cast_possible_truncation)]
            Ok(Size::Size(v as usize))
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            v.strip_suffix('%')
                .ok_or(E::invalid_value(
                    serde::de::Unexpected::Char(v.chars().last().unwrap_or_default()),
                    &"'%' at the end",
                ))?
                .parse::<usize>()
                .map(Size::Percentage)
                .map_err(E::custom)
        }
    }

    deserializer.deserialize_any(SizeVisitor)
}

pub fn border_set<'de, D>(deserializer: D) -> Result<border::Set, D::Error>
where
    D: Deserializer<'de>,
{
    struct BorderSetVisitor;

    impl<'de> Visitor<'de> for BorderSetVisitor {
        type Value = border::Set;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("a string or an array of characters")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            let border_set = match v {
                "plain" => border::PLAIN,
                "rounded" => border::ROUNDED,
                "double" => border::DOUBLE,
                "thick" => border::THICK,
                "quadrant-inside" => border::QUADRANT_INSIDE,
                "quadrant-outside" => border::QUADRANT_OUTSIDE,
                "none" => border::Set {
                    top_left: " ",
                    top_right: " ",
                    bottom_left: " ",
                    bottom_right: " ",
                    vertical_left: " ",
                    vertical_right: " ",
                    horizontal_top: " ",
                    horizontal_bottom: " ",
                },
                v => {
                    return Err(E::unknown_variant(
                        v,
                        &[
                            "plain",
                            "rounded",
                            "double",
                            "thick",
                            "quadrant-inside",
                            "quadrant-outside",
                            "none",
                        ],
                    ))
                }
            };
            Ok(border_set)
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let mut get_next_border_str = |i: usize| -> Result<&'static str, A::Error> {
                let border_string = seq
                    .next_element::<String>()?
                    .ok_or_else(|| serde::de::Error::invalid_length(i, &"length 8"))?;
                Ok(Box::leak(border_string.into_boxed_str()))
            };
            let border_set = border::Set {
                top_left: get_next_border_str(0)?,
                top_right: get_next_border_str(1)?,
                bottom_left: get_next_border_str(2)?,
                bottom_right: get_next_border_str(3)?,
                vertical_left: get_next_border_str(4)?,
                vertical_right: get_next_border_str(5)?,
                horizontal_top: get_next_border_str(6)?,
                horizontal_bottom: get_next_border_str(7)?,
            };
            Ok(border_set)
        }
    }

    deserializer.deserialize_any(BorderSetVisitor)
}

pub fn style<'de, D>(deserializer: D) -> Result<Style, D::Error>
where
    D: Deserializer<'de>,
{
    let mut style = Style::new();
    let Some(style_string) = Option::<String>::deserialize(deserializer)? else {
        return Ok(style);
    };
    for token in style_string.split(' ') {
        if let Some(color_fg) = token.strip_prefix("fg:") {
            let color = color_fg
                .parse::<Color>()
                .map_err(serde::de::Error::custom)?;
            style = style.fg(color);
            continue;
        }
        if let Some(color_bg) = token.strip_prefix("bg:") {
            let color = color_bg
                .parse::<Color>()
                .map_err(serde::de::Error::custom)?;
            style = style.bg(color);
            continue;
        }
        let modifier = match token {
            "bold" => Modifier::BOLD,
            "dim" => Modifier::DIM,
            "italic" => Modifier::ITALIC,
            "underlined" => Modifier::UNDERLINED,
            "slowblink" | "slow_blink" | "slow-blink" => Modifier::SLOW_BLINK,
            "rapidblink" | "rapid_blink" | "rapid-blink" => Modifier::RAPID_BLINK,
            "reversed" => Modifier::REVERSED,
            "hidden" => Modifier::HIDDEN,
            "crossedout" | "crossed_out" | "crossed-out" => Modifier::CROSSED_OUT,
            token => {
                return Err(serde::de::Error::unknown_variant(
                    token,
                    &[
                        "fg:<color>",
                        "bg:<color>",
                        "bold",
                        "dim",
                        "italic",
                        "underlined",
                        "slow-blink",
                        "rapid-blink",
                        "reversed",
                        "hidden",
                        "crossed-out",
                    ],
                ))
            }
        };
        style = style.add_modifier(modifier);
    }
    Ok(style)
}

pub fn pattern_vec<'de, D>(deserializer: D) -> Result<Vec<Pattern>, D::Error>
where
    D: Deserializer<'de>,
{
    Vec::<String>::deserialize(deserializer)?
        .iter()
        .map(|pattern_string| Pattern::new(pattern_string))
        .collect::<Result<Vec<_>, _>>()
        .map_err(serde::de::Error::custom)
}

pub fn path_pretty_opt<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<PathBuf>::deserialize(deserializer)
        .map(|path_opt| path_opt.map(|path| path.display_pretty().to_string()))
}
