# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

Rust port of [ccze](https://github.com/cornet/ccze), a streaming log colorizer. Phases 0–11 done; 27/27 snapshot tests + 9/9 unit tests green. The C source tree the port is based on lives at `/Users/malcolm/dev/ccze/ccze/` (referenced from comments throughout the Rust source as `ccze.c:NNN`, `mod_<plugin>.c`, etc.).

## Build / test

```sh
cargo test                       # all tests (unit + snapshot)
cargo test --test snapshot       # snapshot suite only
cargo test --test snapshot snap_<plugin>   # one plugin
cargo build --release            # optimised binary at target/release/ccze
make help                        # self-documenting target list
```

`make demo` and `make list-plugins` are useful smoke checks. `make cssdump` prints the embedded HTML CSS palette.

## Snapshot harness — the spine of the project

Every plugin and renderer is verified by **byte-exact** snapshot tests in `tests/snapshot.rs`:

- `bug-*.{in,ok}` — fixtures copied verbatim from the original C testsuite.
- `snap-*.{in,ok}` — synthetic fixtures we authored; `.ok` files are minted from the **C reference binary** running inside Docker so they reflect canonical ccze behaviour, not our interpretation of it.
- `snap-ansi-<base>.ok` / `snap-html-<base>.ok` — same input piped through `-A` / `-h` instead of `-d`.

When changing plugin or renderer logic, **do not edit `.ok` files by hand**. If output legitimately needs to change, regenerate from the C reference:

```sh
./scripts/build-c-ref.sh                                # one-time, builds ccze:reference image
./scripts/generate-baseline.sh snap-<plugin> <plugin>   # one fixture
./scripts/generate-all-baselines.sh                     # all 16 synthetic plugins
```

The C source doesn't build cleanly on macOS — Docker is the only reliable path to the reference. `testdata/SOURCES.md` records provenance + the C tree's git HEAD when each `.ok` was minted.

When a snapshot fails, the harness prints `first diff at byte N` plus an 80-byte window of expected vs. actual.

## Adding a plugin (TDD micro-loop)

1. Author `testdata/snap-<plugin>.in` (5–15 synthetic lines).
2. `./scripts/generate-baseline.sh snap-<plugin> <plugin>` — mints `.ok` from the C reference.
3. Add `#[test] fn snap_<plugin>()` to `tests/snapshot.rs`.
4. Run it — RED.
5. Read `/Users/malcolm/dev/ccze/ccze/src/mod_<plugin>.c`; translate the regex + emit logic into `src/plugins/<plugin>.rs` implementing `Plugin`.
6. Register the new struct in `src/plugins/mod.rs`. **Order matters** — see Pipeline section below.
7. Run snapshot — GREEN.

## Architecture

### Pipeline (`src/plugin.rs`)
Mirrors the C `ccze_plugin_run` flow at `ccze.c:706-730`. Each input line is dispatched in three phases that map directly onto `HandleResult::{Consumed, Remainder, NoMatch}`:

1. **Full plugins** are tried in registration order, first match wins. A `Consumed` match means the plugin emitted its own newline and the dispatcher does nothing else. A `Remainder(rest)` match feeds `rest` into phase 2.
2. **Partial plugins** run on that remainder (currently `apm`, `fetchmail`, `postfix`, `ulogd`).
3. **Wordcolor fallback** colours whatever is left, then the sink emits a newline.

Plugin order in `src/plugins/mod.rs::all_plugins` matters: `syslog` goes first so it claims `Mon DD HH:MM:SS host …` lines before `procmail`'s permissive regex can swallow them. Partial plugins must be registered after a Full plugin that produces matching residue (e.g. `apm` is meaningless without `syslog` parsing the date/host preamble first).

### Sinks (`src/sink.rs`)
The `OutputSink` trait is the seam between "what plugins emit" (`(Color, &str)` events) and "how the user sees it":

- `DebugSink` — wraps each emit in `<keyword>...</keyword>`; this is what every snapshot test compares against.
- `AnsiSink` — emits raw SGR escapes; bit-to-SGR mapping is taken verbatim from `ccze.c:449-484`.
- `HtmlSink` — emits `<font class="...">` spans + an embedded CSS preamble.

`-m curses` is intentionally an alias for ANSI. The C source's curses mode owned the alternate screen via `initscr()`, which is bad UX for streaming logs.

### Colors and rcfiles
- `src/color.rs` — `Color` enum + tag-name strings (verbatim from `ccze_color_keyword_map` at `ccze-color.c:110-197`). The keyword strings are the source of truth for debug output **and** rcfile keys; do not invent new names.
- `src/config.rs` — parses `~/.cczerc` per `ccze-color.c:352-452`. Both whitespace and `=` separate key/value, which lets `main.rs` reuse the rcfile parser for `-c key=value` CLI overrides.

### Wordcolor fallback (`src/wordcolor.rs`)
Ports `ccze-wordcolor.c` — 13 regexes plus bad/good/error/system word lists. Service/protocol/user lookups (`getservbyname` etc.) are gated on the `slookup` flag; the test suite always runs with `-o nolookups` so they're off in snapshots.

## Source-of-truth conventions

- Comments throughout the Rust source point back to specific line ranges in the C tree (e.g. `// see ccze.c:706-720`). Keep these references when refactoring; they are the audit trail for "did we port this faithfully."
- Regexes in plugins should be byte-for-byte translations of the corresponding `mod_*.c` regex string. If you find yourself "improving" one, you are probably about to break a snapshot.
- Snapshot tests run with `-F /dev/null -d -o nolookups` to neutralise the user's rcfile and DNS/service lookups.

## Out of scope

`-a plugin=args` (no plugin uses argv), `-C` unix→date conversion, SIGHUP reload, dynamic plugin loading via `dlopen` (Rust port statically registers all plugins).

## External plan

The full porting strategy lives at `/Users/malcolm/.claude/plans/i-want-to-port-glistening-ladybug.md`.
