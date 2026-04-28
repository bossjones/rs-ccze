//! Generic xferlog colorizer. Port of `mod_xferlog.c`.

use crate::color::Color;
use crate::plugin::{HandleResult, Plugin, PluginType};
use crate::sink::OutputSink;
use once_cell::sync::Lazy;
use regex::Regex;
use std::io;

static RE_XFERLOG: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"^(... ... +\d{1,2} +\d{1,2}:\d{1,2}:\d{1,2} \d+) (\d+) ([^ ]+) (\d+) (\S+) (a|b) (C|U|T|_) (o|i) (a|g|r) ([^ ]+) ([^ ]+) (0|1) ([^ ]+) (c|i)",
    )
    .unwrap()
});

pub struct Xferlog;

impl Xferlog {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for Xferlog {
    fn name(&self) -> &'static str {
        "xferlog"
    }
    fn ptype(&self) -> PluginType {
        PluginType::Full
    }
    fn description(&self) -> &'static str {
        "Generic xferlog coloriser."
    }

    fn handle(&self, line: &str, sink: &mut dyn OutputSink) -> io::Result<HandleResult> {
        let caps = match RE_XFERLOG.captures(line) {
            Some(c) => c,
            None => return Ok(HandleResult::NoMatch),
        };
        let curtime = caps.get(1).unwrap().as_str();
        let transtime = caps.get(2).unwrap().as_str();
        let host = caps.get(3).unwrap().as_str();
        let fsize = caps.get(4).unwrap().as_str();
        let fname = caps.get(5).unwrap().as_str();
        let transtype = caps.get(6).unwrap().as_str();
        let actionflag = caps.get(7).unwrap().as_str();
        let direction = caps.get(8).unwrap().as_str();
        let amode = caps.get(9).unwrap().as_str();
        let user = caps.get(10).unwrap().as_str();
        let service = caps.get(11).unwrap().as_str();
        let amethod = caps.get(12).unwrap().as_str();
        let auid = caps.get(13).unwrap().as_str();
        let status = caps.get(14).unwrap().as_str();

        sink.emit(Color::Date, curtime)?;
        sink.space()?;
        sink.emit(Color::GetTime, transtime)?;
        sink.space()?;
        sink.emit(Color::Host, host)?;
        sink.space()?;
        sink.emit(Color::GetSize, fsize)?;
        sink.space()?;
        sink.emit(Color::Dir, fname)?;
        sink.space()?;
        sink.emit(Color::PidB, transtype)?;
        sink.space()?;
        sink.emit(Color::FtpCodes, actionflag)?;
        sink.space()?;
        sink.emit(Color::FtpCodes, direction)?;
        sink.space()?;
        sink.emit(Color::FtpCodes, amode)?;
        sink.space()?;
        sink.emit(Color::User, user)?;
        sink.space()?;
        sink.emit(Color::Service, service)?;
        sink.space()?;
        sink.emit(Color::FtpCodes, amethod)?;
        sink.space()?;
        sink.emit(Color::User, auid)?;
        sink.space()?;
        sink.emit(Color::FtpCodes, status)?;
        sink.newline()?;
        Ok(HandleResult::Consumed)
    }
}
