// SPDX-License-Identifier: GPL-3.0-only
//! Pure play-session and playtime bookkeeping.
//!
//! Time is passed in as seconds since the Unix epoch; this module performs no
//! I/O and reads no clocks.

/// A finished play session for a single profile.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Session {
    /// Profile the session was played with.
    pub profile: String,
    /// Start time in seconds since the Unix epoch.
    pub started: u64,
    /// End time in seconds since the Unix epoch.
    pub ended: u64,
}

impl Session {
    /// Length of the session in whole seconds, saturating at zero.
    pub fn seconds(&self) -> u64 {
        self.ended.saturating_sub(self.started)
    }
}

/// Value kept by the caller while a game session is running.
#[derive(Clone, Debug, PartialEq)]
pub struct Running {
    /// Profile the running session belongs to.
    pub profile: String,
    /// Start time in seconds since the Unix epoch.
    pub started: u64,
}

/// Recorded play sessions and playtime totals.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Playtime {
    sessions: Vec<Session>,
}

impl Playtime {
    /// Start a session now and return the value the caller keeps while it runs.
    pub fn start(&mut self, profile: &str, now: u64) -> Running {
        Running {
            profile: profile.to_string(),
            started: now,
        }
    }

    /// Stop a running session, record it, and return a reference to it.
    ///
    /// A stop time before the start time records a zero-length session.
    pub fn stop(&mut self, running: Running, now: u64) -> &Session {
        let session = Session {
            profile: running.profile,
            started: running.started,
            ended: now,
        };
        self.sessions.push(session);
        self.sessions.last().expect("just pushed a session")
    }

    /// Total playtime in seconds for the given profile.
    pub fn total(&self, profile: &str) -> u64 {
        self.sessions
            .iter()
            .filter(|s| s.profile == profile)
            .map(|s| s.seconds())
            .sum()
    }

    /// End time of the newest session for the given profile, if any.
    pub fn last_played(&self, profile: &str) -> Option<u64> {
        self.sessions
            .iter()
            .filter(|s| s.profile == profile)
            .map(|s| s.ended)
            .max()
    }
}

/// Format a duration in seconds the way the Home status card does.
///
/// Examples: `3 s`, `59 s`, `1 min`, `1 h 1 min`, `2 h`.
pub fn duration_text(secs: u64) -> String {
    if secs < 60 {
        format!("{secs} s")
    } else {
        let mins = secs / 60;
        if mins < 60 {
            format!("{mins} min")
        } else {
            let hours = mins / 60;
            let remaining_mins = mins % 60;
            if remaining_mins == 0 {
                format!("{hours} h")
            } else {
                format!("{hours} h {remaining_mins} min")
            }
        }
    }
}

/// Relative wording for the last-played timestamp.
///
/// Examples: `just now`, `5 min ago`, `2 h ago`, `yesterday`, `3 days ago`.
pub fn last_played_text(last: u64, now: u64) -> String {
    let diff_secs = now.saturating_sub(last);
    let mins = (diff_secs as f64 / 60.0).round() as u64;

    if mins < 1 {
        "just now".to_string()
    } else if mins < 60 {
        format!("{mins} min ago")
    } else {
        let hours = (mins as f64 / 60.0).round() as u64;
        if hours < 24 {
            format!("{hours} h ago")
        } else {
            let days = (hours as f64 / 24.0).round() as u64;
            if days == 1 {
                "yesterday".to_string()
            } else {
                format!("{days} days ago")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_stop_records_length() {
        let mut playtime = Playtime::default();
        let running = playtime.start("Vanilla+", 1000);
        let session = playtime.stop(running, 1063);
        assert_eq!(session.profile, "Vanilla+");
        assert_eq!(session.started, 1000);
        assert_eq!(session.ended, 1063);
        assert_eq!(session.seconds(), 63);
    }

    #[test]
    fn totals_are_per_profile() {
        let mut playtime = Playtime::default();
        playtime.stop(playtime.start("Vanilla+", 0), 120);
        playtime.stop(playtime.start("Vanilla+", 200), 500);
        playtime.stop(playtime.start("Hardcore", 0), 300);

        assert_eq!(playtime.total("Vanilla+"), 420);
        assert_eq!(playtime.total("Hardcore"), 300);
        assert_eq!(playtime.total("Missing"), 0);
    }

    #[test]
    fn last_played_is_newest_end() {
        let mut playtime = Playtime::default();
        playtime.stop(playtime.start("Vanilla+", 0), 100);
        playtime.stop(playtime.start("Vanilla+", 200), 500);
        playtime.stop(playtime.start("Hardcore", 0), 300);

        assert_eq!(playtime.last_played("Vanilla+"), Some(500));
        assert_eq!(playtime.last_played("Hardcore"), Some(300));
        assert_eq!(playtime.last_played("Missing"), None);
    }

    #[test]
    fn reversed_clock_records_zero() {
        let mut playtime = Playtime::default();
        let running = playtime.start("Vanilla+", 1000);
        let session = playtime.stop(running, 500);
        assert_eq!(session.seconds(), 0);
        assert_eq!(playtime.total("Vanilla+"), 0);
    }

    #[test]
    fn duration_text_formats() {
        assert_eq!(duration_text(3), "3 s");
        assert_eq!(duration_text(59), "59 s");
        assert_eq!(duration_text(60), "1 min");
        assert_eq!(duration_text(61 * 60), "1 h 1 min");
        assert_eq!(duration_text(2 * 60 * 60), "2 h");
    }

    #[test]
    fn last_played_text_formats() {
        assert_eq!(last_played_text(1000, 1000), "just now");
        assert_eq!(last_played_text(1000, 1000 + 5 * 60), "5 min ago");
        assert_eq!(last_played_text(1000, 1000 + 2 * 60 * 60), "2 h ago");
        assert_eq!(last_played_text(1000, 1000 + 24 * 60 * 60), "yesterday");
        assert_eq!(last_played_text(1000, 1000 + 3 * 24 * 60 * 60), "3 days ago");
    }
}
