//! Fallback word colorizer.
//!
//! Ports `ccze-wordcolor.c` (regexes + bad/good/error/system word lists).
//! Patterns and word lists are taken verbatim from the C source —
//! see `ccze-wordcolor.c:36-57` for the lists and `:261-303` for the regexes.
//!
//! Service / protocol / user lookups (`getservbyname` etc.) are honoured only
//! when `slookup` is true; tests run with `-o nolookups` so they are off.

use crate::color::Color;
use crate::sink::OutputSink;
use once_cell::sync::Lazy;
use regex::Regex;
use std::io;

static RE_PRE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^([`'".,!?:;(\[{<]+)([^`'".,!?:;(\[{<]\S*)$"#).unwrap()
});
static RE_POST: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^(\S*[^`'".,!?:;)\]}>])([`'".,!?:;)\]}>]+)$"#).unwrap()
});
static RE_HOST: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"^(((\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})|(([a-z0-9-_]+\.)+[a-z]{2,3})|(localhost)|(\w*::\w+)+)(:\d{1,5})?)$",
    )
    .unwrap()
});
static RE_HOSTIP: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"^(((\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})|(([a-z0-9-_\.]+)+)|(localhost)|(\w*::\w+)+)(:\d{1,5})?)\[",
    )
    .unwrap()
});
static RE_MAC: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^([0-9a-f]{2}:){5}[0-9a-f]{2}$").unwrap());
static RE_EMAIL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-z0-9-_=\+]+@([a-z0-9-_\.]+)+(\.[a-z]{2,4})+").unwrap()
});
static RE_EMAIL2: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(\.[a-z]{2,4})+$").unwrap());
static RE_URI: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\w{2,}:/\/(\S+/?)+$").unwrap());
static RE_SIZE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\d+(\.\d+)?[kmgt]i?b?(ytes?)?").unwrap());
static RE_VER: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^v?(\d+\.){1}((\d|[a-z])+\.)*(\d|[a-z])+$").unwrap());
static RE_TIME: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\d{1,2}:\d{1,2}(:\d{1,2})?").unwrap());
static RE_ADDR: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^0x(\d|[a-f])+$").unwrap());
static RE_NUM: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[+-]?\d+$").unwrap());
static RE_SIG: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"^sig(hup|int|quit|ill|abrt|fpe|kill|segv|pipe|alrm|term|usr1|usr2|chld|cont|stop|tstp|tin|tout|bus|poll|prof|sys|trap|urg|vtalrm|xcpu|xfsz|iot|emt|stkflt|io|cld|pwr|info|lost|winch|unused)",
    )
    .unwrap()
});
static RE_MSGID: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-z0-9-_\.\$=\+]+@([a-z0-9-_\.]+)+(\.?[a-z]+)+").unwrap()
});

const WORDS_BAD: &[&str] = &[
    "warn", "restart", "exit", "stop", "end", "shutting", "down", "close",
    "unreach", "can't", "cannot", "skip", "deny", "disable", "ignored", "miss",
    "oops", "not", "backdoor", "blocking", "ignoring", "unable", "readonly",
    "offline", "terminate", "empty", "virus",
];
const WORDS_GOOD: &[&str] = &[
    "activ", "start", "ready", "online", "load", "ok", "register", "detected",
    "configured", "enable", "listen", "open", "complete", "attempt", "done",
    "check", "listen", "connect", "finish", "clean",
];
const WORDS_ERROR: &[&str] = &["error", "crit", "invalid", "fail", "false", "alarm", "fatal"];
const WORDS_SYSTEM: &[&str] = &[
    "ext2-fs", "reiserfs", "vfs", "iso", "isofs", "cslip", "ppp", "bsd",
    "linux", "tcp/ip", "mtrr", "pci", "isa", "scsi", "ide", "atapi", "bios",
    "cpu", "fpu", "discharging", "resume",
];

/// Top-level entry. Mirrors `ccze_wordcolor_process`.
pub fn process(
    msg: &str,
    sink: &mut dyn OutputSink,
    wcol: bool,
    slookup: bool,
) -> io::Result<()> {
    if msg.is_empty() {
        return Ok(());
    }
    if !wcol {
        sink.emit(Color::Default, msg)?;
        return Ok(());
    }
    if (msg.contains("last message repeated") && msg.contains("times"))
        || msg.contains("-- MARK --")
    {
        sink.emit(Color::Repeat, msg)?;
        return Ok(());
    }

    // Split on a literal space, NOT on whitespace classes — matches the C
    // `ccze_strbrk(msg, ' ')` loop, which produces empty tokens for consecutive
    // spaces. After every word (including the last one), a space tag is
    // emitted, which is why every wordcolor-coloured line ends with
    // `<default> </default>`.
    let words: Vec<&str> = msg.split(' ').collect();
    if words.is_empty() {
        return Ok(());
    }
    for word in words {
        process_one(word, sink, slookup)?;
        sink.space()?;
    }
    Ok(())
}

