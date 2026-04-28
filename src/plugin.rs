//! Plugin trait + dispatch pipeline.
//!
//! Mirrors the C `ccze_plugin_run` flow at `src/ccze.c:706-730`. The C source
//! distinguishes three cases by the value of `rest`:
//!   1. No Full plugin matched (status==0): wordcolor the whole line + newline.
//!   2. A Full plugin matched and returned `rest != NULL`: try Partial plugins on
//!      the remainder, then wordcolor whatever is left, then newline.
//!   3. A Full plugin matched and returned `rest == NULL`: the plugin emitted
//!      its own newline; the dispatcher does nothing else.
//!
//! In Rust those map directly onto `HandleResult::{NoMatch, Remainder, Consumed}`.

use crate::sink::OutputSink;
use crate::wordcolor;
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginType {
    Full,
    Partial,
    #[allow(dead_code)]
    Any,
}

pub enum HandleResult {
    /// Plugin matched and emitted everything, including the trailing newline.
    /// The dispatcher must not emit any further output for this line.
    Consumed,
    /// Plugin matched and emitted a prefix; this string still needs colorising.
    Remainder(String),
    /// Plugin did not match; the dispatcher should try the next plugin.
    NoMatch,
}

pub trait Plugin: Send + Sync {
    fn name(&self) -> &'static str;
    fn ptype(&self) -> PluginType;
    #[allow(dead_code)]
    fn description(&self) -> &'static str;
    fn handle(&self, line: &str, sink: &mut dyn OutputSink) -> io::Result<HandleResult>;
}

#[derive(Debug, Clone, Copy)]
pub struct PipelineOptions {
    pub wordcolor: bool,
    pub slookup: bool,
}

impl Default for PipelineOptions {
    fn default() -> Self {
        Self {
            wordcolor: true,
            slookup: true,
        }
    }
}

pub struct Pipeline {
    plugins: Vec<Box<dyn Plugin>>,
}

impl Pipeline {
    pub fn new(plugins: Vec<Box<dyn Plugin>>) -> Self {
        Self { plugins }
    }

    pub fn process_line(
        &self,
        line: &str,
        sink: &mut dyn OutputSink,
        opts: PipelineOptions,
    ) -> io::Result<()> {
        // Phase 1: Full plugins. First match wins.
        enum FullOutcome {
            Unmatched,
            Consumed,
            Remainder(String),
        }
        let mut outcome = FullOutcome::Unmatched;
        for p in &self.plugins {
            if !matches!(p.ptype(), PluginType::Full | PluginType::Any) {
                continue;
            }
            match p.handle(line, sink)? {
                HandleResult::NoMatch => continue,
                HandleResult::Consumed => {
                    outcome = FullOutcome::Consumed;
                    break;
                }
                HandleResult::Remainder(r) => {
                    outcome = FullOutcome::Remainder(r);
                    break;
                }
            }
        }

        match outcome {
            FullOutcome::Consumed => {
                // Plugin emitted its own newline; we're done.
                Ok(())
            }
            FullOutcome::Unmatched => {
                // No Full plugin recognised the line — colour everything as words.
                wordcolor::process(line, sink, opts.wordcolor, opts.slookup)?;
                sink.newline()
            }
            FullOutcome::Remainder(rest) => {
                // Phase 2: try Partial plugins on the residue.
                let mut partial_text: Option<String> = None;
                let mut partial_matched = false;
                for p in &self.plugins {
                    if !matches!(p.ptype(), PluginType::Partial | PluginType::Any) {
                        continue;
                    }
                    match p.handle(&rest, sink)? {
                        HandleResult::NoMatch => continue,
                        HandleResult::Consumed => {
                            partial_matched = true;
                            break;
                        }
                        HandleResult::Remainder(r) => {
                            partial_matched = true;
                            partial_text = Some(r);
                            break;
                        }
                    }
                }

                // Phase 3: wordcolor whatever is left, then newline.
                let to_color: &str = if partial_matched {
                    partial_text.as_deref().unwrap_or("")
                } else {
                    &rest
                };
                wordcolor::process(to_color, sink, opts.wordcolor, opts.slookup)?;
                sink.newline()
            }
        }
    }
}
