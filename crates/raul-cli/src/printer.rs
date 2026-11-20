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

/// A [`Write`] that hands each complete line to a callback as it is written, so a run that prints `raul-events` lines
/// can be read line by line without a process or a pipe.
pub(crate) struct LineSink<'a> {
    partial: Vec<u8>,
    on_line: &'a mut dyn FnMut(&str),
}

impl<'a> LineSink<'a> {
    pub(crate) fn new(on_line: &'a mut dyn FnMut(&str)) -> Self {
        Self {
            partial: Vec::new(),
            on_line,
        }
    }

    /// Hands over a last line that has no newline, if there is one.
    pub(crate) fn finish(&mut self) {
        if !self.partial.is_empty() {
            let line = std::mem::take(&mut self.partial);
            (self.on_line)(&String::from_utf8_lossy(&line));
        }
    }
}

impl Write for LineSink<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        for &byte in buf {
            if byte == b'\n' {
                let line = std::mem::take(&mut self.partial);
                (self.on_line)(&String::from_utf8_lossy(&line));
            } else {
                self.partial.push(byte);
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sink_hands_over_each_line_when_it_is_complete_and_the_last_one_at_the_end() {
        let mut seen = Vec::new();
        {
            let mut take = |line: &str| seen.push(line.to_owned());
            let mut sink = LineSink::new(&mut take);
            sink.write_all(b"one\ntw").unwrap();
            sink.write_all(b"o\n\nthree").unwrap();
            sink.flush().unwrap();
            sink.finish();
            sink.finish();
        }
        assert_eq!(seen, ["one", "two", "", "three"]);
    }
}
