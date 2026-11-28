// SPDX-License-Identifier: GPL-3.0-only
//! Pure rules for stability recommendations and known-stable setups (CORE-05).
//!
//! Time is passed in by the caller; this module performs no I/O and reads no clock.

use std::collections::BTreeSet;

/// FNV-1a 64-bit offset basis.
const FNV_OFFSET: u64 = 0xcbf29ce484222325;
/// FNV-1a 64-bit prime.
const FNV_PRIME: u64 = 0x100000001b3;

/// A playable mod setup.
///
/// The order of `mods` and `plugins` matters, and a disabled mod is represented differently
/// from an enabled one. `ttw` records whether Tale of Two Wastelands is active for the setup.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Setup {
    /// Mods in display order, paired with their enabled state.
    pub mods: Vec<(String, bool)>,
    /// Active plugins in their load order.
    pub plugins: Vec<String>,
    /// Whether Tale of Two Wastelands is active.
    pub ttw: bool,
}

/// Returns a stable fingerprint of the parts of `setup` that affect stability.
///
/// The fingerprint covers whether TTW is active, the ordered list of enabled mods, and the
/// ordered list of active plugins. It is computed with a small hand-written FNV-1a 64-bit
/// hash over a canonical string and returned as lowercase hexadecimal.
///
/// This is a fingerprint for matching setups, not a cryptographic hash.
pub fn signature(setup: &Setup) -> String {
    let mut canonical = String::new();
    if setup.ttw {
        canonical.push('T');
    }
    canonical.push(';');
    let mut first = true;
    for (id, on) in &setup.mods {
        if !on {
            continue;
        }
        if !first {
            canonical.push(',');
        }
        canonical.push_str(id);
        first = false;
    }
    canonical.push('|');
    for (i, plugin) in setup.plugins.iter().enumerate() {
        if i > 0 {
            canonical.push(',');
        }
        canonical.push_str(plugin);
    }
    format!("{:016x}", fnv1a_64(canonical.as_bytes()))
}

fn fnv1a_64(bytes: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET;
    for &b in bytes {
        hash ^= u64::from(b);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// A setup that has passed a stability test.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct KnownStable {
    /// Fingerprint of the stable setup.
    pub signature: String,
    /// Unix seconds when the test passed.
    pub at_secs: u64,
    /// Human-readable label for the test result.
    pub label: String,
}

/// What the stability banner should show.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Banner {
    /// Nothing to recommend.
    Hidden,
    /// The current setup is untested and not dismissed: recommend a stability test.
    RecommendTest,
}

/// Known-stable history and per-signature banner dismissal.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Stability {
    /// Stable setups, newest first after operations that maintain order.
    known: Vec<KnownStable>,
    /// Signatures whose untested banner has been dismissed.
    dismissed: BTreeSet<String>,
}

impl Stability {
    /// Records that a stability test passed for `setup_sig` at `now`.
    ///
    /// If the signature is already known-stable, the existing entry is updated with the new
    /// time and label.
    pub fn record_pass(&mut self, setup_sig: String, label: String, now: u64) {
        if let Some(existing) = self.known.iter_mut().find(|k| k.signature == setup_sig) {
            existing.at_secs = now;
            existing.label = label;
        } else {
            self.known.push(KnownStable {
                signature: setup_sig,
                at_secs: now,
                label,
            });
        }
    }

    /// Returns `true` if `sig` is a known-stable setup.
    pub fn is_known_stable(&self, sig: &str) -> bool {
        self.known.iter().any(|k| k.signature == sig)
    }

    /// Decides whether the untested-change banner should show for `current_sig`.
    pub fn banner(&self, current_sig: &str) -> Banner {
        if self.is_known_stable(current_sig) || self.dismissed.contains(current_sig) {
            Banner::Hidden
        } else {
            Banner::RecommendTest
        }
    }

    /// Dismisses the banner for `sig`. A different setup will still show the banner.
    pub fn dismiss(&mut self, sig: String) {
        self.dismissed.insert(sig);
    }

    /// Removes `sig` from the known-stable list.
    ///
    /// If `sig` matches the current setup, the banner returns (unless it is also dismissed).
    pub fn forget(&mut self, sig: &str) {
        self.known.retain(|k| k.signature != sig);
    }

