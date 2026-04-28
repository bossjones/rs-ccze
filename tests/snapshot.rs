//! Byte-exact snapshot tests against the C reference's `.ok` files.
//!
//! Each test runs the Rust `ccze` binary in debug mode with the same flags the
//! original C testsuite shell scripts used, pipes `<name>.in` to stdin, and
//! asserts that stdout matches `<name>.ok` byte-for-byte.
//!
//! - `bug_*` tests: input + expected output copied verbatim from the original
//!   C testsuite at `/Users/malcolm/dev/ccze/ccze/testsuite/`.
//! - `snap_*` tests: synthetic input we authored, expected output minted from
//!   the C reference binary running inside Docker (see
//!   `rust/scripts/generate-baseline.sh`).
//!
//! Snapshot failures dump a short prefix-diff so you can see where output
//! diverges. Tests for plugins that haven't been ported to Rust yet will be
//! red; they go green one-by-one through Phase 6.

use assert_cmd::Command;
use std::path::PathBuf;

fn testdata_dir() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("testdata");
    p
}

fn run_snapshot(name: &str, plugins: &[&str]) {
    run_snapshot_mode(name, plugins, "-d", &format!("{name}.in"), &format!("{name}.ok"));
}

/// Run a snapshot test with a specific output-mode flag.
///
/// `mode_flag` is the single ccze flag that selects the renderer (`-d`, `-A`,
/// `-h`). `input_file` and `expected_file` are file names within `testdata/`.
fn run_snapshot_mode(name: &str, plugins: &[&str], mode_flag: &str, input_file: &str, expected_file: &str) {
    let dir = testdata_dir();
    let input = std::fs::read(dir.join(input_file))
        .unwrap_or_else(|e| panic!("read {input_file}: {e}"));
    let expected = std::fs::read(dir.join(expected_file))
        .unwrap_or_else(|e| panic!("read {expected_file}: {e}"));

    let plugins_arg = plugins.join(",");
    let assert = Command::cargo_bin("ccze")
        .unwrap()
        .args([
            "-F",
            "/dev/null",
            mode_flag,
            "-o",
            "nolookups",
            "-p",
            &plugins_arg,
        ])
        .write_stdin(input)
        .assert()
        .success();

    let actual = &assert.get_output().stdout;
    if actual.as_slice() != expected.as_slice() {
        let actual_s = String::from_utf8_lossy(actual);
        let expected_s = String::from_utf8_lossy(&expected);
        let mismatch = first_diff(&actual_s, &expected_s);
        panic!(
            "snapshot mismatch for {name}\n  first diff at byte {pos}\n  expected: {exp:?}\n  actual:   {act:?}\n",
            pos = mismatch.byte,
            exp = mismatch.expected_window,
            act = mismatch.actual_window,
        );
    }
}

/// Run an ANSI snapshot test: pipe `bug-<base>.in` (the existing debug-mode
/// fixture) through `ccze -A` and assert byte-exact against
/// `snap-ansi-<base>.ok` (minted from the Docker reference).
fn run_ansi_snapshot(base: &str, plugins: &[&str]) {
    let name = format!("snap-ansi-{base}");
    run_snapshot_mode(
        &name,
        plugins,
        "-A",
        &format!("bug-{base}.in"),
        &format!("{name}.ok"),
    );
}

/// Run an HTML snapshot test — `bug-<base>.in` through `ccze -h`, byte-exact
/// against `snap-html-<base>.ok`.
fn run_html_snapshot(base: &str, plugins: &[&str]) {
    let name = format!("snap-html-{base}");
    run_snapshot_mode(
        &name,
        plugins,
        "-h",
        &format!("bug-{base}.in"),
        &format!("{name}.ok"),
    );
}

struct DiffWindow {
    byte: usize,
    expected_window: String,
    actual_window: String,
}

fn first_diff(actual: &str, expected: &str) -> DiffWindow {
    let abytes = actual.as_bytes();
    let ebytes = expected.as_bytes();
    let n = abytes.len().min(ebytes.len());
    let mut pos = 0;
    while pos < n && abytes[pos] == ebytes[pos] {
        pos += 1;
    }
    let start = pos.saturating_sub(40);
    let end_a = (pos + 80).min(abytes.len());
    let end_e = (pos + 80).min(ebytes.len());
    DiffWindow {
        byte: pos,
        actual_window: String::from_utf8_lossy(&abytes[start..end_a]).into_owned(),
        expected_window: String::from_utf8_lossy(&ebytes[start..end_e]).into_owned(),
    }
}

#[test]
fn version_flag_prints_version() {
    let out = Command::cargo_bin("ccze")
        .unwrap()
        .arg("-V")
        .output()
        .expect("run ccze -V");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(stdout, format!("ccze {}\n", env!("CARGO_PKG_VERSION")));
}

#[test]
fn bug_sysrepeat() {
    run_snapshot("bug-sysrepeat", &["syslog"]);
}

#[test]
fn bug_wnum() {
    run_snapshot("bug-wnum", &["syslog"]);
}

