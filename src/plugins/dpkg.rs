//! dpkg log colorizer. Port of `mod_dpkg.c`.

use crate::color::Color;
use crate::plugin::{HandleResult, Plugin, PluginType};
use crate::sink::OutputSink;
use once_cell::sync::Lazy;
use regex::Regex;
use std::io;

static RE_STATUS: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^([-\d]{10}\s[:\d]{8})\sstatus\s(\S+)\s(\S+)\s(\S+)$").unwrap());
static RE_ACTION: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^([-\d]{10}\s[:\d]{8})\s(install|upgrade|remove|purge)\s(\S+)\s(\S+)\s(\S+)$")
        .unwrap()
});
static RE_CONFFILE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^([-\d]{10}\s[:\d]{8})\sconffile\s(\S+)\s(install|keep)$").unwrap());

pub struct Dpkg;

impl Dpkg {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for Dpkg {
    fn name(&self) -> &'static str {
        "dpkg"
    }
    fn ptype(&self) -> PluginType {
        PluginType::Full
    }
    fn description(&self) -> &'static str {
        "Coloriser for dpkg logs."
    }

    fn handle(&self, line: &str, sink: &mut dyn OutputSink) -> io::Result<HandleResult> {
        if let Some(caps) = RE_STATUS.captures(line) {
            let date = caps.get(1).unwrap().as_str();
            let state = caps.get(2).unwrap().as_str();
            let pkg = caps.get(3).unwrap().as_str();
            let version = caps.get(4).unwrap().as_str();
            sink.emit(Color::Date, date)?;
            sink.space()?;
            sink.emit(Color::Keyword, "status")?;
            sink.space()?;
            sink.emit(Color::PkgStatus, state)?;
            sink.space()?;
            sink.emit(Color::Pkg, pkg)?;
            sink.space()?;
            sink.emit(Color::Default, version)?;
            sink.newline()?;
            return Ok(HandleResult::Consumed);
        }
        if let Some(caps) = RE_ACTION.captures(line) {
            let date = caps.get(1).unwrap().as_str();
            let action = caps.get(2).unwrap().as_str();
            let pkg = caps.get(3).unwrap().as_str();
            let installed = caps.get(4).unwrap().as_str();
            let available = caps.get(5).unwrap().as_str();
            sink.emit(Color::Date, date)?;
            sink.space()?;
            sink.emit(Color::Keyword, action)?;
            sink.space()?;
            sink.emit(Color::Pkg, pkg)?;
            sink.space()?;
            sink.emit(Color::Default, installed)?;
            sink.space()?;
            sink.emit(Color::Default, available)?;
            sink.newline()?;
            return Ok(HandleResult::Consumed);
        }
        if let Some(caps) = RE_CONFFILE.captures(line) {
            let date = caps.get(1).unwrap().as_str();
            let filename = caps.get(2).unwrap().as_str();
            let decision = caps.get(3).unwrap().as_str();
            sink.emit(Color::Date, date)?;
            sink.space()?;
            sink.emit(Color::Keyword, "conffile")?;
            sink.space()?;
            sink.emit(Color::File, filename)?;
            sink.space()?;
            sink.emit(Color::Keyword, decision)?;
            sink.newline()?;
            return Ok(HandleResult::Consumed);
        }
        Ok(HandleResult::NoMatch)
    }
}
