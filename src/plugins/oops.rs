//! oops proxy log colorizer. Port of `mod_oops.c`.

use crate::color::Color;
use crate::plugin::{HandleResult, Plugin, PluginType};
use crate::sink::OutputSink;
use once_cell::sync::Lazy;
use regex::Regex;
use std::io;

static RE_OOPS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"^((Mon|Tue|Wed|Thu|Fri|Sat|Sun) (Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec) \d+ \d+:\d+:\d+ \d+)(\s+)\[([\dxa-fA-F]+)\]statistics\(\): ([\S]+)(\s*): (\d+)(.*)",
    )
    .unwrap()
});

pub struct Oops;

impl Oops {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for Oops {
    fn name(&self) -> &'static str {
        "oops"
    }
    fn ptype(&self) -> PluginType {
        PluginType::Full
    }
    fn description(&self) -> &'static str {
        "Coloriser for oops proxy logs."
    }

    fn handle(&self, line: &str, sink: &mut dyn OutputSink) -> io::Result<HandleResult> {
        let caps = match RE_OOPS.captures(line) {
            Some(c) => c,
            None => return Ok(HandleResult::NoMatch),
        };
        let date = caps.get(1).unwrap().as_str();
        let sp1 = caps.get(4).unwrap().as_str();
        let id = caps.get(5).unwrap().as_str();
        let field = caps.get(6).unwrap().as_str();
        let sp2 = caps.get(7).unwrap().as_str();
        let value = caps.get(8).unwrap().as_str();
        let etc = caps.get(9).unwrap().as_str();

        sink.emit(Color::Date, date)?;
        sink.emit(Color::Default, sp1)?;
        sink.emit(Color::PidB, "[")?;
        sink.emit(Color::Proc, id)?;
        sink.emit(Color::PidB, "]")?;
        sink.emit(Color::Keyword, "statistics()")?;
        sink.emit(Color::Default, ":")?;
        sink.space()?;
        sink.emit(Color::Field, field)?;
        sink.emit(Color::Default, sp2)?;
        sink.emit(Color::Default, ":")?;
        sink.space()?;
        sink.emit(Color::Numbers, value)?;
        sink.emit(Color::Default, etc)?;
        sink.newline()?;
        Ok(HandleResult::Consumed)
    }
}
