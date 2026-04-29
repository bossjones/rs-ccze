//! vsftpd(8) log colorizer. Port of `mod_vsftpd.c`.

use crate::color::Color;
use crate::plugin::{HandleResult, Plugin, PluginType};
use crate::sink::OutputSink;
use once_cell::sync::Lazy;
use regex::Regex;
use std::io;

static RE_VSFTPD: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"^(\S+\s+\S+\s+\d{1,2}\s+\d{1,2}:\d{1,2}:\d{1,2}\s+\d+)(\s+)\[pid (\d+)\]\s+(\[(\S+)\])?\s*(.*)$",
    )
    .unwrap()
});

pub struct Vsftpd;

impl Vsftpd {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for Vsftpd {
    fn name(&self) -> &'static str {
        "vsftpd"
    }
    fn ptype(&self) -> PluginType {
        PluginType::Full
    }
    fn description(&self) -> &'static str {
        "Coloriser for vsftpd(8) logs."
    }

    fn handle(&self, line: &str, sink: &mut dyn OutputSink) -> io::Result<HandleResult> {
        let caps = match RE_VSFTPD.captures(line) {
            Some(c) => c,
            None => return Ok(HandleResult::NoMatch),
        };
        let date = caps.get(1).unwrap().as_str();
        let sspace = caps.get(2).unwrap().as_str();
        let pid = caps.get(3).unwrap().as_str();
        // Group 4 wraps the optional `[user]`; group 5 is the user inside it.
        let user = caps.get(5).map_or("", |m| m.as_str());
        let other = caps.get(6).map_or("", |m| m.as_str());

        sink.emit(Color::Date, date)?;
        sink.emit(Color::Default, sspace)?;
        sink.emit(Color::PidB, "[")?;
        sink.emit(Color::Default, "pid ")?;
        sink.emit(Color::Pid, pid)?;
        sink.emit(Color::PidB, "]")?;
        sink.space()?;
        if !user.is_empty() {
            sink.emit(Color::PidB, "[")?;
            sink.emit(Color::User, user)?;
            sink.emit(Color::PidB, "]")?;
            sink.space()?;
        }
        Ok(HandleResult::Remainder(other.to_owned()))
    }
}
