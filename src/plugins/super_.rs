//! super(1) log colorizer. Port of `mod_super.c`.

use crate::color::Color;
use crate::plugin::{HandleResult, Plugin, PluginType};
use crate::sink::OutputSink;
use once_cell::sync::Lazy;
use regex::Regex;
use std::io;

static RE_SUPER: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\S+)\s(\w+\s+\w+\s+\d+\s+\d+:\d+:\d+\s+\d+)(\s+)(\S+)\s\(([^\)]+)\)").unwrap()
});

pub struct Super;

impl Super {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for Super {
    fn name(&self) -> &'static str {
        "super"
    }
    fn ptype(&self) -> PluginType {
        PluginType::Full
    }
    fn description(&self) -> &'static str {
        "Coloriser for super(1) logs."
    }

    fn handle(&self, line: &str, sink: &mut dyn OutputSink) -> io::Result<HandleResult> {
        let caps = match RE_SUPER.captures(line) {
            Some(c) => c,
            None => return Ok(HandleResult::NoMatch),
        };
        let email = caps.get(1).unwrap().as_str();
        let date = caps.get(2).unwrap().as_str();
        let space = caps.get(3).unwrap().as_str();
        let suptag = caps.get(4).unwrap().as_str();
        let other = caps.get(5).unwrap().as_str();

        sink.emit(Color::Email, email)?;
        sink.space()?;
        sink.emit(Color::Date, date)?;
        sink.emit(Color::Default, space)?;
        sink.emit(Color::Proc, suptag)?;
        sink.space()?;
        sink.emit(Color::PidB, "(")?;
        sink.emit(Color::Default, other)?;
        sink.emit(Color::PidB, ")")?;
        sink.newline()?;
        Ok(HandleResult::Consumed)
    }
}
