use std::{
    env, fs, io,
    path::{Path, PathBuf},
    time::SystemTime,
};

use iced::{
    Color, Font, Size, Theme,
    font::{Family, Weight},
    theme::Palette,
    window,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct Config {
    pub appearance: Appearance,
    pub window: WindowConfig,
    pub behavior: Behavior,
    pub text: Text,
    pub typography: Typography,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Appearance {
    pub theme: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub danger: Option<String>,
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            theme: "dark".to_owned(),
            background: None,
            text: None,
            primary: None,
            success: None,
            warning: None,
            danger: None,
        }
    }
}

impl Appearance {
    pub fn theme(&self) -> Theme {
        let base = builtin_theme(&self.theme);
        let palette = base.palette();
        let has_custom_colors = self.background.is_some()
            || self.text.is_some()
            || self.primary.is_some()
            || self.success.is_some()
            || self.warning.is_some()
            || self.danger.is_some();

        if !has_custom_colors {
            return base;
        }

        Theme::custom(
            "iguard",
            Palette {
                background: parse_color(self.background.as_deref()).unwrap_or(palette.background),
                text: parse_color(self.text.as_deref()).unwrap_or(palette.text),
                primary: parse_color(self.primary.as_deref()).unwrap_or(palette.primary),
                success: parse_color(self.success.as_deref()).unwrap_or(palette.success),
                warning: parse_color(self.warning.as_deref()).unwrap_or(palette.warning),
                danger: parse_color(self.danger.as_deref()).unwrap_or(palette.danger),
            },
        )
    }
}

fn builtin_theme(name: &str) -> Theme {
    match name.trim().to_ascii_lowercase().as_str() {
        "light" => Theme::Light,
        "dracula" => Theme::Dracula,
        "nord" => Theme::Nord,
        "solarized-light" => Theme::SolarizedLight,
        "solarized-dark" => Theme::SolarizedDark,
        "gruvbox-light" => Theme::GruvboxLight,
        "gruvbox-dark" => Theme::GruvboxDark,
        "catppuccin-latte" => Theme::CatppuccinLatte,
        "catppuccin-frappe" => Theme::CatppuccinFrappe,
        "catppuccin-macchiato" => Theme::CatppuccinMacchiato,
        "catppuccin-mocha" => Theme::CatppuccinMocha,
        "tokyo-night" => Theme::TokyoNight,
        "tokyo-night-storm" => Theme::TokyoNightStorm,
        "tokyo-night-light" => Theme::TokyoNightLight,
        "kanagawa-wave" => Theme::KanagawaWave,
        "kanagawa-dragon" => Theme::KanagawaDragon,
        "kanagawa-lotus" => Theme::KanagawaLotus,
        "moonfly" => Theme::Moonfly,
        "nightfly" => Theme::Nightfly,
        "oxocarbon" => Theme::Oxocarbon,
        "ferra" => Theme::Ferra,
        _ => Theme::Dark,
    }
}

