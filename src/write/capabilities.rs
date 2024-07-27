use clap::builder::styling::{AnsiColor, Style};

const ENV_COLORS: &str = "GREP_COLORS";

pub(crate) struct Capabilities {
    pub(super) match_text: Option<Style>,
    pub(super) filename: Option<Style>,
    pub(super) line_number: Option<Style>,
    pub(super) separator: Option<Style>,
}

impl Capabilities {
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
