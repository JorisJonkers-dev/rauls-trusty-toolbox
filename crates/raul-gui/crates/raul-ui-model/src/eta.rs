// SPDX-License-Identifier: GPL-3.0-only
//! Minutes-left estimate, job keys and status line text for running jobs.

const MIN_SAMPLE_SPAN_SECONDS: f64 = 5.0;
const RECENT_WINDOW_SECONDS: f64 = 60.0;

/// Estimate time remaining from progress samples.
///
/// Samples are `(now_secs, fraction)` pairs. The estimate needs at least two
/// samples spanning at least five seconds with the fraction moving forward.
/// A lower fraction resets the history, which happens when a job enters a new
/// stage.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Eta {
    samples: Vec<(f64, f32)>,
}

impl Eta {
    /// Create an empty estimator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a progress sample.
    ///
    /// `now` is a monotonic timestamp in seconds and `fraction` is in the
    /// range `0.0` to `1.0`. A fraction that goes backwards clears the sample
    /// history.
    pub fn sample(&mut self, now: f64, fraction: f32) {
        if let Some(&(_, last)) = self.samples.last() {
            if fraction < last {
                self.samples.clear();
            }
        }
        self.samples.push((now, fraction));
    }

    /// Whole minutes left, if there is enough data to estimate.
    ///
    /// Returns `Some(0)` only when the latest fraction is `1.0` or more.
    /// Otherwise the recent rate is computed from samples taken in the last
    /// minute (or all samples if there are fewer), and the remaining fraction
    /// is divided by that rate. The result is rounded up to whole minutes and
    /// clamped to at least one minute while the job is not finished.
    #[allow(clippy::cast_possible_truncation)]
    pub fn minutes_left(&self) -> Option<u32> {
        let &(_, latest_fraction) = self.samples.last()?;
        if latest_fraction >= 1.0_f32 {
            return Some(0);
        }
        if self.samples.len() < 2 {
            return None;
        }

        let latest_time = self.samples.last()?.0;
        let mut window: Vec<_> = self
            .samples
            .iter()
            .filter(|&&(t, _)| latest_time - t <= RECENT_WINDOW_SECONDS)
            .copied()
            .collect();
        if window.len() < 2 {
            window = self.samples.clone();
        }
        if window.len() < 2 {
            return None;
        }

        let (first_time, first_fraction) = window[0];
        let (last_time, last_fraction) = window[window.len() - 1];
        let span = last_time - first_time;
        let progress = f64::from(last_fraction) - f64::from(first_fraction);
        if span < MIN_SAMPLE_SPAN_SECONDS || progress <= 0.0 {
            return None;
        }

        let rate = progress / span; // fraction per second
        let remaining = 1.0 - f64::from(last_fraction);
        let seconds_left = remaining / rate;
        let minutes = (seconds_left / 60.0).ceil().max(1.0);
        Some(minutes as u32)
    }
}

/// Format a minute estimate the way the demo does.
///
/// `None` stays `None`. `Some(0)` becomes `"0 min left"`, `Some(1)` becomes
/// `"1 min left"`, and any larger value becomes `"about N min left"`.
pub fn minutes_left_text(minutes: Option<u32>) -> Option<String> {
    minutes.map(|m| match m {
        0 => "0 min left".to_owned(),
        1 => "1 min left".to_owned(),
        _ => format!("about {m} min left"),
    })
}

/// A job kind, optionally scoped to a profile.
///
/// The demo uses per-profile keys such as `far:<profile>`, `stab:<profile>`,
/// `bench:<profile>` and `pics:<profile>`. Shared keys such as `ttw`,
/// `update`, `check`, `ml`, `doctor`, `install` and `find` have no profile.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct JobKey {
    /// The operation kind, e.g. `"far"` or `"ttw"`.
    pub kind: String,
    /// The profile id when the job is per-profile.
    pub profile: Option<String>,
}

impl JobKey {
    /// A shared job key, used for operations that are global such as an
    /// update or a TTW install.
    pub fn shared(kind: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            profile: None,
        }
    }

    /// A per-profile job key, used when the same kind of work runs
    /// independently per profile such as a LOD build or a stability test.
    pub fn for_profile(kind: impl Into<String>, profile: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            profile: Some(profile.into()),
        }
    }
}

/// Status line text for the footer.
///
/// With no jobs the text is `"Ready"`. With one job it is
/// `"<label> · N %"`. With several jobs the first job is shown and the rest
/// are summarized as `"· N more"`.
pub fn status_line(jobs: &[(String, Option<f32>)]) -> String {
    match jobs {
        [] => "Ready".to_owned(),
        [(label, fraction)] => format_percent(label, *fraction),
        [(label, fraction), rest @ ..] => {
            let mut s = format_percent(label, *fraction);
            let _ = write!(s, " · {} more", rest.len());
            s
        }
    }
}

fn format_percent(label: &str, fraction: Option<f32>) -> String {
    match fraction {
        Some(f) => format!("{label} · {:.0} %", f * 100.0),
        None => label.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_sample_gives_no_estimate() {
        let mut eta = Eta::new();
        eta.sample(0.0, 0.4);
        assert_eq!(eta.minutes_left(), None);
    }

    #[test]
    fn steady_rate_gives_one_minute() {
        let mut eta = Eta::new();
        eta.sample(0.0, 0.4);
        eta.sample(20.0, 0.6);
        assert_eq!(eta.minutes_left(), Some(1));
    }

    #[test]
    fn slower_rate_gives_more_minutes() {
        let mut eta = Eta::new();
        eta.sample(0.0, 0.4);
        eta.sample(40.0, 0.6);
        assert_eq!(eta.minutes_left(), Some(2));
    }

    #[test]
    fn backwards_fraction_resets_samples() {
        let mut eta = Eta::new();
        eta.sample(0.0, 0.5);
        eta.sample(10.0, 0.6);
        eta.sample(15.0, 0.2);
        assert_eq!(eta.minutes_left(), None);
    }

    #[test]
    fn finished_has_zero_minutes_left() {
        let mut eta = Eta::new();
        eta.sample(0.0, 0.0);
        eta.sample(10.0, 1.0);
        assert_eq!(eta.minutes_left(), Some(0));
    }

    #[test]
    fn job_key_per_profile_vs_shared() {
        let shared = JobKey::shared("ttw");
        let lod_a = JobKey::for_profile("far", "beaut");
        let lod_b = JobKey::for_profile("far", "hard");
        let other_shared = JobKey::shared("ttw");
        assert_eq!(shared, other_shared);
        assert_ne!(shared, lod_a);
        assert_ne!(lod_a, lod_b);
        assert_eq!(lod_a, JobKey::for_profile("far", "beaut"));
    }

    #[test]
    fn status_line_variants() {
        assert_eq!(status_line(&[]), "Ready");
        assert_eq!(
            status_line(&[("LOD".to_owned(), Some(0.42))]),
            "LOD · 42 %"
        );
        assert_eq!(
            status_line(&[
                ("LOD".to_owned(), Some(0.42)),
                ("Update".to_owned(), Some(0.10)),
                ("Install".to_owned(), Some(0.75)),
            ]),
            "LOD · 42 % · 2 more"
        );
    }

    #[test]
    fn minutes_left_text_wording() {
        assert_eq!(minutes_left_text(None), None);
        assert_eq!(minutes_left_text(Some(0)), Some("0 min left".to_owned()));
        assert_eq!(minutes_left_text(Some(1)), Some("1 min left".to_owned()));
        assert_eq!(
            minutes_left_text(Some(3)),
            Some("about 3 min left".to_owned())
        );
    }
}
