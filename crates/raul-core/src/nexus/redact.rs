// SPDX-License-Identifier: GPL-3.0-only
//! Redaction of the Nexus API key from text that may be shown, logged or put in a report.

use super::key::Secret;

/// What replaces a secret.
pub const MASK: &str = "[redacted]";

/// `text` with the `secret` and the value of any `apikey` header or `apikey=` pair replaced by [`MASK`].
///
/// The pattern part catches a key the program does not know it holds (a curl error that echoes a header, a
/// response body that repeats the key).
#[must_use]
pub fn redact(text: &str, secret: Option<&Secret>) -> String {
    let mut out = text.to_owned();
    if let Some(secret) = secret {
        let value = secret.expose();
        if !value.is_empty() {
            out = out.replace(value, MASK);
        }
    }
    redact_apikey_values(&out)
}

/// Replaces the token after `apikey` (any case) and a `:` or `=`, with optional quotes and spaces between.
fn redact_apikey_values(text: &str) -> String {
    const NAME: &str = "apikey";
    let lower = text.to_ascii_lowercase();
    let mut out = String::with_capacity(text.len());
    let mut at = 0;
    while let Some(found) = lower[at..].find(NAME) {
        let name_end = at + found + NAME.len();
        out.push_str(&text[at..name_end]);
        let rest = &text[name_end..];
        let skip = |c: char| c == '"' || c == '\'' || c == ' ' || c == '\t';
        let lead = rest.len() - rest.trim_start_matches(skip).len();
        let after = &rest[lead..];
        if let Some(sep) = after.strip_prefix([':', '=']) {
            let gap = sep.len() - sep.trim_start_matches(skip).len();
            let value_start = name_end + lead + 1 + gap;
            let value_len = text[value_start..]
                .find(|c: char| c.is_whitespace() || c == '"' || c == '\'' || c == '&')
                .unwrap_or(text.len() - value_start);
            out.push_str(&text[name_end..value_start]);
            if value_len > 0 {
                out.push_str(MASK);
            }
            at = value_start + value_len;
        } else {
            at = name_end;
        }
    }
    out.push_str(&text[at..]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_known_key_is_removed_wherever_it_appears() {
        let secret = Secret::new("abcDEF123-xyz==").unwrap();
        let text = redact(
            "failed with abcDEF123-xyz== in the url ?k=abcDEF123-xyz==",
            Some(&secret),
        );
        assert!(!text.contains("abcDEF"), "{text}");
    }

    #[test]
    fn the_value_of_an_apikey_header_is_removed_without_knowing_the_key() {
        assert_eq!(
            redact("apikey: SECRET123 next", None),
            "apikey: [redacted] next"
        );
        assert_eq!(
            redact("APIKEY=SECRET123&x=1", None),
            "APIKEY=[redacted]&x=1"
        );
        assert_eq!(
            redact("header = \"apikey: SECRET123\"", None),
            "header = \"apikey: [redacted]\""
        );
    }

    #[test]
    fn text_without_a_key_is_unchanged() {
        assert_eq!(
            redact("no secrets here, apikey is a word", None),
            "no secrets here, apikey is a word"
        );
    }
}