fn parse_color(value: Option<&str>) -> Option<Color> {
    let value = value?.trim();
    let value = value.strip_prefix('#').unwrap_or(value);

    if !value.is_ascii() {
        return None;
    }

    match value.len() {
        3 => {
            let r = u8::from_str_radix(&value[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&value[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&value[2..3], 16).ok()? * 17;
            Some(Color::from_rgba8(r, g, b, 1.0))
        }
        6 => {
            let r = u8::from_str_radix(&value[0..2], 16).ok()?;
            let g = u8::from_str_radix(&value[2..4], 16).ok()?;
            let b = u8::from_str_radix(&value[4..6], 16).ok()?;
            Some(Color::from_rgba8(r, g, b, 1.0))
        }
        8 => {
            let r = u8::from_str_radix(&value[0..2], 16).ok()?;
            let g = u8::from_str_radix(&value[2..4], 16).ok()?;
            let b = u8::from_str_radix(&value[4..6], 16).ok()?;
            let a = u8::from_str_radix(&value[6..8], 16).ok()?;
            Some(Color::from_rgba8(r, g, b, f32::from(a) / 255.0))
        }
        _ => None,
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(default)]
pub struct WindowConfig {
    pub width: f32,
    pub height: f32,
    pub placement: Placement,
    pub x: f32,
    pub y: f32,
    pub resizable: bool,
    pub decorations: bool,
    pub always_on_top: bool,
    pub closeable: bool,
    pub minimizable: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: 520.0,
            height: 380.0,
            placement: Placement::Center,
            x: 0.0,
            y: 0.0,
            resizable: false,
            decorations: true,
            always_on_top: false,
            closeable: true,
            minimizable: false,
        }
    }
}

impl WindowConfig {
    pub fn size(&self) -> Size {
        Size::new(
            self.width.clamp(320.0, 4096.0),
            self.height.clamp(200.0, 4096.0),
        )
    }

    pub fn position(&self) -> window::Position {
        match self.placement {
            Placement::Default => window::Position::Default,
            Placement::Center => window::Position::Centered,
            Placement::Fixed => window::Position::Specific(iced::Point::new(self.x, self.y)),
        }
    }

    pub fn settings(&self) -> window::Settings {
        window::Settings {
            size: self.size(),
            position: self.position(),
            resizable: self.resizable,
            decorations: self.decorations,
            closeable: self.closeable,
            minimizable: self.minimizable,
            level: if self.always_on_top {
                window::Level::AlwaysOnTop
            } else {
                window::Level::Normal
            },
            exit_on_close_request: false,
            ..window::Settings::default()
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Placement {
    Default,
    #[default]
    Center,
    Fixed,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Behavior {
    pub autofocus: bool,
    pub close_on_escape: bool,
    pub submit_on_enter: bool,
    pub max_attempts: usize,
    pub show_action_id: bool,
    pub show_details: bool,
}

impl Default for Behavior {
    fn default() -> Self {
        Self {
            autofocus: true,
            close_on_escape: true,
            submit_on_enter: true,
            max_attempts: 3,
            show_action_id: false,
            show_details: false,
        }
    }
}

impl Behavior {
    pub fn max_attempts(&self) -> usize {
        self.max_attempts.clamp(1, 10)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Text {
    pub window_title: String,
    pub title: String,
    pub identity: String,
    pub waiting: String,
    pub authenticating: String,
    pub password: String,
    pub submit: String,
    pub cancel: String,
    pub failed: String,
    pub cancelled: String,
    pub retry: String,
    pub action: String,
    pub details: String,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            window_title: "iguard".to_owned(),
            title: "Authentication required".to_owned(),
            identity: "Identity".to_owned(),
            waiting: "Waiting for authentication...".to_owned(),
            authenticating: "Authenticating...".to_owned(),
            password: "Password".to_owned(),
            submit: "Authenticate".to_owned(),
            cancel: "Cancel".to_owned(),
            failed: "Authentication failed".to_owned(),
            cancelled: "Authentication cancelled".to_owned(),
            retry: "Authentication failed. Try again.".to_owned(),
            action: "Action".to_owned(),
            details: "Details".to_owned(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Typography {
    pub font: FontFamily,
    pub weight: FontWeight,
    pub title_size: f32,
    pub body_size: f32,
    pub small_size: f32,
    pub input_size: f32,
    pub spacing: f32,
    pub padding: f32,
    pub input_padding: f32,
    pub button_padding_x: f32,
    pub button_padding_y: f32,
}

impl Default for Typography {
    fn default() -> Self {
        Self {
            font: FontFamily::SansSerif,
            weight: FontWeight::Normal,
            title_size: 22.0,
            body_size: 14.0,
            small_size: 13.0,
            input_size: 16.0,
            spacing: 10.0,
            padding: 24.0,
            input_padding: 10.0,
            button_padding_x: 16.0,
            button_padding_y: 8.0,
        }
    }
}

impl Typography {
    pub fn font(&self) -> Font {
        Font {
            family: self.font.into(),
            weight: self.weight.into(),
            ..Font::DEFAULT
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum FontFamily {
    #[default]
    SansSerif,
    Serif,
    Monospace,
    Cursive,
    Fantasy,
}

impl From<FontFamily> for Family {
    fn from(family: FontFamily) -> Self {
        match family {
            FontFamily::SansSerif => Self::SansSerif,
            FontFamily::Serif => Self::Serif,
            FontFamily::Monospace => Self::Monospace,
            FontFamily::Cursive => Self::Cursive,
            FontFamily::Fantasy => Self::Fantasy,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum FontWeight {
    Thin,
    ExtraLight,
    Light,
    #[default]
    Normal,
    Medium,
    Semibold,
    Bold,
    ExtraBold,
    Black,
}

impl From<FontWeight> for Weight {
    fn from(weight: FontWeight) -> Self {
        match weight {
            FontWeight::Thin => Self::Thin,
            FontWeight::ExtraLight => Self::ExtraLight,
            FontWeight::Light => Self::Light,
            FontWeight::Normal => Self::Normal,
            FontWeight::Medium => Self::Medium,
            FontWeight::Semibold => Self::Semibold,
            FontWeight::Bold => Self::Bold,
            FontWeight::ExtraBold => Self::ExtraBold,
            FontWeight::Black => Self::Black,
        }
    }
}

pub fn path() -> PathBuf {
    let base = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .unwrap_or_else(|| PathBuf::from("."));

    base.join("iguard").join("config.toml")
}

pub fn load() -> (Config, Option<SystemTime>) {
    let path = path();

    let config = if path.exists() {
        load_from(&path).unwrap_or_default()
    } else {
        let config = Config::default();

        if let Err(error) = write_default(&path, &config) {
            eprintln!(
                "iguard: could not create the configuration at {}: {error}",
                path.display()
            );
        }

        config
    };

    (config, modified(&path))
}

pub fn load_from(path: &Path) -> Option<Config> {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) => {
            if error.kind() != io::ErrorKind::NotFound {
                eprintln!(
                    "iguard: could not read the configuration at {}: {error}",
                    path.display()
                );
            }

            return None;
        }
    };

    match toml::from_str(&contents) {
        Ok(config) => Some(config),
        Err(error) => {
            eprintln!(
                "iguard: invalid configuration at {}: {error}",
                path.display()
            );
            None
        }
    }
}

pub fn modified(path: &Path) -> Option<SystemTime> {
    fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
}

fn write_default(path: &Path, config: &Config) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let body = toml::to_string_pretty(config).map_err(io::Error::other)?;
    let contents = format!(
        "# iguard configuration\n\
         # Los cambios se recargan automáticamente al guardar.\n\
         #\n\
         # appearance.theme: dark, light, dracula, nord, solarized-light,\n\
         #   solarized-dark, gruvbox-light, gruvbox-dark, catppuccin-*,\n\
         #   tokyo-night, tokyo-night-storm, tokyo-night-light, kanagawa-*,\n\
         #   moonfly, nightfly, oxocarbon, ferra\n\
         # appearance colors: #RRGGBB o #RRGGBBAA\n\
         # window.placement: default, center, fixed\n\
         # typography.font: sans-serif, serif, monospace, cursive, fantasy\n\
         # typography.weight: thin, extra-light, light, normal, medium,\n\
         #   semibold, bold, extra-bold, black\n\
         \n{body}"
    );

    fs::write(path, contents)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_custom_values() {
        let config: Config = toml::from_str(
            r##"
            [appearance]
            theme = "nord"
            primary = "#ff0000"

            [window]
            width = 640
            height = 400
            placement = "fixed"
            x = 10
            y = 20

            [behavior]
            max_attempts = 5
            show_details = true

            [text]
            title = "Hola"

            [typography]
            font = "monospace"
            weight = "bold"
            "##,
        )
        .expect("configuration should parse");

        assert_eq!(config.window.width, 640.0);
        assert_eq!(config.window.height, 400.0);
        assert_eq!(config.window.x, 10.0);
        assert_eq!(config.window.y, 20.0);
        assert!(matches!(config.window.placement, Placement::Fixed));
        assert_eq!(config.behavior.max_attempts(), 5);
        assert!(config.behavior.show_details);
        assert_eq!(config.text.title, "Hola");
        assert!(matches!(config.typography.font, FontFamily::Monospace));
        assert!(matches!(config.typography.weight, FontWeight::Bold));
    }

    #[test]
    fn parses_hex_colors() {
        assert_eq!(
            parse_color(Some("#0f0")),
            Some(Color::from_rgba8(0, 255, 0, 1.0))
        );
        assert_eq!(
            parse_color(Some("336699")),
            Some(Color::from_rgba8(0x33, 0x66, 0x99, 1.0))
        );
        assert_eq!(
            parse_color(Some("#33669980")),
            Some(Color::from_rgba8(0x33, 0x66, 0x99, 128.0 / 255.0))
        );
        assert_eq!(parse_color(Some("nope")), None);
    }

    #[test]
    fn unknown_theme_falls_back_to_dark() {
        let appearance = Appearance {
            theme: "unknown".to_owned(),
            ..Appearance::default()
        };

        assert_eq!(appearance.theme(), Theme::Dark);
    }
}
