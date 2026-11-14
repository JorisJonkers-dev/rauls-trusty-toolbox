// SPDX-License-Identifier: GPL-3.0-only
//! Running the `nvmod` command line.

use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// What one run of the command line printed and how it ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    /// The exit code, `None` when the process did not exit normally (killed by a signal).
    pub code: Option<i64>,
    /// Standard output.
    pub stdout: String,
    /// Standard error.
    pub stderr: String,
}

/// Something that runs `nvmod` with arguments. The real one starts a process; tests run the command line in
/// process or answer from a script.
pub trait Exec {
    /// Runs `nvmod <args>`.
    ///
    /// # Errors
    /// The reason the command line could not be started at all.
    fn run(&self, args: &[String]) -> Result<Output, String>;

    /// Runs `nvmod <args>` and calls `on_line` with each line of standard output (without its newline) as it
    /// is printed. The returned [`Output`] holds the whole of it as well. The default runs the command and
    /// then hands over the lines, which is right for an in-process or scripted run; [`ProcessExec`] delivers
    /// each line while the process is still running.
    ///
    /// # Errors
    /// The reason the command line could not be started at all.
    fn run_streaming(
        &self,
        args: &[String],
        on_line: &mut dyn FnMut(&str),
    ) -> Result<Output, String> {
        let out = self.run(args)?;
        for line in out.stdout.lines() {
            on_line(line);
        }
        Ok(out)
    }
}

/// Starts the `nvmod` program as a subprocess.
#[derive(Debug, Clone)]
pub struct ProcessExec {
    program: PathBuf,
}

impl ProcessExec {
    /// Runs the program at `program` (a path, or a bare name found on `PATH`).
    pub fn new(program: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
        }
    }
}

impl Default for ProcessExec {
    fn default() -> Self {
        Self::new("nvmod")
    }
}

impl Exec for ProcessExec {
    fn run(&self, args: &[String]) -> Result<Output, String> {
        let out = Command::new(&self.program)
            .args(args)
            .output()
            .map_err(|e| format!("could not start {}: {e}", self.program.display()))?;
        Ok(Output {
            code: out.status.code().map(i64::from),
            stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        })
    }

    fn run_streaming(
        &self,
        args: &[String],
        on_line: &mut dyn FnMut(&str),
    ) -> Result<Output, String> {
        let mut child = Command::new(&self.program)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| self.start_error(&e))?;
        // Standard error is read on its own thread so a full pipe cannot stall the process.
        let stderr = child.stderr.take();
        let reader = std::thread::spawn(move || {
            let mut bytes = Vec::new();
            if let Some(mut pipe) = stderr {
                let _ = pipe.read_to_end(&mut bytes);
            }
            String::from_utf8_lossy(&bytes).into_owned()
        });
        let mut stdout = String::new();
        if let Some(pipe) = child.stdout.take() {
            for chunk in BufReader::new(pipe).split(b'\n') {
                let Ok(bytes) = chunk else { break };
                let line = String::from_utf8_lossy(&bytes).into_owned();
                on_line(&line);
                stdout.push_str(&line);
                stdout.push('\n');
            }
        }
        let status = child.wait().map_err(|e| self.start_error(&e))?;
        Ok(Output {
            code: status.code().map(i64::from),
            stdout,
            stderr: reader.join().unwrap_or_default(),
        })
    }
}

impl ProcessExec {
    fn start_error(&self, e: &std::io::Error) -> String {
        format!("could not start {}: {e}", self.program.display())
    }
}
