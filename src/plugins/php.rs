//! PHP error log colorizer. Port of `mod_php.c`.

use crate::color::Color;
use crate::plugin::{HandleResult, Plugin, PluginType};
use crate::sink::OutputSink;
use once_cell::sync::Lazy;
use regex::Regex;
use std::io;

static RE_PHP: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(\[\d+-...-\d+ \d+:\d+:\d+\]) PHP (.*)$").unwrap());

pub struct Php;

impl Php {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for Php {
    fn name(&self) -> &'static str {
        "php"
    }
    fn ptype(&self) -> PluginType {
        PluginType::Full
    }
    fn description(&self) -> &'static str {
        "Coloriser for PHP logs."
    }

    fn handle(&self, line: &str, sink: &mut dyn OutputSink) -> io::Result<HandleResult> {
        let caps = match RE_PHP.captures(line) {
            Some(c) => c,
            None => return Ok(HandleResult::NoMatch),
        };
        sink.emit(Color::Date, caps.get(1).unwrap().as_str())?;
        sink.space()?;
        sink.emit(Color::Keyword, "PHP")?;
        sink.space()?;
        Ok(HandleResult::Remainder(
            caps.get(2).unwrap().as_str().to_owned(),
        ))
    }
}
