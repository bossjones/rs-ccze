//! APM battery sub-log colorizer. Port of `mod_apm.c`. Partial type.

use crate::color::Color;
use crate::plugin::{HandleResult, Plugin, PluginType};
use crate::sink::OutputSink;
use once_cell::sync::Lazy;
use regex::Regex;
use std::io;

static RE_APM: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"Battery: (-?\d*)%, ((.*)charging) \((-?\d*)% ([^ ]*) (\d*:\d*:\d*)\), (\d*:\d*:\d*) (.*)",
    )
    .unwrap()
});

pub struct Apm;

impl Apm {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for Apm {
    fn name(&self) -> &'static str {
        "apm"
    }
    fn ptype(&self) -> PluginType {
        PluginType::Partial
    }
    fn description(&self) -> &'static str {
        "Coloriser for APM sub-logs."
    }

    fn handle(&self, line: &str, sink: &mut dyn OutputSink) -> io::Result<HandleResult> {
        let caps = match RE_APM.captures(line) {
            Some(c) => c,
            None => return Ok(HandleResult::NoMatch),
        };
        let battery = caps.get(1).unwrap().as_str();
        let charge = caps.get(2).unwrap().as_str();
        let rate = caps.get(4).unwrap().as_str();
        let stuff1 = caps.get(5).unwrap().as_str();
        let elapsed = caps.get(6).unwrap().as_str();
        let remain = caps.get(7).unwrap().as_str();
        let stuff2 = caps.get(8).unwrap().as_str();

        sink.emit(Color::Default, "Battery:")?;
        sink.space()?;
        sink.emit(Color::Percentage, battery)?;
        sink.emit(Color::Default, "%,")?;
        sink.space()?;
        sink.emit(Color::System, charge)?;
        sink.space()?;
        sink.emit(Color::Default, "(")?;
        sink.emit(Color::Percentage, rate)?;
        sink.emit(Color::Default, "%")?;
        sink.space()?;
        sink.emit(Color::Default, stuff1)?;
        sink.space()?;
        sink.emit(Color::Date, elapsed)?;
        sink.emit(Color::Default, "),")?;
        sink.space()?;
        sink.emit(Color::Date, remain)?;
        sink.space()?;

        Ok(HandleResult::Remainder(stuff2.to_owned()))
    }
}
