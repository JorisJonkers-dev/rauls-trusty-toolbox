// SPDX-License-Identifier: GPL-3.0-only
//! The program name in the text of a command that nvmod runs for raul.
//!
//! nvmod prints its own name (`nvmod init: made an Instance`, `usage: nvmod sort ...`, ``run `nvmod stage` ``). The
//! person typed `raul`, and raul has no `nvmod` program, so every line that comes back is rewritten to say `raul`.
//! Only the program name changes: the command words after it are the same in both.

use std::io::{self, Write};

/// A writer that passes whole lines on with `nvmod` as a program name written as `raul`.
pub(crate) struct Renamed<'a> {
    inner: &'a mut dyn Write,
    pending: Vec<u8>,
}

impl<'a> Renamed<'a> {
    pub(crate) fn new(inner: &'a mut dyn Write) -> Self {
        Self {
            inner,
            pending: Vec::new(),
        }
    }

    fn emit(&mut self, line: &[u8]) -> io::Result<()> {
        match std::str::from_utf8(line) {
            Ok(text) => self.inner.write_all(rename(text).as_bytes()),
            Err(_) => self.inner.write_all(line),
        }
    }

    fn drain_pending(&mut self) -> io::Result<()> {
        if self.pending.is_empty() {
            return Ok(());
        }
        let rest = std::mem::take(&mut self.pending);
        self.emit(&rest)
    }
}

impl Write for Renamed<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.pending.extend_from_slice(buf);
        while let Some(end) = self.pending.iter().position(|b| *b == b'\n') {
            let line: Vec<u8> = self.pending.drain(..=end).collect();
            self.emit(&line)?;
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.drain_pending()?;
        self.inner.flush()
    }
}

impl Drop for Renamed<'_> {
    fn drop(&mut self) {
        let _ = self.drain_pending();
    }
}

/// `line` with `nvmod` written as `raul` where it is a program name: at the start of the line (`nvmod init: ...`,
/// `nvmod: ...`), after `usage: ` and inside backticks (`` `nvmod stage` ``). The removed `install-nv-rs` command keeps
/// its text: `raul` answers it with its own explanation, and rewriting it would hide where the hint comes from.
pub(crate) fn rename(line: &str) -> String {
    let mut text = line.to_owned();
    for (from, to) in [("usage: nvmod ", "usage: raul "), ("`nvmod ", "`raul ")] {
        text = text.replace(from, to);
    }
    if let Some(rest) = text.strip_prefix("nvmod")
        && (rest.starts_with(' ') || rest.starts_with(':'))
    {
        text = format!("raul{rest}");
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_program_name_is_raul_at_the_start_after_usage_and_in_backticks() {
        assert_eq!(
            rename("nvmod init: made an Instance\n"),
            "raul init: made an Instance\n"
        );
        assert_eq!(rename("nvmod: no such thing\n"), "raul: no such thing\n");
        assert_eq!(
            rename("usage: nvmod sort [--apply]\n"),
            "usage: raul sort [--apply]\n"
        );
        assert_eq!(
            rename("run `nvmod stage` first\n"),
            "run `raul stage` first\n"
        );
    }

    #[test]
    fn other_text_with_the_word_is_left_alone() {
        for line in [
            "the nvmod-rs component\n",
            "see nvmod's log\n",
            "nvmodule loaded\n",
            "/home/a/.nvmod/lock\n",
            "{\"tool\":\"nvmod\"}\n",
        ] {
            assert_eq!(rename(line), line);
        }
    }

    #[test]
    fn lines_split_across_writes_are_renamed_whole() {
        let mut sink = Vec::new();
        {
            let mut renamed = Renamed::new(&mut sink);
            renamed.write_all(b"nvmod ini").unwrap();
            renamed.write_all(b"t: made\nusage: nvmod sort").unwrap();
            renamed.flush().unwrap();
        }
        assert_eq!(
            String::from_utf8(sink).unwrap(),
            "raul init: made\nusage: raul sort"
        );
    }

    #[test]
    fn text_that_is_not_utf8_passes_unchanged() {
        let mut sink = Vec::new();
        {
            let mut renamed = Renamed::new(&mut sink);
            renamed.write_all(b"nvmod \xff\n").unwrap();
        }
        assert_eq!(sink, b"nvmod \xff\n");
    }
}
