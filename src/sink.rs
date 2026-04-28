//! Output sinks. The sink is the seam between "what the plugin says" and "how
//! the user sees it." Plugins always emit `(Color, &str)` events; each sink
//! renders them differently (debug tags, ANSI escapes, HTML).

use crate::color::{
    ANSI_FG_CODE, AnsiAttr, CSS_BOLD, CSS_ITER_ORDER, CSS_NORMAL, Color, HTML_BODY_BG,
};
use crate::config::ColorOverrides;
use std::io::{self, Write};

pub trait OutputSink {
    fn emit(&mut self, color: Color, text: &str) -> io::Result<()>;
    fn space(&mut self) -> io::Result<()> {
        self.emit(Color::Default, " ")
    }
    fn newline(&mut self) -> io::Result<()>;
    fn finish(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub struct DebugSink<W: Write> {
    w: W,
}

impl<W: Write> DebugSink<W> {
    pub fn new(w: W) -> Self {
        Self { w }
    }
}

impl<W: Write> OutputSink for DebugSink<W> {
    fn emit(&mut self, color: Color, text: &str) -> io::Result<()> {
        let kw = color.keyword();
        write!(self.w, "<{kw}>{text}</{kw}>")
    }

    fn newline(&mut self) -> io::Result<()> {
        self.w.write_all(b"\n")
    }

    fn finish(&mut self) -> io::Result<()> {
        self.w.flush()
    }
}

/// Raw-ANSI sink. Mirrors the `CCZE_MODE_RAW_ANSI` branch in
/// `ccze.c:449-484`: each emit is `ESC[22m` + per-attribute escapes +
/// optional `ESC[<bg+10>m]` + `ESC[<fg>m]` + text + `ESC[0m`.
///
/// The bit-to-SGR mapping is taken verbatim from the C source:
///   0x1000 → 1 (bold)
///   0x2000 → 4 (underline)
///   0x4000 → 5 (slow blink — the C source labels this "Reverse")
///   0x8000 → 7 (reverse — the C source labels this "Blink")
/// We follow the C bit→code mapping, not the C variable names; the names are
/// just labels and don't affect the output bytes.
pub struct AnsiSink<W: Write> {
    w: W,
    transparent: bool,
    overrides: ColorOverrides,
}

impl<W: Write> AnsiSink<W> {
    pub fn new(w: W) -> Self {
        // Matches `ccze_config.transparent = 1` at ccze.c:61 — when no
        // background is configured for a colour, omit the bg escape entirely.
        Self {
            w,
            transparent: true,
            overrides: ColorOverrides::default(),
        }
    }

    pub fn with_overrides(mut self, overrides: ColorOverrides) -> Self {
        self.overrides = overrides;
        self
    }

    /// Setter for the transparent flag — `false` forces an explicit bg40 to be
    /// emitted for any colour with no bg of its own. The CLI's
    /// `-o notransparent` flips this off; nothing else uses it yet.
    #[allow(dead_code)]
    pub fn with_transparent(mut self, transparent: bool) -> Self {
        self.transparent = transparent;
        self
    }

    fn write_attr(&mut self, attr: AnsiAttr) -> io::Result<()> {
        // Reset weight first so a previous bold doesn't bleed into a
        // subsequent non-bold span (matches `ESC[22m` at ccze.c:454).
        self.w.write_all(b"\x1b[22m")?;
        if attr.bold {
            self.w.write_all(b"\x1b[1m")?;
        }
        if attr.underline {
            self.w.write_all(b"\x1b[4m")?;
        }
        if attr.reverse {
            self.w.write_all(b"\x1b[5m")?;
        }
        if attr.blink {
            self.w.write_all(b"\x1b[7m")?;
        }
        if let Some(bg_idx) = attr.bg {
            let code = ANSI_FG_CODE[bg_idx as usize] + 10;
            write!(self.w, "\x1b[{code}m")?;
        } else if !self.transparent {
            // C source: `if (c >> 8 > 0 || !ccze_config.transparent)` — when
            // not transparent, emit a default background even if no bg was
            // configured. The bg byte is 0 in the default table, which maps to
            // ANSI 30+10 = 40 (black bg).
            self.w.write_all(b"\x1b[40m")?;
        }
        let fg = ANSI_FG_CODE[attr.fg as usize];
        write!(self.w, "\x1b[{fg}m")?;
        Ok(())
    }
}

impl<W: Write> OutputSink for AnsiSink<W> {
    fn emit(&mut self, color: Color, text: &str) -> io::Result<()> {
        let attr = self.overrides.ansi_attr(color);
        self.write_attr(attr)?;
        self.w.write_all(text.as_bytes())?;
        self.w.write_all(b"\x1b[0m")?;
        Ok(())
    }

    fn newline(&mut self) -> io::Result<()> {
        self.w.write_all(b"\n")
    }

    fn finish(&mut self) -> io::Result<()> {
        // The C source's sigint_handler at ccze.c:533 emits a final `ESC[0m`
        // when shutting down RAW_ANSI mode. Mirror it so the byte count and
        // terminal state match the reference exactly.
        self.w.write_all(b"\x1b[0m")?;
        self.w.flush()
    }
}

/// Emit the per-`Color` CSS class blocks. Used both by `HtmlSink`'s preamble
/// and by the standalone `ccze --cssdump` mode (which prints just these lines
/// to stdout for embedding in an external stylesheet).
///
/// Format mirrors `ccze_colors_to_css` at `ccze-color.c:292-305`: one
/// `.ccze_<keyword> { color: …; [text-background: …;] [text-decoration: …;] }`
/// block per colour, in C-enum numeric order, separated by blank lines.
pub fn write_css_classes<W: Write>(w: &mut W, overrides: &ColorOverrides) -> io::Result<()> {
    for &c in CSS_ITER_ORDER.iter() {
        let attr = overrides.html_attr(c);
        let kw = c.keyword();
        let fg = css_fg_name(overrides, attr);
        let underline = if attr.underline {
            "\ttext-decoration: underline\n"
        } else {
            ""
        };
        match attr.bg.map(|i| css_bg_name(overrides, i)) {
            Some(bg) => writeln!(
                w,
                ".ccze_{kw} {{\n\tcolor: {fg}\n\ttext-background: {bg}\n{underline}}}\n"
            )?,
            None => writeln!(w, ".ccze_{kw} {{\n\tcolor: {fg}\n{underline}}}\n")?,
        }
    }
    Ok(())
}

fn css_fg_name(overrides: &ColorOverrides, attr: AnsiAttr) -> String {
    let table_override = if attr.bold {
        &overrides.css_bold
    } else {
        &overrides.css_normal
    };
    let default_table = if attr.bold { &CSS_BOLD } else { &CSS_NORMAL };
    table_override[attr.fg as usize]
        .clone()
        .unwrap_or_else(|| default_table[attr.fg as usize].to_owned())
}

fn css_bg_name(overrides: &ColorOverrides, idx: u8) -> String {
    overrides.css_normal[idx as usize]
        .clone()
        .unwrap_or_else(|| CSS_NORMAL[idx as usize].to_owned())
}

/// HTML sink. Mirrors the `CCZE_MODE_HTML` branch:
///   - constructor writes the `<!DOCTYPE>` + `<head>` (with embedded `<style>`
///     reproducing `ccze_colors_to_css`) + `<body bgcolor="#404040">` preamble.
///   - `emit()` writes `<font class="ccze_<keyword>">html-encoded-text</font>`.
///   - `space()` writes `<font class="ccze_default">&nbsp;</font>` (no encoding).
///   - `newline()` writes `<br>\n`.
///   - `finish()` writes `\n</body>\n</html>\n` (matches `ccze.c:531`).
///
/// The `generator` string is hard-coded to `"ccze 0.2.1"` to stay byte-exact
/// with the in-tree C reference's snapshots. A future version flag can override
/// this if we ever bump the public version number.
pub struct HtmlSink<W: Write> {
    w: W,
    preamble_written: bool,
    overrides: ColorOverrides,
}

const HTML_GENERATOR: &str = "ccze 0.2.1";

impl<W: Write> HtmlSink<W> {
    #[allow(dead_code)]
    pub fn new(w: W) -> io::Result<Self> {
        Self::with_overrides(w, ColorOverrides::default())
    }

    pub fn with_overrides(w: W, overrides: ColorOverrides) -> io::Result<Self> {
        let mut sink = Self {
            w,
            preamble_written: false,
            overrides,
        };
        sink.write_preamble()?;
        Ok(sink)
    }

    fn write_preamble(&mut self) -> io::Result<()> {
        if self.preamble_written {
            return Ok(());
        }
        // Mirrors ccze.c:629-649 verbatim.
        write!(
            self.w,
            "<!DOCTYPE HTML PUBLIC \"-//W3C//DTD HTML 4.01//Transitional//EN\">\n\
             <html>\n\
             <head>\n\
             <meta name=\"generator\" content=\"{HTML_GENERATOR}\">\n\
             <style type=\"text/css\">\n\
             body {{ font: 10pt courier; white-space: nowrap }}\n",
        )?;
        write_css_classes(&mut self.w, &self.overrides)?;
        let body_bg = self.overrides.css_body.as_deref().unwrap_or(HTML_BODY_BG);
        write!(
            self.w,
            "</style>\n\
             <title>Log colorisation generated by {HTML_GENERATOR}</title>\n\
             </head>\n\
             <body bgcolor=\"{body_bg}\">\n\n",
        )?;
        self.preamble_written = true;
        Ok(())
    }

    fn html_encode<'a>(&self, text: &'a str) -> std::borrow::Cow<'a, str> {
        // C source at ccze.c:387-428 only encodes 3 entities (`<`, `>`, `&`).
        // Matching that exactly — quotes pass through verbatim, which appears
        // in fixtures like `"X=a HTTP/1.0"`.
        if !text.bytes().any(|b| matches!(b, b'<' | b'>' | b'&')) {
            return std::borrow::Cow::Borrowed(text);
        }
        let mut out = String::with_capacity(text.len() + 8);
        for ch in text.chars() {
            match ch {
                '<' => out.push_str("&lt;"),
                '>' => out.push_str("&gt;"),
                '&' => out.push_str("&amp;"),
                _ => out.push(ch),
            }
        }
        std::borrow::Cow::Owned(out)
    }
}

