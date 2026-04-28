//! Command-line interface.
//!
//! Mirrors the C `ccze`'s argp-based options. Phase 0 wires the flags but only
//! `--debug` is honoured; the rest are accepted and ignored until later phases
//! need them.

use clap::{ArgAction, Parser, ValueEnum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Mode {
    Curses,
    Ansi,
    Html,
    Debug,
}

#[derive(Debug, Parser)]
#[command(
    name = "ccze",
    version,
    about = "A fast log colorizer (Rust port)",
    disable_version_flag = true,
    disable_help_flag = true
)]
pub struct Cli {
    /// Print help.
    #[arg(long = "help", action = ArgAction::Help)]
    pub help: Option<bool>,

    /// Read color config from FILE (use /dev/null to disable defaults).
    #[arg(short = 'F', long = "rcfile", value_name = "FILE")]
    pub rcfile: Option<String>,

    /// Load a plugin (comma-separated, or repeat the flag).
    #[arg(short = 'p', long = "plugin", value_delimiter = ',', action = ArgAction::Append)]
    pub plugins: Vec<String>,

    /// Toggle a sub-option (comma-separated, or repeat the flag).
    /// Known: nolookups, scroll, wordcolor, transparent, cssfile.
    #[arg(short = 'o', long = "options", value_delimiter = ',', action = ArgAction::Append)]
    pub options: Vec<String>,

    /// Pass arguments to a plugin: PLUGIN=ARGS
    #[arg(short = 'a', long = "argument", value_name = "PLUGIN=ARGS", action = ArgAction::Append)]
    pub plugin_args: Vec<String>,

    /// Override a color: KEY=COLOR
    #[arg(short = 'c', long = "color", value_name = "KEY=COLOR", action = ArgAction::Append)]
    pub color_overrides: Vec<String>,

    /// Output mode.
    #[arg(short = 'm', long = "mode", value_enum)]
    pub mode: Option<Mode>,

    /// Debug output (XML-like color tags).
    #[arg(short = 'd', long = "debug")]
    pub debug: bool,

    /// Raw ANSI escape output.
    #[arg(short = 'A', long = "raw-ansi")]
    pub raw_ansi: bool,

    /// HTML output.
    #[arg(short = 'h', long = "html")]
    pub html: bool,

    /// List available plugins.
    #[arg(short = 'l', long = "list-plugins")]
    pub list_plugins: bool,

    /// Strip syslog facility codes from line start.
    #[arg(short = 'r', long = "remove-facility")]
    pub remove_facility: bool,

    /// Convert UNIX timestamps in input to readable dates.
    #[arg(short = 'C', long = "convert-date")]
    pub convert_date: bool,

    /// Print version and exit.
    #[arg(short = 'V', long = "version", action = ArgAction::SetTrue)]
    pub version: bool,

    /// Dump the embedded CSS classes used in HTML mode and exit. Equivalent
    /// to the standalone `ccze-cssdump` binary in the C distribution.
    #[arg(long = "cssdump", action = ArgAction::SetTrue)]
    pub cssdump: bool,
}

impl Cli {
    pub fn resolved_mode(&self) -> Mode {
        if self.debug {
            Mode::Debug
        } else if self.html {
            Mode::Html
        } else if self.raw_ansi {
            Mode::Ansi
        } else if let Some(m) = self.mode {
            m
        } else {
            Mode::Curses
        }
    }
}
