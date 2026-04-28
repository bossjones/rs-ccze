//! Squid access / cache / store log colorizer. Port of `mod_squid.c`.

use crate::color::Color;
use crate::plugin::{HandleResult, Plugin, PluginType};
use crate::sink::OutputSink;
use once_cell::sync::Lazy;
use regex::Regex;
use std::io;

static RE_ACCESS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"^(\d{9,10}\.\d{3})(\s+)(\d+)\s(\S+)\s(\w+)/(\d{3})\s(\d+)\s(\w+)\s(\S+)\s(\S+)\s(\w+)/([\d\.]+|-)\s(.*)",
    )
    .unwrap()
});
static RE_CACHE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\d{4}/\d{2}/\d{2}\s(\d{2}:){2}\d{2}\|)\s(.*)$").unwrap()
});
static RE_STORE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"^([\d\.]+)\s(\w+)\s(\-?[\dA-F]+)\s+(\S+)\s([\dA-F]+)(\s+)(\d{3}|\?)(\s+)(\-?[\d\?]+)(\s+)(\-?[\d\?]+)(\s+)(\-?[\d\?]+)\s(\S+)\s(\-?[\d|\?]+)/(\-?[\d|\?]+)\s(\S+)\s(.*)",
    )
    .unwrap()
});

/// Map a squid `action` token (`TCP_HIT`, `TCP_MISS`, `TCP_DENIED`, …) to the
/// colour the C source uses. The C source uses `strstr(action, "ERR") == action`
/// for the prefix check (i.e. `starts_with`) and plain `strstr` for substrings.
fn proxy_action(action: &str) -> Color {
    if action.starts_with("ERR") {
        Color::Error
    } else if action.contains("MISS") {
        Color::ProxyMiss
    } else if action.contains("HIT") {
        Color::ProxyHit
    } else if action.contains("DENIED") {
        Color::ProxyDenied
    } else if action.contains("REFRESH") {
        Color::ProxyRefresh
    } else if action.contains("SWAPFAIL") {
        Color::ProxySwapfail
    } else if action.contains("NONE") {
        Color::Debug
    } else {
        Color::Unknown
    }
}

fn proxy_hierarch(hierar: &str) -> Color {
    if hierar.starts_with("NO") {
        Color::Warning
    } else if hierar.contains("DIRECT") {
        Color::ProxyDirect
    } else if hierar.contains("PARENT") {
        Color::ProxyParent
    } else if hierar.contains("MISS") {
        Color::ProxyMiss
    } else {
        Color::Unknown
    }
}

#[allow(dead_code)]
fn proxy_tag(tag: &str) -> Color {
    if tag.contains("CREATE") {
        Color::ProxyCreate
    } else if tag.contains("SWAPIN") {
        Color::ProxySwapin
    } else if tag.contains("SWAPOUT") {
        Color::ProxySwapout
    } else if tag.contains("RELEASE") {
        Color::ProxyRelease
    } else {
        Color::Unknown
    }
}

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

pub struct Squid;

impl Squid {
    pub fn new() -> Self {
        Self
    }

    fn handle_access(&self, line: &str, sink: &mut dyn OutputSink) -> io::Result<bool> {
        let caps = match RE_ACCESS.captures(line) {
            Some(c) => c,
            None => return Ok(false),
        };
        let date = caps.get(1).unwrap().as_str();
        let espace = caps.get(2).unwrap().as_str();
        let elaps = caps.get(3).unwrap().as_str();
        let host = caps.get(4).unwrap().as_str();
        let action = caps.get(5).unwrap().as_str();
        let httpc = caps.get(6).unwrap().as_str();
        let gsize = caps.get(7).unwrap().as_str();
        let method = caps.get(8).unwrap().as_str();
        let uri = caps.get(9).unwrap().as_str();
        let ident = caps.get(10).unwrap().as_str();
        let hierar = caps.get(11).unwrap().as_str();
        let fhost = caps.get(12).unwrap().as_str();
        let ctype = caps.get(13).unwrap().as_str();

        sink.emit(Color::Date, date)?;
        sink.emit(Color::Default, espace)?;
        sink.emit(Color::GetTime, elaps)?;
        sink.space()?;

        sink.emit(Color::Host, host)?;
        sink.space()?;

        sink.emit(proxy_action(action), action)?;
        sink.emit(Color::Default, "/")?;
        sink.emit(Color::HttpCodes, httpc)?;
        sink.space()?;

        sink.emit(Color::GetSize, gsize)?;
        sink.space()?;

        sink.emit(http_action(method), method)?;
        sink.space()?;

        sink.emit(Color::Uri, uri)?;
        sink.space()?;

        sink.emit(Color::Ident, ident)?;
        sink.space()?;

        sink.emit(proxy_hierarch(hierar), hierar)?;
        sink.emit(Color::Default, "/")?;
        sink.emit(Color::Host, fhost)?;
        sink.space()?;

        sink.emit(Color::CType, ctype)?;
        sink.newline()?;
        Ok(true)
    }

