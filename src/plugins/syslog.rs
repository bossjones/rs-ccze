//! Generic syslog(8) log colorizer. Port of `mod_syslog.c`.

use crate::color::Color;
use crate::plugin::{HandleResult, Plugin, PluginType};
use crate::sink::OutputSink;
use once_cell::sync::Lazy;
use regex::Regex;
use std::io;

static RE_SYSLOG: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\S*\s{1,2}\d{1,2}\s\d\d:\d\d:\d\d)\s(\S+)\s+((\S+:?)\s(.*))$").unwrap()
});

pub struct Syslog;

impl Syslog {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for Syslog {
    fn name(&self) -> &'static str {
        "syslog"
    }
    fn ptype(&self) -> PluginType {
        PluginType::Full
    }
    fn description(&self) -> &'static str {
        "Generic syslog(8) log coloriser."
    }

    fn handle(&self, line: &str, sink: &mut dyn OutputSink) -> io::Result<HandleResult> {
        let caps = match RE_SYSLOG.captures(line) {
            Some(c) => c,
            None => return Ok(HandleResult::NoMatch),
        };
        let date = caps.get(1).unwrap().as_str();
        let host = caps.get(2).unwrap().as_str();
        let send = caps.get(3).unwrap().as_str();

        let is_repeat = (send.contains("last message repeated") && send.contains("times"))
            || send.contains("-- MARK --");

        // Date + host preamble
        sink.emit(Color::Date, date)?;
        sink.space()?;
        sink.emit(Color::Host, host)?;
        sink.space()?;

        if is_repeat {
            // No process token. Pass the whole `send` to wordcolor; its
            // repeat-marker fast path will colour it as <repeat>.
            return Ok(HandleResult::Remainder(send.to_owned()));
        }

        let proc_field = caps.get(4).unwrap().as_str();
        let msg = caps.get(5).unwrap().as_str();

        // Split process[pid] into name and pid if a `[` is present.
        let (proc_name, pid) = match proc_field.find('[') {
            Some(open_idx) => {
                let close_idx = proc_field[open_idx..]
                    .find(']')
                    .map(|i| open_idx + i)
                    .unwrap_or(proc_field.len());
                let pid_str = &proc_field[open_idx + 1..close_idx];
                let name = &proc_field[..open_idx];
                (name, Some(pid_str))
            }
            None => (proc_field, None),
        };

        sink.emit(Color::Proc, proc_name)?;
        if let Some(pid) = pid {
            sink.emit(Color::PidB, "[")?;
            sink.emit(Color::Pid, pid)?;
            sink.emit(Color::PidB, "]")?;
            sink.emit(Color::Proc, ":")?;
        }
        sink.space()?;

        Ok(HandleResult::Remainder(msg.to_owned()))
    }
}