#[test]
fn bug_procmailsubj() {
    run_snapshot("bug-procmailsubj", &["syslog", "procmail"]);
}

#[test]
fn bug_procmailsubj2() {
    run_snapshot("bug-procmailsubj2", &["syslog", "procmail"]);
}

#[test]
fn bug_httpd() {
    run_snapshot("bug-httpd", &["httpd"]);
}

#[test]
fn bug_dpkg() {
    run_snapshot("bug-dpkg", &["dpkg"]);
}

// ---- snap_* tests: synthetic baselines minted from the C reference ----
// These will be RED until each plugin is ported in Phase 6. Partial-type
// plugins (apm, fetchmail, postfix, ulogd) need syslog ahead of them in the
// plugin list because they only run on the residue of a Full plugin match.

#[test]
fn snap_apm() {
    run_snapshot("snap-apm", &["syslog", "apm"]);
}

#[test]
fn snap_distcc() {
    run_snapshot("snap-distcc", &["distcc"]);
}

#[test]
fn snap_exim() {
    run_snapshot("snap-exim", &["exim"]);
}

#[test]
fn snap_fetchmail() {
    run_snapshot("snap-fetchmail", &["syslog", "fetchmail"]);
}

#[test]
fn snap_ftpstats() {
    run_snapshot("snap-ftpstats", &["ftpstats"]);
}

#[test]
fn snap_icecast() {
    run_snapshot("snap-icecast", &["icecast"]);
}

#[test]
fn snap_oops() {
    run_snapshot("snap-oops", &["oops"]);
}

#[test]
fn snap_php() {
    run_snapshot("snap-php", &["php"]);
}

#[test]
fn snap_postfix() {
    run_snapshot("snap-postfix", &["syslog", "postfix"]);
}

#[test]
fn snap_proftpd() {
    run_snapshot("snap-proftpd", &["proftpd"]);
}

#[test]
fn snap_squid() {
    run_snapshot("snap-squid", &["squid"]);
}

#[test]
fn snap_sulog() {
    run_snapshot("snap-sulog", &["sulog"]);
}

#[test]
fn snap_super() {
    run_snapshot("snap-super", &["super"]);
}

#[test]
fn snap_ulogd() {
    run_snapshot("snap-ulogd", &["syslog", "ulogd"]);
}

#[test]
fn snap_vsftpd() {
    run_snapshot("snap-vsftpd", &["vsftpd"]);
}

#[test]
fn snap_xferlog() {
    run_snapshot("snap-xferlog", &["xferlog"]);
}

// ---- ANSI snapshot tests (Phase 7) ---------------------------------------
// These use the existing `bug-<base>.in` fixtures piped through `ccze -A` and
// compare against `snap-ansi-<base>.ok` minted from the Docker C reference.

#[test]
fn snap_ansi_sysrepeat() {
    run_ansi_snapshot("sysrepeat", &["syslog"]);
}

#[test]
fn snap_ansi_httpd() {
    run_ansi_snapshot("httpd", &["httpd"]);
}

// ---- HTML snapshot tests (Phase 8) ---------------------------------------

#[test]
fn snap_html_sysrepeat() {
    run_html_snapshot("sysrepeat", &["syslog"]);
}

// ---- rcfile-driven test (Phase 9) ----------------------------------------

#[test]
fn rcfile_overrides_date_color_in_ansi_output() {
    // Default: `date` is bold cyan, which renders as ESC[1m...ESC[36m.
    // With an rcfile that says `date red`, it should render as plain red
    // (ESC[31m, no bold).
    let tmp = std::env::temp_dir().join("ccze-rcfile-test.rc");
    std::fs::write(&tmp, "date red\n").unwrap();

    let input = "Sep 14 11:00:00 host syslog: test\n";
    let out = Command::cargo_bin("ccze")
        .unwrap()
        .args([
            "-A",
            "-F",
            tmp.to_str().unwrap(),
            "-o",
            "nolookups",
            "-p",
            "syslog",
        ])
        .write_stdin(input)
        .output()
        .expect("run ccze with rcfile");
    assert!(out.status.success(), "ccze exited non-zero: {out:?}");

    let stdout = String::from_utf8_lossy(&out.stdout);
    // The date emit. Should NOT contain ESC[1m before the foreground code.
    let date_idx = stdout.find("Sep 14 11:00:00").expect("date span not found");
    let prefix = &stdout[..date_idx];

    // The byte sequence preceding the text — should be the new attr block.
    // We expect: ESC[22m + ESC[31m + Sep 14… (no ESC[1m, no ESC[36m).
    assert!(prefix.ends_with("\x1b[22m\x1b[31m"),
        "expected date span to be preceded by ESC[22m ESC[31m, got: {prefix:?}");
    assert!(!prefix.contains("\x1b[1m"),
        "rcfile said `date red` (not bold) but bold escape was emitted: {prefix:?}");

    let _ = std::fs::remove_file(&tmp);
}
