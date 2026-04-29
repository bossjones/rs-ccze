//! Procmail(1) log colorizer. Port of `mod_procmail.c`.

use crate::color::Color;
use crate::plugin::{HandleResult, Plugin, PluginType};
use crate::sink::OutputSink;
use once_cell::sync::Lazy;
use regex::Regex;
use std::io;

static RE_PROCMAIL: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(\s*)(>?From|Subject:|Folder:)?\s(\S+)(\s+)?(.*)").unwrap());

pub struct Procmail;

impl Procmail {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for Procmail {
    fn name(&self) -> &'static str {
        "procmail"
    }
    fn ptype(&self) -> PluginType {
        PluginType::Full
    }
    fn description(&self) -> &'static str {
        "Coloriser for procmail(1) logs."
    }

    fn handle(&self, line: &str, sink: &mut dyn OutputSink) -> io::Result<HandleResult> {
        let caps = match RE_PROCMAIL.captures(line) {
            Some(c) => c,
            None => return Ok(HandleResult::NoMatch),
        };

        // The C code uses `pcre_get_substring`, which returns "" for unmatched
        // optional groups. Mirror that with map_or.
        let space1 = caps.get(1).map_or("", |m| m.as_str());
        let header = caps.get(2).map_or("", |m| m.as_str());
        let value = caps.get(3).map_or("", |m| m.as_str());
        let space2 = caps.get(4).map_or("", |m| m.as_str());
        let extra = caps.get(5).map_or("", |m| m.as_str());

        // Header gating — only the recognised three are decorated. Anything
        // else: claim the line but pass it back as-is (the C source returns
        // strdup(str) in this case) so the dispatcher word-colours the whole
        // line.
        let h = header.to_ascii_lowercase();
        let value_color = match h.as_str() {
            "from" | ">from" => Some(Color::Email),
            "subject:" => Some(Color::Subject),
            "folder:" => Some(Color::Dir),
            _ => None,
        };

        let value_color = match value_color {
            Some(c) => c,
            None => return Ok(HandleResult::Remainder(line.to_owned())),
        };

        // C `ccze_addstr(DEFAULT, str)` skips emit only when str is NULL.
        // Rust never has a null `&str`, so always emit — this preserves the
        // empty `<default></default>` and `<subject></subject>` tags seen in
        // the snapshot fixtures.
        sink.emit(Color::Default, space1)?;
        sink.emit(Color::Default, header)?;
        sink.space()?;

        sink.emit(value_color, value)?;

        // After Email: col resets to Default for the trailing space + extra
        // is coloured Date for "from", Size for "folder:". For Subject, col
        // stays Subject through both space2 and extra.
        let mut col_after = value_color;
        if value_color == Color::Email {
            col_after = Color::Default;
        }
        sink.emit(col_after, space2)?;

        let extra_col = match h.as_str() {
            "folder:" => Color::Size,
            "from" | ">from" => Color::Date,
            _ => col_after,
        };
        sink.emit(extra_col, extra)?;

        sink.newline()?;
        Ok(HandleResult::Consumed)
    }
}
