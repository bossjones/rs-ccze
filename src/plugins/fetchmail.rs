//! fetchmail(1) sub-log colorizer. Port of `mod_fetchmail.c`. Partial type.

use crate::color::Color;
use crate::plugin::{HandleResult, Plugin, PluginType};
use crate::sink::OutputSink;
use once_cell::sync::Lazy;
use regex::Regex;
use std::io;

static RE_FETCHMAIL: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(reading message) ([^@]*@[^:]*):([0-9]*) of ([0-9]*) (.*)").unwrap());

pub struct Fetchmail;

impl Fetchmail {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for Fetchmail {
    fn name(&self) -> &'static str {
        "fetchmail"
    }
    fn ptype(&self) -> PluginType {
        PluginType::Partial
    }
    fn description(&self) -> &'static str {
        "Coloriser for fetchmail(1) sub-logs."
    }

    fn handle(&self, line: &str, sink: &mut dyn OutputSink) -> io::Result<HandleResult> {
        let caps = match RE_FETCHMAIL.captures(line) {
            Some(c) => c,
            None => return Ok(HandleResult::NoMatch),
        };
        let start = caps.get(1).unwrap().as_str();
        let addy = caps.get(2).unwrap().as_str();
        let current = caps.get(3).unwrap().as_str();
        let full = caps.get(4).unwrap().as_str();
        let rest = caps.get(5).unwrap().as_str();

        sink.emit(Color::Default, start)?;
        sink.space()?;
        sink.emit(Color::Email, addy)?;
        sink.emit(Color::Default, ":")?;
        sink.emit(Color::Numbers, current)?;
        sink.space()?;
        sink.emit(Color::Default, "of")?;
        sink.space()?;
        sink.emit(Color::Numbers, full)?;
        sink.space()?;
        Ok(HandleResult::Remainder(rest.to_owned()))
    }
}
