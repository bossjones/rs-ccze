//! ftpstats (pure-ftpd) log colorizer. Port of `mod_ftpstats.c`.

use crate::color::Color;
use crate::plugin::{HandleResult, Plugin, PluginType};
use crate::sink::OutputSink;
use once_cell::sync::Lazy;
use regex::Regex;
use std::io;

static RE_FTPSTATS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"^(\d{9,10})\s([\da-f]+\.[\da-f]+)\s([^\s]+)\s([^\s]+)\s(U|D)\s(\d+)\s(\d+)\s(.*)$",
    )
    .unwrap()
});

pub struct Ftpstats;

impl Ftpstats {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for Ftpstats {
    fn name(&self) -> &'static str {
        "ftpstats"
    }
    fn ptype(&self) -> PluginType {
        PluginType::Full
    }
    fn description(&self) -> &'static str {
        "Coloriser for ftpstats (pure-ftpd) logs."
    }

    fn handle(&self, line: &str, sink: &mut dyn OutputSink) -> io::Result<HandleResult> {
        let caps = match RE_FTPSTATS.captures(line) {
            Some(c) => c,
            None => return Ok(HandleResult::NoMatch),
        };
        let date = caps.get(1).unwrap().as_str();
        let sessionid = caps.get(2).unwrap().as_str();
        let user = caps.get(3).unwrap().as_str();
        let host = caps.get(4).unwrap().as_str();
        let xfer_type = caps.get(5).unwrap().as_str();
        let size = caps.get(6).unwrap().as_str();
        let duration = caps.get(7).unwrap().as_str();
        let file = caps.get(8).unwrap().as_str();

        sink.emit(Color::Date, date)?;
        sink.space()?;
        sink.emit(Color::UniqN, sessionid)?;
        sink.space()?;
        sink.emit(Color::User, user)?;
        sink.space()?;
        sink.emit(Color::Host, host)?;
        sink.space()?;
        sink.emit(Color::FtpCodes, xfer_type)?;
        sink.space()?;
        sink.emit(Color::GetSize, size)?;
        sink.space()?;
        sink.emit(Color::Date, duration)?;
        sink.space()?;
        sink.emit(Color::Dir, file)?;
        sink.newline()?;
        Ok(HandleResult::Consumed)
    }
}