/// Mirrors `ccze_wordcolor_process_one`.
pub fn process_one(
    word: &str,
    sink: &mut dyn OutputSink,
    slookup: bool,
) -> io::Result<()> {
    // 1. Strip leading/trailing punctuation.
    let (pre, body_after_pre): (Option<&str>, &str) = match RE_PRE.captures(word) {
        Some(caps) => (Some(caps.get(1).unwrap().as_str()), caps.get(2).unwrap().as_str()),
        None => (None, word),
    };
    let (body, post): (&str, Option<&str>) = match RE_POST.captures(body_after_pre) {
        Some(caps) => (caps.get(1).unwrap().as_str(), Some(caps.get(2).unwrap().as_str())),
        None => (body_after_pre, None),
    };

    let lword = body.to_ascii_lowercase();

    // 2. Try the regex / heuristic cascade. The first match wins.
    //    `hostip` is a special case — it emits its own substring tags rather
    //    than colouring the whole word with one colour, so it sets `printed`.
    let mut printed = false;
    let mut col = Color::Default;

    if RE_HOST.is_match(&lword) {
        col = Color::Host;
    } else if RE_MAC.is_match(&lword) {
        col = Color::Mac;
    } else if lword.starts_with('/') {
        col = Color::Dir;
    } else if RE_EMAIL.is_match(&lword) && RE_EMAIL2.is_match(&lword) {
        col = Color::Email;
    } else if RE_MSGID.is_match(&lword) {
        col = Color::Email;
    } else if RE_URI.is_match(&lword) {
        col = Color::Uri;
    } else if RE_SIZE.is_match(&lword) {
        col = Color::Size;
    } else if RE_VER.is_match(&lword) {
        col = Color::Version;
    } else if RE_TIME.is_match(&lword) {
        col = Color::Date;
    } else if RE_ADDR.is_match(&lword) {
        col = Color::Address;
    } else if RE_NUM.is_match(&lword) {
        col = Color::Numbers;
    } else if RE_SIG.is_match(&lword) {
        col = Color::Signal;
    } else if RE_HOSTIP.is_match(&lword) {
        // host[ip] split. The C `host = word[..find('[')]` /
        // `ip = word[find('[') + 1 ..]` pair, with stripped `]` already gone
        // via RE_POST. The C source emits the four substrings *without* the
        // surrounding pre/post — when `printed=1` it skips the trailing
        // `addstr(DEFAULT, pre)`/`addstr(DEFAULT, post)` block, so leading or
        // trailing punctuation on a host[ip] token is silently dropped.
        if let Some(open_idx) = body.find('[') {
            let host = &body[..open_idx];
            let inner = &body[open_idx + 1..];
            sink.emit(Color::Host, host)?;
            sink.emit(Color::PidB, "[")?;
            sink.emit(Color::Host, inner)?;
            sink.emit(Color::PidB, "]")?;
            printed = true;
        }
    } else if slookup && service_known(&lword) {
        col = Color::Service;
    } else if slookup && protocol_known(&lword) {
        col = Color::Prot;
    } else if slookup && user_known(&lword) {
        col = Color::User;
    } else {
        // 3. Bad / good / error / system word substring lists. Match if the
        //    body STARTS WITH the listed word (matches the C
        //    `strstr(lword, w) == lword` check).
        for w in WORDS_BAD {
            if lword.starts_with(w) {
                col = Color::Bad;
            }
        }
        for w in WORDS_GOOD {
            if lword.starts_with(w) {
                col = Color::Good;
            }
        }
        for w in WORDS_ERROR {
            if lword.starts_with(w) {
                col = Color::Error;
            }
        }
        for w in WORDS_SYSTEM {
            if lword.starts_with(w) {
                col = Color::System;
            }
        }
    }

    if !printed {
        if let Some(p) = pre {
            sink.emit(Color::Default, p)?;
        }
        sink.emit(col, body)?;
        if let Some(p) = post {
            sink.emit(Color::Default, p)?;
        }
    }
    Ok(())
}

// Stubs — only consulted when `slookup` is true (i.e. `-o nolookups` was NOT
// passed). The tests pass nolookups, so these are unused in the snapshot
// suite. Will be wired up in Phase 11 if/when we want full parity with C.
fn service_known(_lword: &str) -> bool {
    false
}
fn protocol_known(_lword: &str) -> bool {
    false
}
fn user_known(_lword: &str) -> bool {
    false
}
