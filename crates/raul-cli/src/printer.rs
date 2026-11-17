// SPDX-License-Identifier: GPL-3.0-only
//! The output of a driver command: `raul-events` lines on standard output with `--json`, else text for people on
//! standard error.

use std::io::Write;

use raul_drivers::events::{CONTRACT, Event, EventLine, SCHEMA_VERSION};

/// Prints events as `raul-events` lines or as text for people.
pub(crate) struct Printer<'a> {
    pub(crate) command: &'static str,
    pub(crate) out: &'a mut dyn Write,
    pub(crate) err: &'a mut dyn Write,
    pub(crate) json: bool,
    pub(crate) seq: u64,
    pub(crate) last_percent: Option<(String, u64)>,
}

impl Printer<'_> {
    pub(crate) fn emit(&mut self, event: Event) {
        if self.json {
            self.seq += 1;
            let line = EventLine {
                schema_version: SCHEMA_VERSION,
                contract: CONTRACT.to_owned(),
                seq: self.seq,
                command: self.command.to_owned(),
                event,
            };
            let _ = writeln!(
                self.out,
                "{}",
                serde_json::to_string(&line).unwrap_or_default()
            );
            let _ = self.out.flush();
            return;
        }
        match event {
            Event::Started { tool, .. } => {
                let _ = writeln!(
                    self.err,
                    "raul {}: running {} (SHA-256 {})",
                    self.command,
                    tool.path.display(),
                    tool.sha256
                );
            }
            Event::Phase { phase } => {
                let _ = writeln!(self.err, "raul {}: {phase}", self.command);
            }
            Event::Progress {
                phase, done, of, ..
            } if of > 0 => {
                // One line per tenth, so a long phase does not flood the terminal.
                let tenth = done.saturating_mul(10) / of;
                if self.last_percent.as_ref() != Some(&(phase.clone(), tenth)) {
                    let _ = writeln!(self.err, "raul {}: {phase} {done}/{of}", self.command);
                    self.last_percent = Some((phase, tenth));
                }
            }
            Event::Warning { message } => {
                let _ = writeln!(self.err, "raul {}: warning: {message}", self.command);
            }
            _ => {}
        }
    }
}
