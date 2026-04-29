//! Exim mail log colorizer. Port of `mod_exim.c`.

use crate::color::Color;
use crate::plugin::{HandleResult, Plugin, PluginType};
use crate::sink::OutputSink;
use once_cell::sync::Lazy;
use regex::Regex;
use std::io;

static RE_EXIM: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(\d{4}-\d{2}-\d{2}\s\d{2}:\d{2}:\d{2})\s(.*)$").unwrap());
static RE_EXIM_ACTIONTYPE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(\S{16})\s([<=\*][=>\*])\s(\S+.*)$").unwrap());
static RE_EXIM_UNIQN: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\S{16})\s(.*)$").unwrap());

pub struct Exim;

impl Exim {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for Exim {
    fn name(&self) -> &'static str {
        "exim"
    }
    fn ptype(&self) -> PluginType {
        PluginType::Full
    }
    fn description(&self) -> &'static str {
        "Coloriser for exim logs."
    }

    fn handle(&self, line: &str, sink: &mut dyn OutputSink) -> io::Result<HandleResult> {
        let caps = match RE_EXIM.captures(line) {
            Some(c) => c,
            None => return Ok(HandleResult::NoMatch),
        };
        let date = caps.get(1).unwrap().as_str();
        let msgfull = caps.get(2).unwrap().as_str();

        let mut uniqn: Option<&str> = None;
        let mut action: Option<&str> = None;
        let mut color = Color::Unknown;
        let msg: String;

        if let Some(c2) = RE_EXIM_ACTIONTYPE.captures(msgfull) {
            uniqn = Some(c2.get(1).unwrap().as_str());
            let act = c2.get(2).unwrap().as_str();
            action = Some(act);
            // Action symbol meanings — see mod_exim.c:52-57.
            // act is two ASCII chars from `[<=\*][=>\*]`.
            let bytes = act.as_bytes();
            if bytes.first() == Some(&b'<') {
                color = Color::Incoming;
            } else if bytes.get(1) == Some(&b'>') {
                color = Color::Outgoing;
            } else if bytes.first() == Some(&b'=') || bytes.first() == Some(&b'*') {
                color = Color::Error;
            }
            msg = c2.get(3).unwrap().as_str().to_owned();
        } else if let Some(c2) = RE_EXIM_UNIQN.captures(msgfull) {
            uniqn = Some(c2.get(1).unwrap().as_str());
            msg = c2.get(2).unwrap().as_str().to_owned();
        } else {
            msg = msgfull.to_owned();
        }

        sink.emit(Color::Date, date)?;
        sink.space()?;
        if let Some(u) = uniqn
            && !u.is_empty()
        {
            sink.emit(Color::UniqN, u)?;
            sink.space()?;
        }
        if let Some(a) = action
            && !a.is_empty()
        {
            sink.emit(color, a)?;
            sink.space()?;
        }
        Ok(HandleResult::Remainder(msg))
    }
}