    /// Returns the most recent known-stable entry.
    pub fn last_stable(&self) -> Option<&KnownStable> {
        self.known.iter().max_by_key(|k| k.at_secs)
    }

    /// Returns the known-stable entry whose signature exactly matches `current_sig`.
    ///
    /// This drives the "Matches now" chip.
    pub fn matches_now(&self, current_sig: &str) -> Option<&KnownStable> {
        self.known.iter().find(|k| k.signature == current_sig)
    }
}

#[cfg(test)]
mod tests {
    use super::{signature, Banner, Setup, Stability};

    fn setup_a() -> Setup {
        Setup {
            mods: vec![
                ("mod1".to_string(), true),
                ("mod2".to_string(), true),
                ("mod3".to_string(), false),
            ],
            plugins: vec!["plugin1.esp".to_string(), "plugin2.esp".to_string()],
            ttw: false,
        }
    }

    fn setup_b() -> Setup {
        // Same enabled mod set as setup_a but different order.
        Setup {
            mods: vec![
                ("mod2".to_string(), true),
                ("mod1".to_string(), true),
                ("mod3".to_string(), false),
            ],
            plugins: vec!["plugin1.esp".to_string(), "plugin2.esp".to_string()],
            ttw: false,
        }
    }

    fn setup_disabled() -> Setup {
        // Disabling mod2 instead of enabling it.
        Setup {
            mods: vec![
                ("mod1".to_string(), true),
                ("mod2".to_string(), false),
                ("mod3".to_string(), false),
            ],
            plugins: vec!["plugin1.esp".to_string(), "plugin2.esp".to_string()],
            ttw: false,
        }
    }

    #[test]
    fn same_setup_same_signature() {
        let a = setup_a();
        let b = setup_a();
        assert_eq!(signature(&a), signature(&b));
    }

    #[test]
    fn reordering_mods_changes_signature() {
        assert_ne!(signature(&setup_a()), signature(&setup_b()));
    }

    #[test]
    fn disabling_a_mod_changes_signature() {
        assert_ne!(signature(&setup_a()), signature(&setup_disabled()));
    }

    #[test]
    fn passed_test_hides_banner() {
        let mut st = Stability::default();
        let sig = signature(&setup_a());
        st.record_pass(sig.clone(), "passed".to_string(), 1000);
        assert_eq!(st.banner(&sig), Banner::Hidden);
    }

    #[test]
    fn untested_change_shows_banner() {
        let st = Stability::default();
        let sig = signature(&setup_a());
        assert_eq!(st.banner(&sig), Banner::RecommendTest);
    }

    #[test]
    fn dismiss_hides_only_for_that_signature() {
        let mut st = Stability::default();
        let sig_a = signature(&setup_a());
        let sig_b = signature(&setup_b());
        st.dismiss(sig_a.clone());
        assert_eq!(st.banner(&sig_a), Banner::Hidden);
        assert_eq!(st.banner(&sig_b), Banner::RecommendTest);
    }

    #[test]
    fn forget_brings_banner_back() {
        let mut st = Stability::default();
        let sig = signature(&setup_a());
        st.record_pass(sig.clone(), "passed".to_string(), 1000);
        assert_eq!(st.banner(&sig), Banner::Hidden);
        st.forget(&sig);
        assert_eq!(st.banner(&sig), Banner::RecommendTest);
    }

    #[test]
    fn last_stable_is_newest_pass() {
        let mut st = Stability::default();
        let sig_a = signature(&setup_a());
        let sig_b = signature(&setup_b());
        st.record_pass(sig_a.clone(), "first".to_string(), 1000);
        st.record_pass(sig_b.clone(), "second".to_string(), 2000);
        let last = st.last_stable().expect("should have a last stable");
        assert_eq!(last.signature, sig_b);
        assert_eq!(last.at_secs, 2000);
    }

    #[test]
    fn matches_now_returns_current_signature() {
        let mut st = Stability::default();
        let sig = signature(&setup_a());
        st.record_pass(sig.clone(), "passed".to_string(), 1000);
        let matched = st.matches_now(&sig).expect("should match");
        assert_eq!(matched.signature, sig);
    }
}
