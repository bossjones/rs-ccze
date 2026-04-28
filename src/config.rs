//! `.cczerc` parser. Mirrors `ccze_color_parse` at `ccze-color.c:352-452`.
//!
//! Two kinds of lines:
//!   * **Plain colour overrides**: `<keyword> [attribute] <colour> [on_<bg>]`
//!     where `keyword` is one of the keyword names (`date`, `host`, …),
//!     `attribute` is an optional `bold|underline|reverse|blink`, `colour` is
//!     one of the eight base colours, and `on_<bg>` is an optional background.
//!   * **CSS overrides** (HTML mode only): `cssbody <css-color>`,
//!     `css<colour> <css-color>`, `cssbold<colour> <css-color>`.
//!
//! Comments start with `#` and run to end of line (only on plain-colour lines —
//! the `css*` keys can legitimately contain `#` for hex colours like
//! `cssbody #404040`).

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::color::{AnsiAttr, Color, ansi_idx};

#[derive(Debug, Clone, Default)]
pub struct ColorOverrides {
    /// Per-`Color` attribute overrides applied on top of `default_ansi_attr`
    /// / `default_html_attr`.
    pub attrs: HashMap<Color, AnsiAttr>,
    /// HTML body background colour (`<body bgcolor="…">`).
    pub css_body: Option<String>,
    /// Per-colour-index CSS overrides for normal-weight text. `None` falls back
    /// to the static `CSS_NORMAL` table.
    pub css_normal: [Option<String>; 8],
    /// Per-colour-index CSS overrides for bold-weight text.
    pub css_bold: [Option<String>; 8],
}

impl ColorOverrides {
    /// Read and parse an rcfile. Returns `Ok(empty)` when the file is missing,
    /// not a regular file (per the C `S_ISREG` check at `ccze-color.c:463`),
    /// or empty.
    pub fn parse_file(path: &Path) -> Self {
        match fs::metadata(path) {
            Ok(meta) if meta.is_file() => match fs::read_to_string(path) {
                Ok(s) => Self::parse_str(&s),
                Err(_) => Self::default(),
            },
            _ => Self::default(),
        }
    }

    pub fn parse_str(content: &str) -> Self {
        let mut overrides = Self::default();
        for raw_line in content.lines() {
            overrides.parse_line(raw_line);
        }
        overrides
    }

    fn parse_line(&mut self, raw_line: &str) {
        // Pull the first token to decide if this is a css-key line. Comments
        // get stripped from non-css lines (css lines may legitimately contain
        // `#` for hex colours).
        let trimmed = raw_line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return;
        }
        let is_css = trimmed.starts_with("css");
        let working = if is_css {
            raw_line
        } else {
            // Strip everything from the first `#` onwards.
            raw_line.split_once('#').map_or(raw_line, |(before, _)| before)
        };

        // Tokenise the way the C `strtok(line, " \t\n=")` does: whitespace and
        // `=` are all delimiters; runs of delimiters collapse.
        let mut tokens = working
            .split(|c: char| c.is_whitespace() || c == '=')
            .filter(|s| !s.is_empty());

        let keyword = match tokens.next() {
            Some(k) => k,
            None => return,
        };

        if is_css {
            // For css lines the rest is a single colour value (the C source
            // takes the second token as a complete value; `cssbody #404040`
            // → "#404040").
            let value = match tokens.next() {
                Some(v) => v,
                None => return,
            };
            self.apply_css(keyword, value);
            return;
        }

        let target_color = match Color::from_keyword(keyword) {
            Some(c) => c,
            None => return,
        };

        // Next token: attribute or colour.
        let next_tok = match tokens.next() {
            Some(t) => t,
            None => return,
        };
        let (attr_keyword, color_tok) = match next_tok {
            "bold" | "underline" | "reverse" | "blink" => (
                Some(next_tok),
                match tokens.next() {
                    Some(c) => c,
                    None => return,
                },
            ),
            other => (None, other),
        };

        let fg = match color_idx(color_tok) {
            Some(i) => i,
            None => return,
        };

        // Optional bg (must be `on_<colour>` or any other recognised keyword;
        // C accepts both forms via the same colorname_map).
        let bg = tokens.next().and_then(color_idx);

