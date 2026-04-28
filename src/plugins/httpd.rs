//! HTTPD access + error log colorizer. Port of `mod_httpd.c`.

use crate::color::Color;
use crate::plugin::{HandleResult, Plugin, PluginType};
use crate::sink::OutputSink;
use once_cell::sync::Lazy;
use regex::Regex;
use std::io;

static RE_ACCESS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"^(\S*)\s(\S*)?\s?-\s(\S+)\s(\[\d{1,2}/\S*/\d{4}:\d{2}:\d{2}:\d{2}.{0,6}[^\]]*\])\s("([^ "]+)\s*[^"]*")\s(\d{3})\s(\d+|-)\s*(.*)$"#,
    )
    .unwrap()
});
static RE_ERROR: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\[\w{3}\s\w{3}\s{1,2}\d{1,2}\s\d{2}:\d{2}:\d{2}\s\d{4}\])\s(\[\w*\])\s(.*)$")
        .unwrap()
});

fn http_action(method: &str) -> Color {
    match method.to_ascii_uppercase().as_str() {
        "GET" => Color::HttpGet,
        "POST" => Color::HttpPost,
        "HEAD" => Color::HttpHead,
        "PUT" => Color::HttpPut,
        "CONNECT" => Color::HttpConnect,
        "TRACE" => Color::HttpTrace,
        _ => Color::Unknown,
    }
}

fn error_level_color(level: &str) -> Color {
    if level.contains("debug") || level.contains("info") || level.contains("notice") {
        Color::Debug
    } else if level.contains("warn") {
        Color::Warning
    } else if level.contains("error")
        || level.contains("crit")
        || level.contains("alert")
        || level.contains("emerg")
    {
        Color::Error
    } else {
        Color::Unknown
    }
}

pub struct Httpd;

impl Httpd {
    pub fn new() -> Self {
        Self
    }

    fn handle_access(&self, line: &str, sink: &mut dyn OutputSink) -> io::Result<bool> {
        let caps = match RE_ACCESS.captures(line) {
            Some(c) => c,
            None => return Ok(false),
        };
        let vhost = caps.get(1).map_or("", |m| m.as_str());
        let host = caps.get(2).map_or("", |m| m.as_str());
        let user = caps.get(3).map_or("", |m| m.as_str());
        let date = caps.get(4).map_or("", |m| m.as_str());
        let full_action = caps.get(5).map_or("", |m| m.as_str());
        let method = caps.get(6).map_or("", |m| m.as_str());
        let http_code = caps.get(7).map_or("", |m| m.as_str());
        let gsize = caps.get(8).map_or("", |m| m.as_str());
        let other = caps.get(9).map_or("", |m| m.as_str());

        sink.emit(Color::Host, vhost)?;
        sink.space()?;
        sink.emit(Color::Host, host)?;
        // The C source emits the inter-host space only when `host[0]` is set —
        // so an empty host produces `<host></host><default>-</default>` with
        // no separating space tag.
        if !host.is_empty() {
            sink.space()?;
        }
        sink.emit(Color::Default, "-")?;
        sink.space()?;

        sink.emit(Color::User, user)?;
        sink.space()?;

        sink.emit(Color::Date, date)?;
        sink.space()?;

        sink.emit(http_action(method), full_action)?;
        sink.space()?;

        sink.emit(Color::HttpCodes, http_code)?;
        sink.space()?;

        sink.emit(Color::GetSize, gsize)?;
        sink.space()?;

        sink.emit(Color::Default, other)?;
        sink.newline()?;
        Ok(true)
    }

    fn handle_error(&self, line: &str, sink: &mut dyn OutputSink) -> io::Result<bool> {
        let caps = match RE_ERROR.captures(line) {
            Some(c) => c,
            None => return Ok(false),
        };
        let date = caps.get(1).map_or("", |m| m.as_str());
        let level = caps.get(2).map_or("", |m| m.as_str());
        let msg = caps.get(3).map_or("", |m| m.as_str());

        sink.emit(Color::Date, date)?;
        sink.space()?;
        let lcol = error_level_color(level);
        sink.emit(lcol, level)?;
        sink.space()?;
        sink.emit(lcol, msg)?;
        sink.newline()?;
        Ok(true)
    }
}

impl Plugin for Httpd {
    fn name(&self) -> &'static str {
        "httpd"
    }
    fn ptype(&self) -> PluginType {
        PluginType::Full
    }
    fn description(&self) -> &'static str {
        "Coloriser for generic HTTPD access and error logs."
    }

    fn handle(&self, line: &str, sink: &mut dyn OutputSink) -> io::Result<HandleResult> {
        if self.handle_access(line, sink)? {
            return Ok(HandleResult::Consumed);
        }
        if self.handle_error(line, sink)? {
            return Ok(HandleResult::Consumed);
        }
        Ok(HandleResult::NoMatch)
    }
}
