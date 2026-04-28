mod cli;
mod color;
mod config;
mod plugin;
mod plugins;
mod sink;
mod wordcolor;

use clap::Parser;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;
use std::process::ExitCode;

use crate::cli::{Cli, Mode};
use crate::config::ColorOverrides;
use crate::plugin::{Pipeline, PipelineOptions, PluginType};
use crate::sink::{AnsiSink, DebugSink, HtmlSink, OutputSink};

fn main() -> ExitCode {
    let cli = Cli::parse();

    if cli.version {
        println!("ccze {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }

    if cli.list_plugins {
        list_plugins();
        return ExitCode::SUCCESS;
    }

    if cli.cssdump {
        let overrides = load_overrides(&cli);
        let stdout = io::stdout().lock();
        let mut buf = std::io::BufWriter::new(stdout);
        if let Err(e) = sink::write_css_classes(&mut buf, &overrides) {
            eprintln!("ccze: {e}");
            return ExitCode::FAILURE;
        }
        return ExitCode::SUCCESS;
    }

    let mode = cli.resolved_mode();
    if let Err(e) = run(&cli, mode) {
        eprintln!("ccze: {e}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn run(cli: &Cli, mode: Mode) -> io::Result<()> {
    let opts = pipeline_options_from_cli(cli);
    let plugin_list = plugins::filter_by_name(plugins::all_plugins(), &cli.plugins);
    let pipeline = Pipeline::new(plugin_list);
    let overrides = load_overrides(cli);
    let process_opts = ProcessOpts {
        remove_facility: cli.remove_facility,
    };

    let stdin = io::stdin().lock();
    let stdout = io::stdout().lock();

    match mode {
        Mode::Debug => process(
            BufReader::new(stdin),
            DebugSink::new(stdout),
            &pipeline,
            opts,
            process_opts,
        ),
        // `-m curses` is an alias for ANSI — the C source's curses mode takes
        // over the screen via initscr() which is bad UX for streaming logs.
        // See the "TTY rendering" decision in the project plan.
        Mode::Curses | Mode::Ansi => {
            let sink = AnsiSink::new(stdout).with_overrides(overrides);
            process(BufReader::new(stdin), sink, &pipeline, opts, process_opts)
        }
        Mode::Html => {
            let sink = HtmlSink::with_overrides(stdout, overrides)?;
            process(BufReader::new(stdin), sink, &pipeline, opts, process_opts)
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct ProcessOpts {
    remove_facility: bool,
}

/// Resolve the rcfile per the C `ccze_main` block at ccze.c:577-599 and apply
/// any `-c key=color` overrides on top.
fn load_overrides(cli: &Cli) -> ColorOverrides {
    let mut base = if let Some(path) = &cli.rcfile {
        ColorOverrides::parse_file(std::path::Path::new(path))
    } else {
        let mut found = ColorOverrides::default();
        for c in standard_rcfile_candidates() {
            let parsed = ColorOverrides::parse_file(&c);
            let nonempty = !parsed.attrs.is_empty()
                || parsed.css_body.is_some()
                || parsed.css_normal.iter().any(|x| x.is_some())
                || parsed.css_bold.iter().any(|x| x.is_some());
            if nonempty {
                found = parsed;
                break;
            }
        }
        found
    };
    // Apply `-c key=value` overrides. Each one is parsed exactly like an
    // rcfile line, so we just feed `<key> <value>` (the rcfile tokenizer
    // accepts both whitespace and `=` as separators) into `parse_str` and
    // merge the result.
    for kv in &cli.color_overrides {
        let line = kv.replacen('=', " ", 1);
        let cli_overrides = ColorOverrides::parse_str(&line);
        merge_overrides(&mut base, cli_overrides);
    }
    base
}

fn merge_overrides(base: &mut ColorOverrides, extra: ColorOverrides) {
    base.attrs.extend(extra.attrs);
    if extra.css_body.is_some() {
        base.css_body = extra.css_body;
    }
    for (i, v) in extra.css_normal.into_iter().enumerate() {
        if v.is_some() {
            base.css_normal[i] = v;
        }
    }
    for (i, v) in extra.css_bold.into_iter().enumerate() {
        if v.is_some() {
            base.css_bold[i] = v;
        }
    }
}

/// Print Name/Type/Description for every registered plugin. Format mirrors
/// `ccze_plugin_list_fancy` at `src/ccze-plugin.c:423-457`.
fn list_plugins() {
    println!("Available plugins:\n");
    println!("{:<10}| {:<8}| Description", "Name", "Type");
    println!("------------------------------------------------------------");
    for p in plugins::all_plugins() {
        let type_str = match p.ptype() {
            PluginType::Full => "Full",
            PluginType::Partial => "Partial",
            PluginType::Any => "Any",
        };
        println!("{:<10}| {:<8}| {}", p.name(), type_str, p.description());
    }
}

fn standard_rcfile_candidates() -> Vec<PathBuf> {
    let mut v = vec![
        PathBuf::from("/etc/colorizerc"),
        PathBuf::from("/etc/cczerc"),
    ];
    if let Some(home) = std::env::var_os("HOME") {
        let h = PathBuf::from(home);
        v.push(h.join(".colorizerc"));
        v.push(h.join(".cczerc"));
    }
    v
}

fn pipeline_options_from_cli(cli: &Cli) -> PipelineOptions {
    let mut opts = PipelineOptions::default();
    for o in &cli.options {
        match o.as_str() {
            "nolookups" => opts.slookup = false,
            "lookups" => opts.slookup = true,
            "wordcolor" => opts.wordcolor = true,
            "nowordcolor" => opts.wordcolor = false,
            // scroll, transparent, cssfile are runtime / output-mode toggles
            // not yet implemented; tolerate them silently.
            _ => {}
        }
    }
    opts
}

fn process<R, S>(
    mut input: R,
    mut sink: S,
    pipeline: &Pipeline,
    opts: PipelineOptions,
    proc_opts: ProcessOpts,
) -> io::Result<()>
where
    R: BufRead,
    S: OutputSink,
{
    let mut buf = Vec::with_capacity(1024);
    loop {
        buf.clear();
        let n = input.read_until(b'\n', &mut buf)?;
        if n == 0 {
            break;
        }
        if buf.last() == Some(&b'\n') {
            buf.pop();
        }
        let line = String::from_utf8_lossy(&buf);
        let trimmed = if proc_opts.remove_facility {
            strip_syslog_facility(&line)
        } else {
            &line
        };
        pipeline.process_line(trimmed, &mut sink, opts)?;
    }
    sink.finish()?;
    Ok(())
}

/// Strip a leading `<NN>` syslog facility number from a log line. Mirrors the
/// `sscanf(subject, "<%u>", &remfac_tmp)` block at `ccze.c:698-703`. Returns
/// the original slice if the prefix doesn't match.
fn strip_syslog_facility(line: &str) -> &str {
    let bytes = line.as_bytes();
    if bytes.first() != Some(&b'<') {
        return line;
    }
    let close = match bytes.iter().position(|&b| b == b'>') {
        Some(i) => i,
        None => return line,
    };
    if close < 2 {
        return line;
    }
    if !bytes[1..close].iter().all(|&b| b.is_ascii_digit()) {
        return line;
    }
    &line[close + 1..]
}
