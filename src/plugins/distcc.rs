//! distcc(1) log colorizer. Port of `mod_distcc.c`.

use crate::color::Color;
use crate::plugin::{HandleResult, Plugin, PluginType};
use crate::sink::OutputSink;
use once_cell::sync::Lazy;
use regex::Regex;
use std::io;

static RE_DISTCC: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^distccd\[(\d+)\] (\([^\)]+\))? ?(.*)").unwrap());

pub struct Distcc;

impl Distcc {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for Distcc {
    fn name(&self) -> &'static str {
        "distcc"
    }
    fn ptype(&self) -> PluginType {
        PluginType::Full
    }
    fn description(&self) -> &'static str {
        "Coloriser for distcc(1) logs."
    }

    fn handle(&self, line: &str, sink: &mut dyn OutputSink) -> io::Result<HandleResult> {
        let caps = match RE_DISTCC.captures(line) {
            Some(c) => c,
            None => return Ok(HandleResult::NoMatch),
        };
        let pid = caps.get(1).unwrap().as_str();
        let func = caps.get(2).map_or("", |m| m.as_str());
        let rest = caps.get(3).map_or("", |m| m.as_str());

        sink.emit(Color::Proc, "distccd")?;
        sink.emit(Color::PidB, "[")?;
        sink.emit(Color::Pid, pid)?;
        sink.emit(Color::PidB, "]")?;
        sink.space()?;
        if !func.is_empty() {
            sink.emit(Color::Keyword, func)?;
            sink.space()?;
        }
        Ok(HandleResult::Remainder(rest.to_owned()))
    }
}
