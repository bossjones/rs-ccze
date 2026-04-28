//! Color enum + tag-name table.
//!
//! The keyword strings are the source of truth for debug-mode output. They come
//! verbatim from `ccze_color_keyword_map` in the C source
//! (src/ccze-color.c:110-197). Do not invent new keywords — every name here is
//! one a user can write in `~/.cczerc` and one that appears as `<tag>` in
//! debug-mode output.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    // Settable (appear as <tag> in debug output)
    Default,
    Unknown,
    Date,
    Host,
    Mac,
    Pid,
    PidB,
    HttpGet,
    HttpPost,
    HttpHead,
    HttpPut,
    HttpConnect,
    HttpTrace,
    HttpCodes,
    GetTime,
    GetSize,
    Debug,
    Error,
    Warning,
    Bad,
    Good,
    System,
    Proc,
    Dir,
    File,
    Prot,
    Service,
    Email,
    Size,
    Version,
    Address,
    Uri,
    ProxyMiss,
    ProxyParent,
    ProxyDirect,
    ProxyHit,
    ProxyDenied,
    Ident,
    ProxyRefresh,
    ProxySwapfail,
    CType,
    ProxyCreate,
    ProxySwapin,
    ProxySwapout,
    ProxyRelease,
    SwapNum,
    User,
    Numbers,
    Subject,
    Signal,
    Incoming,
    Outgoing,
    UniqN,
    Repeat,
    Field,
    Chain,
    Percentage,
    FtpCodes,
    Keyword,
    PkgStatus,
    Pkg,

    // Hidden static colors (config keywords only; never emitted as debug tags)
    StaticBlack,
    StaticRed,
    StaticGreen,
    StaticYellow,
    StaticBlue,
    StaticCyan,
    StaticMagenta,
    StaticWhite,
    StaticBoldBlack,
    StaticBoldRed,
    StaticBoldGreen,
    StaticBoldYellow,
    StaticBoldBlue,
    StaticBoldCyan,
    StaticBoldMagenta,
    StaticBoldWhite,
}

