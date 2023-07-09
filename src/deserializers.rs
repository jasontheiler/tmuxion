use ratatui::{
    style::{Color, Modifier, Style},
    symbols::border,
};
use serde::{de::Visitor, Deserialize, Deserializer};
use tmux_interface::Size;

pub fn size<'de, D>(deserializer: D) -> Result<Size, D::Error>
where
    D: Deserializer<'de>,
{
    struct SizeVisitor;

    impl<'de> Visitor<'de> for SizeVisitor {
        type Value = Size;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("an unsigned integer or a percentage string")
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            usize::try_from(v).map(Size::Size).map_err(E::custom)
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

const BORDER_SET_STRING_VALUES: &[&str] = &[
    "plain",
    "rounded",
    "double",
    "thick",
    "quadrant_inside",
    "quadrant_outside",
    "none",
];

pub fn border_set<'de, D>(deserializer: D) -> Result<border::Set, D::Error>
where
    D: Deserializer<'de>,
{
    struct BorderSetVisitor;

    impl<'de> Visitor<'de> for BorderSetVisitor {
        type Value = border::Set;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str(&format!(
                "{} or a sequence of 8 individual characters",
                BORDER_SET_STRING_VALUES
                    .iter()
                    .map(|val| format!("`{val}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
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
                "quadrantinside" | "quadrant_inside" | "quadrant-inside" => border::QUADRANT_INSIDE,
                "quadrantoutside" | "quadrant_outside" | "quadrant-outside" => {
                    border::QUADRANT_OUTSIDE
                }
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
                v => return Err(E::unknown_variant(v, BORDER_SET_STRING_VALUES)),
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

const STYLE_MODIFIER_STRING_VALUES: &[&str] = &[
    "bold",
    "dim",
    "italic",
    "underlined",
    "slow_blink",
    "rapid_blink",
    "reversed",
    "hidden",
    "crossed_out",
];

#[derive(Debug, Deserialize)]
struct StyleIntermediate {
    fg: Option<String>,
    bg: Option<String>,
    #[serde(default)]
    modifiers: Vec<String>,
}

pub fn style<'de, D>(deserializer: D) -> Result<Style, D::Error>
where
    D: Deserializer<'de>,
{
    let mut style = Style::new();
    let Some(style_intermediate) = Option::<StyleIntermediate>::deserialize(deserializer)? else {
        return Ok(style);
    };
    if let Some(color_str) = style_intermediate.fg {
        let color = color_str
            .parse::<Color>()
            .map_err(serde::de::Error::custom)?;
        style = style.fg(color);
    }
    if let Some(color_str) = style_intermediate.bg {
        let color = color_str
            .parse::<Color>()
            .map_err(serde::de::Error::custom)?;
        style = style.bg(color);
    }
    for modifier_str in style_intermediate.modifiers {
        let modifier = match modifier_str.as_str() {
            "bold" => Modifier::BOLD,
            "dim" => Modifier::DIM,
            "italic" => Modifier::ITALIC,
            "underlined" => Modifier::UNDERLINED,
            "slowblink" | "slow_blink" | "slow-blink" => Modifier::SLOW_BLINK,
            "rapidblink" | "rapid_blink" | "rapid-blink" => Modifier::RAPID_BLINK,
            "reversed" => Modifier::REVERSED,
            "hidden" => Modifier::HIDDEN,
            "crossedout" | "crossed_out" | "crossed-out" => Modifier::CROSSED_OUT,
            modifier_str => {
                return Err(serde::de::Error::unknown_variant(
                    modifier_str,
                    STYLE_MODIFIER_STRING_VALUES,
                ))
            }
        };
        style = style.add_modifier(modifier);
    }
    Ok(style)
}
