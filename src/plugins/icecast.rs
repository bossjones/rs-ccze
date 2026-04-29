//! Icecast(8) log colorizer. Port of `mod_icecast.c`.

use crate::color::Color;
use crate::plugin::{HandleResult, Plugin, PluginType};
use crate::sink::OutputSink;
use once_cell::sync::Lazy;
use regex::Regex;
use std::io;

static RE_ICECAST: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\[\d+/.../\d+:\d+:\d+:\d+\]) (Admin)? *(\[(\d+)?:?([^\]]*)\]) (.*)$").unwrap()
});
static RE_ICECAST_USAGE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"^(\[\d+/.../\d+:\d+:\d+:\d+\]) (\[(\d+):([^\]]*)\]) (\[\d+/.../\d+:\d+:\d+:\d+\]) Bandwidth:([\d\.]+)([^ ]*) Sources:(\d+) Clients:(\d+) Admins:(\d+)",
    )
    .unwrap()
});

pub struct Icecast;

impl Icecast {
    pub fn new() -> Self {
        Self
    }
}

impl Icecast {
    fn handle_usage(&self, line: &str, sink: &mut dyn OutputSink) -> io::Result<bool> {
        let caps = match RE_ICECAST_USAGE.captures(line) {
            Some(c) => c,
            None => return Ok(false),
        };
        let date = caps.get(1).unwrap().as_str();
        let threadno = caps.get(3).unwrap().as_str();
        let thread = caps.get(4).unwrap().as_str();
        let date2 = caps.get(5).unwrap().as_str();
        let bw = caps.get(6).unwrap().as_str();
        let unit = caps.get(7).unwrap().as_str();
        let src = caps.get(8).unwrap().as_str();
        let clients = caps.get(9).unwrap().as_str();
        let admins = caps.get(10).unwrap().as_str();

        sink.emit(Color::Date, date)?;
        sink.space()?;
        sink.emit(Color::PidB, "[")?;
        sink.emit(Color::Numbers, threadno)?;
        sink.emit(Color::Default, ":")?;
        sink.emit(Color::Keyword, thread)?;
        sink.emit(Color::PidB, "]")?;
        sink.space()?;
        sink.emit(Color::Date, date2)?;
        sink.space()?;
        sink.emit(Color::Keyword, "Bandwidth:")?;
        sink.emit(Color::Numbers, bw)?;
        sink.emit(Color::Default, unit)?;
        sink.space()?;
        sink.emit(Color::Keyword, "Sources:")?;
        sink.emit(Color::Numbers, src)?;
        sink.space()?;
        sink.emit(Color::Keyword, "Clients:")?;
        sink.emit(Color::Numbers, clients)?;
        sink.space()?;
        sink.emit(Color::Keyword, "Admins:")?;
        sink.emit(Color::Numbers, admins)?;
        sink.newline()?;
        Ok(true)
    }

    fn handle_regular(
        &self,
        line: &str,
        sink: &mut dyn OutputSink,
    ) -> io::Result<Option<HandleResult>> {
        let caps = match RE_ICECAST.captures(line) {
            Some(c) => c,
            None => return Ok(None),
        };
        let date = caps.get(1).unwrap().as_str();
        let admin = caps.get(2).map_or("", |m| m.as_str());
        // Group 3 is the wrapping `[...]`; we use 4 (threadno) and 5 (thread).
        let threadno = caps.get(4).map_or("", |m| m.as_str());
        let thread = caps.get(5).map_or("", |m| m.as_str());
        let rest = caps.get(6).unwrap().as_str();

        sink.emit(Color::Date, date)?;
        sink.space()?;
        if !admin.is_empty() {
            sink.emit(Color::Keyword, admin)?;
            sink.space()?;
            sink.emit(Color::PidB, "[")?;
            sink.emit(Color::Host, thread)?;
            sink.emit(Color::PidB, "]")?;
        } else {
            sink.emit(Color::PidB, "[")?;
            sink.emit(Color::Numbers, threadno)?;
            sink.emit(Color::Default, ":")?;
            sink.emit(Color::Keyword, thread)?;
            sink.emit(Color::PidB, "]")?;
        }
        sink.space()?;
        Ok(Some(HandleResult::Remainder(rest.to_owned())))
    }
}

impl Plugin for Icecast {
    fn name(&self) -> &'static str {
        "icecast"
    }
    fn ptype(&self) -> PluginType {
        PluginType::Full
    }
    fn description(&self) -> &'static str {
        "Coloriser for Icecast(8) logs."
    }

    fn handle(&self, line: &str, sink: &mut dyn OutputSink) -> io::Result<HandleResult> {
        if self.handle_usage(line, sink)? {
            return Ok(HandleResult::Consumed);
        }
        if let Some(r) = self.handle_regular(line, sink)? {
            return Ok(r);
        }
        Ok(HandleResult::NoMatch)
    }
}