impl Color {
    /// Keyword string used in debug-mode tags and config files.
    pub fn keyword(self) -> &'static str {
        use Color::*;
        match self {
            Default => "default",
            Unknown => "unknown",
            Date => "date",
            Host => "host",
            Mac => "mac",
            Pid => "pid",
            PidB => "pid-sqbr",
            HttpGet => "get",
            HttpPost => "post",
            HttpHead => "head",
            HttpPut => "put",
            HttpConnect => "connect",
            HttpTrace => "trace",
            HttpCodes => "httpcodes",
            GetTime => "gettime",
            GetSize => "getsize",
            Debug => "debug",
            Error => "error",
            Warning => "warning",
            Bad => "bad",
            Good => "good",
            System => "system",
            Proc => "process",
            Dir => "dir",
            File => "file",
            Prot => "prot",
            Service => "service",
            Email => "email",
            Size => "size",
            Version => "version",
            Address => "address",
            Uri => "uri",
            ProxyMiss => "miss",
            ProxyParent => "parent",
            ProxyDirect => "direct",
            ProxyHit => "hit",
            ProxyDenied => "deny",
            Ident => "ident",
            ProxyRefresh => "refresh",
            ProxySwapfail => "swapfail",
            CType => "ctype",
            ProxyCreate => "create",
            ProxySwapin => "swapin",
            ProxySwapout => "swapout",
            ProxyRelease => "release",
            SwapNum => "swapnum",
            User => "user",
            Numbers => "numbers",
            Subject => "subject",
            Signal => "signal",
            Incoming => "incoming",
            Outgoing => "outgoing",
            UniqN => "uniqn",
            Repeat => "repeat",
            Field => "field",
            Chain => "chain",
            Percentage => "percentage",
            FtpCodes => "ftpcodes",
            Keyword => "keyword",
            PkgStatus => "pkgstatus",
            Pkg => "pkg",
            StaticBlack => "black",
            StaticRed => "red",
            StaticGreen => "green",
            StaticYellow => "yellow",
            StaticBlue => "blue",
            StaticCyan => "cyan",
            StaticMagenta => "magenta",
            StaticWhite => "white",
            StaticBoldBlack => "bold_black",
            StaticBoldRed => "bold_red",
            StaticBoldGreen => "bold_green",
            StaticBoldYellow => "bold_yellow",
            StaticBoldBlue => "bold_blue",
            StaticBoldCyan => "bold_cyan",
            StaticBoldMagenta => "bold_magenta",
            StaticBoldWhite => "bold_white",
        }
    }

    pub fn from_keyword(s: &str) -> Option<Color> {
        use Color::*;
        Some(match s {
            "default" => Default,
            "unknown" => Unknown,
            "date" => Date,
            "host" => Host,
            "mac" => Mac,
            "pid" => Pid,
            "pid-sqbr" => PidB,
            "get" => HttpGet,
            "post" => HttpPost,
            "head" => HttpHead,
            "put" => HttpPut,
            "connect" => HttpConnect,
            "trace" => HttpTrace,
            "httpcodes" => HttpCodes,
            "gettime" => GetTime,
            "getsize" => GetSize,
            "debug" => Debug,
            "error" => Error,
            "warning" => Warning,
            "bad" => Bad,
            "good" => Good,
            "system" => System,
            "process" => Proc,
            "dir" => Dir,
            "file" => File,
            "prot" => Prot,
            "service" => Service,
            "email" => Email,
            "size" => Size,
            "version" => Version,
            "address" => Address,
            "uri" => Uri,
            "miss" => ProxyMiss,
            "parent" => ProxyParent,
            "direct" => ProxyDirect,
            "hit" => ProxyHit,
            "deny" => ProxyDenied,
            "ident" => Ident,
            "refresh" => ProxyRefresh,
            "swapfail" => ProxySwapfail,
            "ctype" => CType,
            "create" => ProxyCreate,
            "swapin" => ProxySwapin,
            "swapout" => ProxySwapout,
            "release" => ProxyRelease,
            "swapnum" => SwapNum,
            "user" => User,
            "numbers" => Numbers,
            "subject" => Subject,
            "signal" => Signal,
            "incoming" => Incoming,
            "outgoing" => Outgoing,
            "uniqn" => UniqN,
            "repeat" => Repeat,
            "field" => Field,
            "chain" => Chain,
            "percentage" => Percentage,
            "ftpcodes" => FtpCodes,
            "keyword" => Keyword,
            "pkgstatus" => PkgStatus,
            "pkg" => Pkg,
            "black" => StaticBlack,
            "red" => StaticRed,
            "green" => StaticGreen,
            "yellow" => StaticYellow,
            "blue" => StaticBlue,
            "cyan" => StaticCyan,
            "magenta" => StaticMagenta,
            "white" => StaticWhite,
            "bold_black" => StaticBoldBlack,
            "bold_red" => StaticBoldRed,
            "bold_green" => StaticBoldGreen,
            "bold_yellow" => StaticBoldYellow,
            "bold_blue" => StaticBoldBlue,
            "bold_cyan" => StaticBoldCyan,
            "bold_magenta" => StaticBoldMagenta,
            "bold_white" => StaticBoldWhite,
            _ => return None,
        })
    }
}

/// ccze's internal colour indices (0-7) for the 8 base colours. **Cyan and
/// magenta are deliberately swapped** vs the conventional ANSI ordering — the C
/// source comment at `ccze.c:50` calls this out, and the rcfile keyword "cyan"
/// produces ANSI code 36 because of it. We must preserve the swap to stay
/// byte-exact with the C reference.
pub mod ansi_idx {
    pub const BLACK: u8 = 0;
    pub const RED: u8 = 1;
    pub const GREEN: u8 = 2;
    pub const YELLOW: u8 = 3;
    pub const BLUE: u8 = 4;
    pub const CYAN: u8 = 5; // → ANSI fg 36
    pub const MAGENTA: u8 = 6; // → ANSI fg 35
    pub const WHITE: u8 = 7;
}

/// Map a ccze base-colour index (0-7) to the SGR foreground code. Add 10 for
/// background. Values from `ccze_raw_ansi_color[]` at `ccze.c:51`.
pub const ANSI_FG_CODE: [u8; 8] = [30, 31, 32, 33, 34, 36, 35, 37];

/// CSS colour names for normal-weight text, indexed by ccze base-colour index.
/// Verbatim from `ccze_csscolor_normal_map` at `ccze-color.c:69-71`.
pub const CSS_NORMAL: [&str; 8] = [
    "black",
    "darkred",
    "#00C000",
    "brown",
    "blue",
    "darkcyan",
    "darkmagenta",
    "grey",
];
/// CSS colour names for bold text. From `ccze_csscolor_bold_map` at `ccze-color.c:72-74`.
pub const CSS_BOLD: [&str; 8] = [
    "black",
    "red",
    "lime",
    "yellow",
    "slateblue",
    "cyan",
    "magenta",
    "white",
];

/// Default body background for HTML output (`ccze_cssbody` in `ccze-color.c:75`).
pub const HTML_BODY_BG: &str = "#404040";

