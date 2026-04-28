//! Postfix sub-log colorizer. Port of `mod_postfix.c`. Partial type.

use crate::color::Color;
use crate::plugin::{HandleResult, Plugin, PluginType};
use crate::sink::OutputSink;
use crate::wordcolor;
use once_cell::sync::Lazy;
use regex::Regex;
use std::io;

static RE_POSTFIX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^([\dA-F]+): ((client|to|message-id|uid|resent-message-id|from)(=.*))").unwrap()
});

pub struct Postfix;

impl Postfix {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for Postfix {
    fn name(&self) -> &'static str {
        "postfix"
    }
    fn ptype(&self) -> PluginType {
        PluginType::Partial
    }
    fn description(&self) -> &'static str {
        "Coloriser for postfix(1) sub-logs."
    }

    fn handle(&self, line: &str, sink: &mut dyn OutputSink) -> io::Result<HandleResult> {
        let caps = match RE_POSTFIX.captures(line) {
            Some(c) => c,
            None => return Ok(HandleResult::NoMatch),
        };
        let spoolid = caps.get(1).unwrap().as_str();
        let message = caps.get(2).unwrap().as_str();

        sink.emit(Color::UniqN, spoolid)?;
        sink.emit(Color::Default, ": ")?;

        // Comma-split fields. Mirrors the C `ccze_strbrk(s, ',')` loop in
        // mod_postfix.c. The loop emits a `,` after each token (matched or
        // not) as long as another token follows; the unmatched-token branch
        // emits the verbatim token + ",", then breaks.
        let tokens: Vec<&str> = message.split(',').collect();
        for (i, tok) in tokens.iter().enumerate() {
            let has_more = i + 1 < tokens.len();
            if let Some(eq_idx) = tok.find('=') {
                let field = &tok[..eq_idx];
                let value = &tok[eq_idx + 1..];
                sink.emit(Color::Field, field)?;
                sink.emit(Color::Default, "=")?;
                // The C source hardcodes `slookup=1` here regardless of CLI.
                // Stubs in our wordcolor return false either way under the
                // Docker reference (no /etc/services in the slim image), so
                // output is byte-identical.
                wordcolor::process_one(value, sink, true)?;
                if has_more {
                    sink.emit(Color::Default, ",")?;
                }
            } else {
                // No '=' — emit verbatim, append a trailing "," (the C source
                // does this regardless of whether more tokens follow), then
                // stop processing further tokens.
                sink.emit(Color::Default, tok)?;
                sink.emit(Color::Default, ",")?;
                break;
            }
        }
        Ok(HandleResult::Consumed)
    }
}
