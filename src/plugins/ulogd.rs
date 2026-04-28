//! ulogd kernel firewall sub-log colorizer. Port of `mod_ulogd.c`. Partial type.

use crate::color::Color;
use crate::plugin::{HandleResult, Plugin, PluginType};
use crate::sink::OutputSink;
use crate::wordcolor;
use once_cell::sync::Lazy;
use regex::Regex;
use std::io;

static RE_ULOGD: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(IN|OUT|MAC|TTL|SRC|TOS|PREC|SPT)=").unwrap());

pub struct Ulogd;

impl Ulogd {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for Ulogd {
    fn name(&self) -> &'static str {
        "ulogd"
    }
    fn ptype(&self) -> PluginType {
        PluginType::Partial
    }
    fn description(&self) -> &'static str {
        "Coloriser for ulogd sub-logs."
    }

    fn handle(&self, line: &str, sink: &mut dyn OutputSink) -> io::Result<HandleResult> {
        if !RE_ULOGD.is_match(line) {
            return Ok(HandleResult::NoMatch);
        }
        // C code splits with ccze_strbrk(' ') which produces empty tokens for
        // consecutive spaces. str::split(' ') is byte-equivalent. The C source
        // hardcodes slookup=1 in its wordcolor_process_one call, so mirror that
        // here — our `service_known`/`protocol_known`/`user_known` stubs all
        // return false, which matches the Docker reference (debian-slim has no
        // /etc/services or /etc/protocols).
        for word in line.split(' ') {
            if let Some(eq_idx) = word.find('=') {
                let field = &word[..eq_idx];
                let value = &word[eq_idx + 1..];
                sink.emit(Color::Field, field)?;
                sink.emit(Color::Default, "=")?;
                wordcolor::process_one(value, sink, true)?;
                sink.space()?;
            } else {
                sink.emit(Color::Field, word)?;
                sink.space()?;
            }
        }
        Ok(HandleResult::Consumed)
    }
}