impl<W: Write> OutputSink for HtmlSink<W> {
    fn emit(&mut self, color: Color, text: &str) -> io::Result<()> {
        let kw = color.keyword();
        let encoded = self.html_encode(text);
        write!(self.w, "<font class=\"ccze_{kw}\">{encoded}</font>")
    }

    fn space(&mut self) -> io::Result<()> {
        // Same `<font ...>` wrapper as emit, but the `&nbsp;` body is *not*
        // HTML-encoded — it's a raw entity. Matches `ccze_addstr_internal
        // (DEFAULT, "&nbsp;", 0)` at ccze.c:513-514.
        self.w
            .write_all(b"<font class=\"ccze_default\">&nbsp;</font>")
    }

    fn newline(&mut self) -> io::Result<()> {
        self.w.write_all(b"<br>\n")
    }

    fn finish(&mut self) -> io::Result<()> {
        // Mirrors `printf("\n</body>\n</html>\n")` at ccze.c:531.
        self.w.write_all(b"\n</body>\n</html>\n")?;
        self.w.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_emits_tag_pair() {
        let mut buf = Vec::new();
        let mut sink = DebugSink::new(&mut buf);
        sink.emit(Color::Date, "Sep 14").unwrap();
        sink.space().unwrap();
        sink.emit(Color::Host, "iluvatar").unwrap();
        sink.newline().unwrap();
        assert_eq!(
            std::str::from_utf8(&buf).unwrap(),
            "<date>Sep 14</date><default> </default><host>iluvatar</host>\n"
        );
    }
}