    fn handle_cache(
        &self,
        line: &str,
        sink: &mut dyn OutputSink,
    ) -> io::Result<Option<HandleResult>> {
        let caps = match RE_CACHE.captures(line) {
            Some(c) => c,
            None => return Ok(None),
        };
        let date = caps.get(1).unwrap().as_str();
        let other = caps.get(3).unwrap().as_str();

        sink.emit(Color::Date, date)?;
        sink.space()?;
        Ok(Some(HandleResult::Remainder(other.to_owned())))
    }

    fn handle_store(&self, line: &str, sink: &mut dyn OutputSink) -> io::Result<bool> {
        let caps = match RE_STORE.captures(line) {
            Some(c) => c,
            None => return Ok(false),
        };
        let date = caps.get(1).unwrap().as_str();
        let tag = caps.get(2).unwrap().as_str();
        let swapnum = caps.get(3).unwrap().as_str();
        let swapname = caps.get(4).unwrap().as_str();
        let swapsum = caps.get(5).unwrap().as_str();
        let space1 = caps.get(6).unwrap().as_str();
        let hcode = caps.get(7).unwrap().as_str();
        let space2 = caps.get(8).unwrap().as_str();
        let hdate = caps.get(9).unwrap().as_str();
        let space3 = caps.get(10).unwrap().as_str();
        let lmdate = caps.get(11).unwrap().as_str();
        let space4 = caps.get(12).unwrap().as_str();
        let expire = caps.get(13).unwrap().as_str();
        let ctype = caps.get(14).unwrap().as_str();
        let size = caps.get(15).unwrap().as_str();
        let read = caps.get(16).unwrap().as_str();
        let method = caps.get(17).unwrap().as_str();
        let uri = caps.get(18).unwrap().as_str();

        sink.emit(Color::Date, date)?;
        sink.space()?;
        sink.emit(proxy_tag(tag), tag)?;
        sink.space()?;
        sink.emit(Color::SwapNum, swapnum)?;
        sink.space()?;
        sink.emit(Color::SwapNum, swapname)?;
        sink.space()?;
        sink.emit(Color::SwapNum, swapsum)?;
        sink.emit(Color::Default, space1)?;
        sink.emit(Color::HttpCodes, hcode)?;
        sink.emit(Color::Default, space2)?;
        sink.emit(Color::Date, hdate)?;
        sink.emit(Color::Default, space3)?;
        sink.emit(Color::Date, lmdate)?;
        sink.emit(Color::Default, space4)?;
        sink.emit(Color::Date, expire)?;
        sink.space()?;
        sink.emit(Color::CType, ctype)?;
        sink.space()?;
        sink.emit(Color::GetSize, size)?;
        sink.emit(Color::Default, "/")?;
        sink.emit(Color::GetSize, read)?;
        sink.space()?;
        sink.emit(http_action(method), method)?;
        sink.space()?;
        sink.emit(Color::Uri, uri)?;
        sink.newline()?;
        Ok(true)
    }
}

impl Plugin for Squid {
    fn name(&self) -> &'static str {
        "squid"
    }
    fn ptype(&self) -> PluginType {
        PluginType::Full
    }
    fn description(&self) -> &'static str {
        "Coloriser for squid access, store and cache logs."
    }

    fn handle(&self, line: &str, sink: &mut dyn OutputSink) -> io::Result<HandleResult> {
        // Order in C source is access → store → cache.
        if self.handle_access(line, sink)? {
            return Ok(HandleResult::Consumed);
        }
        if self.handle_store(line, sink)? {
            return Ok(HandleResult::Consumed);
        }
        if let Some(r) = self.handle_cache(line, sink)? {
            return Ok(r);
        }
        Ok(HandleResult::NoMatch)
    }
}