        let mut new_attr = AnsiAttr {
            fg,
            bg,
            bold: false,
            underline: false,
            reverse: false,
            blink: false,
        };
        match attr_keyword {
            Some("bold") => new_attr.bold = true,
            Some("underline") => new_attr.underline = true,
            Some("reverse") => new_attr.reverse = true,
            Some("blink") => new_attr.blink = true,
            _ => {}
        }

        self.attrs.insert(target_color, new_attr);
    }

    fn apply_css(&mut self, keyword: &str, value: &str) {
        if keyword == "cssbody" {
            self.css_body = Some(value.to_owned());
            return;
        }
        // Strip the `css` prefix; the rest is either `<colour>` or
        // `bold<colour>`. Mirrors `keyword += 3; if strstr(keyword, "bold") ...`
        // at ccze-color.c:439-444.
        let suffix = &keyword[3..];
        let (bold, color_name) = if let Some(rest) = suffix.strip_prefix("bold") {
            (true, rest)
        } else {
            (false, suffix)
        };
        if let Some(idx) = color_idx(color_name) {
            if bold {
                self.css_bold[idx as usize] = Some(value.to_owned());
            } else {
                self.css_normal[idx as usize] = Some(value.to_owned());
            }
        }
    }

    /// ANSI/HTML attribute for `c`, applying any rcfile override.
    pub fn ansi_attr(&self, c: Color) -> AnsiAttr {
        self.attrs.get(&c).copied().unwrap_or_else(|| c.default_ansi_attr())
    }

    pub fn html_attr(&self, c: Color) -> AnsiAttr {
        self.attrs.get(&c).copied().unwrap_or_else(|| c.default_html_attr())
    }
}

/// Map a colour name (`black`, `red`, …, with optional `on_` prefix) to its
/// 0-7 index. Mirrors `ccze_colorname_map` at `ccze-color.c:82-98`.
fn color_idx(name: &str) -> Option<u8> {
    use ansi_idx::*;
    let stripped = name.strip_prefix("on_").unwrap_or(name);
    Some(match stripped {
        "black" => BLACK,
        "red" => RED,
        "green" => GREEN,
        "yellow" => YELLOW,
        "blue" => BLUE,
        "cyan" => CYAN,
        "magenta" => MAGENTA,
        "white" => WHITE,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_color() {
        let o = ColorOverrides::parse_str("date red\n");
        let attr = o.ansi_attr(Color::Date);
        assert_eq!(attr.fg, ansi_idx::RED);
        assert!(!attr.bold);
        assert_eq!(attr.bg, None);
    }

    #[test]
    fn parses_attribute() {
        let o = ColorOverrides::parse_str("host bold blue\n");
        let attr = o.ansi_attr(Color::Host);
        assert_eq!(attr.fg, ansi_idx::BLUE);
        assert!(attr.bold);
    }

    #[test]
    fn parses_with_background() {
        let o = ColorOverrides::parse_str("error bold red on_yellow\n");
        let attr = o.ansi_attr(Color::Error);
        assert_eq!(attr.fg, ansi_idx::RED);
        assert!(attr.bold);
        assert_eq!(attr.bg, Some(ansi_idx::YELLOW));
    }

    #[test]
    fn ignores_comments_and_blank_lines() {
        let o = ColorOverrides::parse_str("\n# comment\n\ndate red # trailing\n");
        assert_eq!(o.ansi_attr(Color::Date).fg, ansi_idx::RED);
    }

    #[test]
    fn parses_cssbody_with_hash_value() {
        let o = ColorOverrides::parse_str("cssbody #123456\n");
        assert_eq!(o.css_body.as_deref(), Some("#123456"));
    }

    #[test]
    fn parses_css_color_overrides() {
        let o = ColorOverrides::parse_str("cssred crimson\ncssboldgreen springgreen\n");
        assert_eq!(o.css_normal[ansi_idx::RED as usize].as_deref(), Some("crimson"));
        assert_eq!(o.css_bold[ansi_idx::GREEN as usize].as_deref(), Some("springgreen"));
    }

    #[test]
    fn unknown_keyword_is_ignored() {
        let o = ColorOverrides::parse_str("nonsense red\ndate green\n");
        assert_eq!(o.ansi_attr(Color::Date).fg, ansi_idx::GREEN);
    }
}
