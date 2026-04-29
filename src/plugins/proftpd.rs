//! ProFTPD access and auth log colorizer. Port of `mod_proftpd.c`.

use crate::color::Color;
use crate::plugin::{HandleResult, Plugin, PluginType};
use crate::sink::OutputSink;
use once_cell::sync::Lazy;
use regex::Regex;
use std::io;

static RE_ACCESS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"^(\d+\.\d+\.\d+\.\d+) (\S+) (\S+) \[(\d{2}/.{3}/\d{4}:\d{2}:\d{2}:\d{2} [\-\+]\d{4})\] "([A-Z]+) ([^"]+)" (\d{3}) (-|\d+)"#,
    )
    .unwrap()
});
static RE_AUTH: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"^(\S+) ftp server \[(\d+)\] (\d+\.\d+\.\d+\.\d+) \[(\d{2}/.{3}/\d{4}:\d{2}:\d{2}:\d{2} [\-\+]\d{4})\] "([A-Z]+) ([^"]+)" (\d{3})"#,
    )
    .unwrap()
});

pub struct Proftpd;

impl Proftpd {
    pub fn new() -> Self {
        Self
    }

    fn handle_access(&self, line: &str, sink: &mut dyn OutputSink) -> io::Result<bool> {
        let caps = match RE_ACCESS.captures(line) {
            Some(c) => c,
            None => return Ok(false),
        };
        let host = caps.get(1).unwrap().as_str();
        let user = caps.get(2).unwrap().as_str();
        let auser = caps.get(3).unwrap().as_str();
        let date = caps.get(4).unwrap().as_str();
        let command = caps.get(5).unwrap().as_str();
        let file = caps.get(6).unwrap().as_str();
        let ftpcode = caps.get(7).unwrap().as_str();
        let size = caps.get(8).unwrap().as_str();

        sink.emit(Color::Host, host)?;
        sink.space()?;
        sink.emit(Color::User, user)?;
        sink.space()?;
        sink.emit(Color::User, auser)?;
        sink.space()?;
        sink.emit(Color::Default, "[")?;
        sink.emit(Color::Date, date)?;
        sink.emit(Color::Default, "]")?;
        sink.space()?;
        sink.emit(Color::Default, "\"")?;
        sink.emit(Color::Keyword, command)?;
        sink.space()?;
        sink.emit(Color::Uri, file)?;
        sink.emit(Color::Default, "\"")?;
        sink.space()?;
        sink.emit(Color::FtpCodes, ftpcode)?;
        sink.space()?;
        sink.emit(Color::GetSize, size)?;
        sink.newline()?;
        Ok(true)
    }

    fn handle_auth(&self, line: &str, sink: &mut dyn OutputSink) -> io::Result<bool> {
        let caps = match RE_AUTH.captures(line) {
            Some(c) => c,
            None => return Ok(false),
        };
        let servhost = caps.get(1).unwrap().as_str();
        let pid = caps.get(2).unwrap().as_str();
        let remhost = caps.get(3).unwrap().as_str();
        let date = caps.get(4).unwrap().as_str();
        let cmd = caps.get(5).unwrap().as_str();
        let value = caps.get(6).unwrap().as_str();
        let ftpcode = caps.get(7).unwrap().as_str();

        sink.emit(Color::Host, servhost)?;
        sink.space()?;
        sink.emit(Color::Default, "ftp server")?;
        sink.space()?;
        sink.emit(Color::PidB, "[")?;
        sink.emit(Color::Pid, pid)?;
        sink.emit(Color::PidB, "]")?;
        sink.space()?;
        sink.emit(Color::Host, remhost)?;
        sink.space()?;
        sink.emit(Color::Default, "[")?;
        sink.emit(Color::Date, date)?;
        sink.emit(Color::Default, "]")?;
        sink.space()?;
        sink.emit(Color::Default, "\"")?;
        sink.emit(Color::Keyword, cmd)?;
        sink.space()?;
        sink.emit(Color::Default, value)?;
        sink.emit(Color::Default, "\"")?;
        sink.space()?;
        sink.emit(Color::FtpCodes, ftpcode)?;
        sink.newline()?;
        Ok(true)
    }
}

impl Plugin for Proftpd {
    fn name(&self) -> &'static str {
        "proftpd"
    }
    fn ptype(&self) -> PluginType {
        PluginType::Full
    }
    fn description(&self) -> &'static str {
        "Coloriser for proftpd access and auth logs."
    }

    fn handle(&self, line: &str, sink: &mut dyn OutputSink) -> io::Result<HandleResult> {
        if self.handle_access(line, sink)? {
            return Ok(HandleResult::Consumed);
        }
        if self.handle_auth(line, sink)? {
            return Ok(HandleResult::Consumed);
        }
        Ok(HandleResult::NoMatch)
    }
}
