use std::env;
use std::str::FromStr;

use clap::builder::styling::{AnsiColor, Style};

const ENV_COLORS: &str = "GREP_COLORS";

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Capabilities {
    pub(super) match_text: Option<Style>,
    pub(super) filename: Option<Style>,
    pub(super) line_number: Option<Style>,
    pub(super) separator: Option<Style>,
}

impl Capabilities {
    pub(crate) fn from_env() -> Capabilities {
        if let Ok(str) = env::var(ENV_COLORS) {
            if let Ok(cs) = str.parse() {
                return cs;
            }
        }
        Capabilities::default()
    }

    pub(super) fn none() -> Capabilities {
        Capabilities {
            match_text: None,
            filename: None,
            line_number: None,
            separator: None,
        }
    }
}

impl Default for Capabilities {
    fn default() -> Self {
        Capabilities {
            match_text: Some(Style::new().bold().fg_color(Some(AnsiColor::Red.into()))),
            filename: Some(Style::new().fg_color(Some(AnsiColor::Magenta.into()))),
            line_number: Some(Style::new().fg_color(Some(AnsiColor::Green.into()))),
            separator: Some(Style::new().fg_color(Some(AnsiColor::Cyan.into()))),
        }
    }
}

impl FromStr for Capabilities {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut caps = Capabilities::default();
        let mut found_ms = false;
        for part in s.split(':') {
            if let Some(tail) = part.strip_prefix("mt=") {
                if !found_ms {
                    caps.match_text = parse_style(tail)
                }
            } else if let Some(tail) = part.strip_prefix("ms=") {
                found_ms = true;
                caps.match_text = parse_style(tail)
            } else if let Some(tail) = part.strip_prefix("fn=") {
                caps.filename = parse_style(tail)
            } else if let Some(tail) = part.strip_prefix("ln=") {
                caps.line_number = parse_style(tail)
            } else if let Some(tail) = part.strip_prefix("se=") {
                caps.separator = parse_style(tail)
            }
        }
        Ok(caps)
    }
}

fn parse_style(str: &str) -> Option<Style> {
    let mut result = Style::new();
    for part in str.split(';') {
        if let Ok(i) = part.parse::<u8>() {
            result = match i {
                1 => result.bold(),
                4 => result.underline(),
                5 => result.blink(),
                7 => result.invert(),
                30 => result.fg_color(Some(AnsiColor::Black.into())),
                31 => result.fg_color(Some(AnsiColor::Red.into())),
                32 => result.fg_color(Some(AnsiColor::Green.into())),
                33 => result.fg_color(Some(AnsiColor::Yellow.into())),
                34 => result.fg_color(Some(AnsiColor::Blue.into())),
                35 => result.fg_color(Some(AnsiColor::Magenta.into())),
                36 => result.fg_color(Some(AnsiColor::Cyan.into())),
                37 => result.fg_color(Some(AnsiColor::White.into())),
                39 => result.fg_color(None),
                40 => result.bg_color(Some(AnsiColor::Black.into())),
                41 => result.bg_color(Some(AnsiColor::Red.into())),
                42 => result.bg_color(Some(AnsiColor::Green.into())),
                43 => result.bg_color(Some(AnsiColor::Yellow.into())),
                44 => result.bg_color(Some(AnsiColor::Blue.into())),
                45 => result.bg_color(Some(AnsiColor::Magenta.into())),
                46 => result.bg_color(Some(AnsiColor::Cyan.into())),
                47 => result.bg_color(Some(AnsiColor::White.into())),
                49 => result.bg_color(None),
                90 => result.fg_color(Some(AnsiColor::BrightBlack.into())),
                91 => result.fg_color(Some(AnsiColor::BrightRed.into())),
                92 => result.fg_color(Some(AnsiColor::BrightGreen.into())),
                93 => result.fg_color(Some(AnsiColor::BrightYellow.into())),
                94 => result.fg_color(Some(AnsiColor::BrightBlue.into())),
                95 => result.fg_color(Some(AnsiColor::BrightMagenta.into())),
                96 => result.fg_color(Some(AnsiColor::BrightCyan.into())),
                97 => result.fg_color(Some(AnsiColor::BrightWhite.into())),
                100 => result.bg_color(Some(AnsiColor::BrightBlack.into())),
                101 => result.bg_color(Some(AnsiColor::BrightRed.into())),
                102 => result.bg_color(Some(AnsiColor::BrightGreen.into())),
                103 => result.bg_color(Some(AnsiColor::BrightYellow.into())),
                104 => result.bg_color(Some(AnsiColor::BrightBlue.into())),
                105 => result.bg_color(Some(AnsiColor::BrightMagenta.into())),
                106 => result.bg_color(Some(AnsiColor::BrightCyan.into())),
                107 => result.bg_color(Some(AnsiColor::BrightWhite.into())),
                _ => result,
            }
        }
    }
    if result == Style::new() {
        None
    } else {
        Some(result)
    }
}

#[cfg(test)]
mod test {
    use clap::builder::styling::AnsiColor::{Green, Magenta};

    use super::*;

    #[test]
    fn parse_empty() {
        assert_eq!(Ok(Capabilities::default()), "".parse())
    }

    #[test]
    fn parse_garbage() {
        assert_eq!(Ok(Capabilities::default()), "goober-whosit!".parse())
    }

    #[test]
    fn parse_mt() {
        assert_eq!(
            Some(Style::new().fg_color(Some(Magenta.into()))),
            "mt=35".parse::<Capabilities>().unwrap().match_text
        );
        assert_eq!(
            Some(Style::new().fg_color(Some(Magenta.into())).bold()),
            "mt=35;01".parse::<Capabilities>().unwrap().match_text
        );
        assert_eq!(
            Some(Style::new().bold().fg_color(Some(Magenta.into()))),
            "mt=01;35".parse::<Capabilities>().unwrap().match_text
        );
    }

    #[test]
    fn parse_ms() {
        assert_eq!(
            Some(Style::new().fg_color(Some(Magenta.into()))),
            "ms=35".parse::<Capabilities>().unwrap().match_text
        );
    }

    #[test]
    fn parse_ms_trumps_mt() {
        assert_eq!(
            Some(Style::new().fg_color(Some(Magenta.into()))),
            "mt=32:ms=35".parse::<Capabilities>().unwrap().match_text
        );
        assert_eq!(
            Some(Style::new().fg_color(Some(Magenta.into()))),
            "ms=35:mt=01;32".parse::<Capabilities>().unwrap().match_text
        );
    }

    #[test]
    fn parse_fn() {
        assert_eq!(
            Some(Style::new().fg_color(Some(Green.into()))),
            "fn=32".parse::<Capabilities>().unwrap().filename
        );
    }

    #[test]
    fn parse_ln() {
        assert_eq!(
            Some(Style::new().fg_color(Some(Green.into()))),
            "ln=32".parse::<Capabilities>().unwrap().line_number
        );
    }

    #[test]
    fn parse_se() {
        assert_eq!(
            Some(Style::new().fg_color(Some(Green.into()))),
            "se=32".parse::<Capabilities>().unwrap().separator
        );
    }
}
