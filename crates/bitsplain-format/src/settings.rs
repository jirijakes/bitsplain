use std::fmt::{Display, Formatter, Result};

use serde::*;
use termion::color::Color;

use crate::ctx::Detail;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FormatType {
    Pretty,
    Json,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub details: Option<Detail>,
    pub format: Format,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Format {
    pub default: FormatType,
    pub pretty: PrettyFormat,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PrettyFormat {
    pub use_color: bool,
    pub doc: PrettyDocFormat,
    pub segment: PrettySegmentFormat,
    pub r#virtual: PrettyVirtualFormat,
    pub ids: PrettyIdsFormat,
    pub hex: PrettyHexFormat,
    pub tag: PrettyTagFormat,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PrettyDocFormat {
    pub show: bool,
    #[serde(flatten)]
    pub font: PrettyFont,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PrettySegmentFormat {
    pub show: bool,
    #[serde(flatten)]
    pub font: PrettyFont,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PrettyVirtualFormat {
    pub show: bool,
    #[serde(flatten)]
    pub font: PrettyFont,
    pub pre: Option<String>,
    pub post: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PrettyTagFormat {
    pub show: bool,
    #[serde(flatten)]
    pub font: PrettyFont,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PrettyHexFormat {
    pub show: bool,
    #[serde(flatten)]
    pub font: PrettyFont,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PrettyIdsFormat {
    pub show: bool,
    #[serde(flatten)]
    pub font: PrettyFont,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PrettyFont {
    pub foreground: Option<FgColor>,
    pub background: Option<BgColor>,
    pub style: Vec<Style>,
}

impl Display for PrettyFont {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.style
            .iter()
            .try_for_each(|s| f.write_str(s.as_termion()))?;
        self.foreground
            .iter()
            .try_for_each(|c| f.write_fmt(format_args!("{}", c)))?;
        self.background
            .iter()
            .try_for_each(|c| f.write_fmt(format_args!("{}", c)))?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FgColor {
    Black,
    Bitcoin,
    Blue,
    Cyan,
    Green,
    Magenta,
    Red,
    White,
    Yellow,
    LightBlack,
    LightBlue,
    LightCyan,
    LightGreen,
    LightMagenta,
    LightRed,
    LightWhite,
    LightYellow,
}

impl Display for FgColor {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            FgColor::Red => termion::color::Red.write_fg(f),
            FgColor::Black => termion::color::Black.write_fg(f),
            FgColor::Bitcoin => termion::color::Rgb(242, 169, 0).write_fg(f),
            FgColor::Blue => termion::color::Blue.write_fg(f),
            FgColor::Cyan => termion::color::Cyan.write_fg(f),
            FgColor::Green => termion::color::Green.write_fg(f),
            FgColor::Magenta => termion::color::Magenta.write_fg(f),
            FgColor::White => termion::color::White.write_fg(f),
            FgColor::Yellow => termion::color::Yellow.write_fg(f),
            FgColor::LightBlack => termion::color::LightBlack.write_fg(f),
            FgColor::LightBlue => termion::color::LightBlue.write_fg(f),
            FgColor::LightCyan => termion::color::LightCyan.write_fg(f),
            FgColor::LightGreen => termion::color::LightGreen.write_fg(f),
            FgColor::LightMagenta => termion::color::LightMagenta.write_fg(f),
            FgColor::LightRed => termion::color::LightRed.write_fg(f),
            FgColor::LightWhite => termion::color::LightWhite.write_fg(f),
            FgColor::LightYellow => termion::color::LightYellow.write_fg(f),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BgColor {
    Black,
    Bitcoin,
    Blue,
    Cyan,
    Green,
    Magenta,
    Red,
    White,
    Yellow,
    LightBlack,
    LightBlue,
    LightCyan,
    LightGreen,
    LightMagenta,
    LightRed,
    LightWhite,
    LightYellow,
}

impl Display for BgColor {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            BgColor::Red => termion::color::Red.write_bg(f),
            BgColor::Black => termion::color::Black.write_bg(f),
            BgColor::Bitcoin => termion::color::Rgb(242, 169, 0).write_bg(f),
            BgColor::Blue => termion::color::Blue.write_bg(f),
            BgColor::Cyan => termion::color::Cyan.write_bg(f),
            BgColor::Green => termion::color::Green.write_bg(f),
            BgColor::Magenta => termion::color::Magenta.write_bg(f),
            BgColor::White => termion::color::White.write_bg(f),
            BgColor::Yellow => termion::color::Yellow.write_bg(f),
            BgColor::LightBlack => termion::color::LightBlack.write_bg(f),
            BgColor::LightBlue => termion::color::LightBlue.write_bg(f),
            BgColor::LightCyan => termion::color::LightCyan.write_bg(f),
            BgColor::LightGreen => termion::color::LightGreen.write_bg(f),
            BgColor::LightMagenta => termion::color::LightMagenta.write_bg(f),
            BgColor::LightRed => termion::color::LightRed.write_bg(f),
            BgColor::LightWhite => termion::color::LightWhite.write_bg(f),
            BgColor::LightYellow => termion::color::LightYellow.write_bg(f),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Style {
    Blink,
    Bold,
    CrossedOut,
    Faint,
    Framed,
    Invert,
    Italic,
    Underline,
}

impl Style {
    pub fn as_termion(&self) -> &str {
        match self {
            Style::Blink => termion::style::Blink.as_ref(),
            Style::Bold => termion::style::Bold.as_ref(),
            Style::CrossedOut => termion::style::CrossedOut.as_ref(),
            Style::Faint => termion::style::Faint.as_ref(),
            Style::Framed => termion::style::Framed.as_ref(),
            Style::Invert => termion::style::Invert.as_ref(),
            Style::Italic => termion::style::Italic.as_ref(),
            Style::Underline => termion::style::Underline.as_ref(),
        }
    }
}