/// Iteration order for emitting the embedded `<style>` block — matches the C
/// `for (cidx = CCZE_COLOR_DATE; cidx < CCZE_COLOR_LAST; cidx++)` loop at
/// `ccze-color.c:297`. The order is the C `ccze_color_t` enum's numeric
/// ordering (DATE=0 first, then HOST, PROC, …, then the 16 STATIC_* entries).
pub const CSS_ITER_ORDER: [Color; 77] = {
    use Color::*;
    [
        Date,
        Host,
        Proc,
        Pid,
        PidB,
        Default,
        Email,
        Subject,
        Dir,
        File,
        Size,
        User,
        HttpCodes,
        GetSize,
        HttpGet,
        HttpPost,
        HttpHead,
        HttpPut,
        HttpConnect,
        HttpTrace,
        Unknown,
        GetTime,
        Uri,
        Ident,
        CType,
        Error,
        ProxyMiss,
        ProxyHit,
        ProxyDenied,
        ProxyRefresh,
        ProxySwapfail,
        Debug,
        Warning,
        ProxyDirect,
        ProxyParent,
        SwapNum,
        ProxyCreate,
        ProxySwapin,
        ProxySwapout,
        ProxyRelease,
        Mac,
        Version,
        Address,
        Numbers,
        Signal,
        Service,
        Prot,
        Bad,
        Good,
        System,
        Incoming,
        Outgoing,
        UniqN,
        Repeat,
        Field,
        Chain,
        Percentage,
        FtpCodes,
        Keyword,
        PkgStatus,
        Pkg,
        StaticBlack,
        StaticRed,
        StaticGreen,
        StaticYellow,
        StaticBlue,
        StaticCyan,
        StaticMagenta,
        StaticWhite,
        StaticBoldBlack,
        StaticBoldRed,
        StaticBoldGreen,
        StaticBoldYellow,
        StaticBoldBlue,
        StaticBoldCyan,
        StaticBoldMagenta,
        StaticBoldWhite,
    ]
};

/// ANSI rendering attributes for a single `Color`. Holds the same information
/// the C `ccze_color_table[]` packs into a single int (low byte = fg, high byte
/// = bg, 0x1000-0x8000 = attribute bits).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnsiAttr {
    pub fg: u8,
    pub bg: Option<u8>,
    pub bold: bool,
    pub underline: bool,
    pub reverse: bool,
    pub blink: bool,
}

impl AnsiAttr {
    pub const fn fg(idx: u8) -> Self {
        Self {
            fg: idx,
            bg: None,
            bold: false,
            underline: false,
            reverse: false,
            blink: false,
        }
    }
    pub const fn bold(idx: u8) -> Self {
        Self {
            fg: idx,
            bg: None,
            bold: true,
            underline: false,
            reverse: false,
            blink: false,
        }
    }
}

impl Color {
    /// Default attributes for HTML CSS / curses colouring — mirrors
    /// `ccze_color_init` at `ccze-color.c:566-654`. Identical to
    /// `default_ansi_attr` for every variant *except* `SwapNum`, which the C
    /// source intentionally renders differently between RAW_ANSI mode (a flat
    /// cyan) and curses/HTML (`COLOR_PAIR(4 + 7*8)` — blue on white).
    pub fn default_html_attr(self) -> AnsiAttr {
        use Color::SwapNum;
        use ansi_idx::*;
        match self {
            SwapNum => AnsiAttr {
                fg: BLUE,
                bg: Some(WHITE),
                bold: false,
                underline: false,
                reverse: false,
                blink: false,
            },
            other => other.default_ansi_attr(),
        }
    }

