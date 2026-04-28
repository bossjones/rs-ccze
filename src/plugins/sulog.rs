//! su(1) log colorizer. Port of `mod_sulog.c`.

use crate::color::Color;
use crate::plugin::{HandleResult, Plugin, PluginType};
use crate::sink::OutputSink;
use once_cell::sync::Lazy;
use regex::Regex;
use std::io;

static RE_SULOG: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^SU (\d{2}/\d{2} \d{2}:\d{2}) ([\+\-]) (\S+) ([^\-]+)-(.*)$").unwrap()
});

pub struct Sulog;

impl Sulog {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for Sulog {
    fn name(&self) -> &'static str {
        "sulog"
    }
    fn ptype(&self) -> PluginType {
        PluginType::Full
    }
    fn description(&self) -> &'static str {
        "Coloriser for su(1) logs."
    }

    fn handle(&self, line: &str, sink: &mut dyn OutputSink) -> io::Result<HandleResult> {
        let caps = match RE_SULOG.captures(line) {
            Some(c) => c,
            None => return Ok(HandleResult::NoMatch),
        };
        let date = caps.get(1).unwrap().as_str();
        let islogin = caps.get(2).unwrap().as_str();
        let tty = caps.get(3).unwrap().as_str();
        let fromuser = caps.get(4).unwrap().as_str();
        let touser = caps.get(5).unwrap().as_str();

        sink.emit(Color::Default, "SU ")?;
        sink.emit(Color::Date, date)?;
        sink.space()?;
        sink.emit(Color::Default, islogin)?;
        sink.space()?;
        let tty_color = if tty.starts_with('?') {
            Color::Unknown
        } else {
            Color::Dir
        };
        sink.emit(tty_color, tty)?;
        sink.space()?;
        sink.emit(Color::User, fromuser)?;
        sink.emit(Color::Default, "-")?;
        sink.emit(Color::User, touser)?;
        sink.newline()?;
        Ok(HandleResult::Consumed)
    }
}