    /// Default ANSI rendering attributes — mirrors `ccze_color_init_raw_ansi`
    /// at `ccze-color.c:481-564`. The rcfile parser (Phase 9) overrides these
    /// at startup; until then this is the static colour table.
    pub fn default_ansi_attr(self) -> AnsiAttr {
        use Color::*;
        use ansi_idx::*;
        match self {
            Date => AnsiAttr::bold(CYAN),
            Host => AnsiAttr::bold(BLUE),
            Proc => AnsiAttr::fg(GREEN),
            Pid => AnsiAttr::bold(WHITE),
            PidB => AnsiAttr::bold(GREEN),
            Default => AnsiAttr::fg(CYAN),
            Email => AnsiAttr::bold(GREEN),
            Subject => AnsiAttr::fg(MAGENTA),
            Dir => AnsiAttr::bold(CYAN),
            File => AnsiAttr::bold(CYAN),
            Size => AnsiAttr::bold(WHITE),
            User => AnsiAttr::bold(YELLOW),
            HttpCodes => AnsiAttr::bold(WHITE),
            GetSize => AnsiAttr::fg(MAGENTA),
            HttpGet => AnsiAttr::fg(GREEN),
            HttpPost => AnsiAttr::bold(GREEN),
            HttpHead => AnsiAttr::fg(GREEN),
            HttpPut => AnsiAttr::bold(GREEN),
            HttpConnect => AnsiAttr::fg(GREEN),
            HttpTrace => AnsiAttr::fg(GREEN),
            Unknown => AnsiAttr::fg(CYAN),
            GetTime => AnsiAttr::bold(MAGENTA),
            Uri => AnsiAttr::bold(GREEN),
            Ident => AnsiAttr::bold(WHITE),
            CType => AnsiAttr::fg(WHITE),
            Error => AnsiAttr::bold(RED),
            ProxyMiss => AnsiAttr::fg(RED),
            ProxyHit => AnsiAttr::bold(YELLOW),
            ProxyDenied => AnsiAttr::bold(RED),
            ProxyRefresh => AnsiAttr::bold(WHITE),
            ProxySwapfail => AnsiAttr::bold(WHITE),
            Debug => AnsiAttr::fg(WHITE),
            Warning => AnsiAttr::fg(RED),
            ProxyDirect => AnsiAttr::bold(WHITE),
            ProxyParent => AnsiAttr::bold(YELLOW),
            SwapNum => AnsiAttr::fg(CYAN),
            ProxyCreate => AnsiAttr::bold(WHITE),
            ProxySwapin => AnsiAttr::bold(WHITE),
            ProxySwapout => AnsiAttr::bold(WHITE),
            ProxyRelease => AnsiAttr::bold(WHITE),
            Mac => AnsiAttr::bold(WHITE),
            Version => AnsiAttr::bold(WHITE),
            Address => AnsiAttr::bold(WHITE),
            Numbers => AnsiAttr::fg(WHITE),
            Signal => AnsiAttr::bold(YELLOW),
            Service => AnsiAttr::bold(MAGENTA),
            Prot => AnsiAttr::fg(MAGENTA),
            Bad => AnsiAttr::bold(YELLOW),
            Good => AnsiAttr::bold(GREEN),
            System => AnsiAttr::bold(CYAN),
            Incoming => AnsiAttr::bold(WHITE),
            Outgoing => AnsiAttr::fg(WHITE),
            UniqN => AnsiAttr::bold(WHITE),
            Repeat => AnsiAttr::fg(WHITE),
            Field => AnsiAttr::fg(GREEN),
            Chain => AnsiAttr::fg(CYAN),
            Percentage => AnsiAttr::bold(YELLOW),
            FtpCodes => AnsiAttr::fg(CYAN),
            Keyword => AnsiAttr::bold(YELLOW),
            PkgStatus => AnsiAttr::fg(GREEN),
            Pkg => AnsiAttr::bold(RED),

            StaticBlack => AnsiAttr::fg(BLACK),
            StaticRed => AnsiAttr::fg(RED),
            StaticGreen => AnsiAttr::fg(GREEN),
            StaticYellow => AnsiAttr::fg(YELLOW),
            StaticBlue => AnsiAttr::fg(BLUE),
            StaticCyan => AnsiAttr::fg(CYAN),
            StaticMagenta => AnsiAttr::fg(MAGENTA),
            StaticWhite => AnsiAttr::fg(WHITE),
            StaticBoldBlack => AnsiAttr::bold(BLACK),
            StaticBoldRed => AnsiAttr::bold(RED),
            StaticBoldGreen => AnsiAttr::bold(GREEN),
            StaticBoldYellow => AnsiAttr::bold(YELLOW),
            StaticBoldBlue => AnsiAttr::bold(BLUE),
            StaticBoldCyan => AnsiAttr::bold(CYAN),
            StaticBoldMagenta => AnsiAttr::bold(MAGENTA),
            StaticBoldWhite => AnsiAttr::bold(WHITE),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyword_round_trip() {
        for c in [
            Color::Default,
            Color::Date,
            Color::Host,
            Color::Proc,
            Color::Pid,
            Color::PidB,
            Color::Bad,
            Color::Good,
            Color::Repeat,
            Color::Numbers,
            Color::Pkg,
            Color::PkgStatus,
            Color::HttpGet,
            Color::HttpPost,
            Color::Subject,
            Color::StaticBoldWhite,
        ] {
            assert_eq!(
                Color::from_keyword(c.keyword()),
                Some(c),
                "round-trip failed for {c:?}"
            );
        }
    }
}
