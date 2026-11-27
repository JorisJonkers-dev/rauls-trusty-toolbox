// SPDX-License-Identifier: GPL-3.0-only
//! Reading and checking a list.

use std::collections::BTreeSet;

use crate::error::Error;
use crate::model::{ModEntry, ModList, SCHEMA, Source};

impl ModList {
    /// Read a list from TOML text and check it against the rules of the format.
    ///
    /// # Errors
    /// [`Error::Syntax`] for text that is not a list, [`Error::Schema`] for another schema version and
    /// [`Error::Invalid`] with every broken rule at once.
    pub fn parse(text: &str) -> Result<ModList, Error> {
        let table: toml::Table = text
            .parse()
            .map_err(|e: toml::de::Error| Error::Syntax(e.to_string()))?;
        match table.get("schema").and_then(toml::Value::as_str) {
            None => return Err(Error::Schema("the Mod list has no `schema` line".into())),
            Some(SCHEMA) => {}
            Some(other) => {
                return Err(Error::Schema(format!(
                    "the Mod list is `{other}` and this raul reads `{SCHEMA}`; update raul or the list"
                )));
            }
        }
        let list: ModList = table
            .try_into()
            .map_err(|e: toml::de::Error| Error::Syntax(e.to_string()))?;
        let problems = list.problems();
        if problems.is_empty() {
            Ok(list)
        } else {
            Err(Error::Invalid(problems))
        }
    }

    /// Every rule of the format that this list breaks, one sentence each.
    #[must_use]
    pub fn problems(&self) -> Vec<String> {
        let mut out = Vec::new();
        if !is_slug(&self.id) {
            out.push(format!(
                "the list id `{}` is not a slug (a-z, 0-9 and -)",
                self.id
            ));
        }
        let mut groups = BTreeSet::new();
        for g in &self.groups {
            if !is_slug(&g.id) {
                out.push(format!("group id `{}` is not a slug", g.id));
            }
            if !groups.insert(g.id.as_str()) {
                out.push(format!("group id `{}` is used twice", g.id));
            }
        }
        let mut seen = BTreeSet::new();
        for m in &self.mods {
            if !seen.insert(m.id.as_str()) {
                out.push(format!("Mod id `{}` is used twice", m.id));
            }
            mod_problems(m, &groups, &mut out);
        }
        out
    }
}

fn mod_problems(m: &ModEntry, groups: &BTreeSet<&str>, out: &mut Vec<String>) {
    let id = &m.id;
    if !is_slug(id) {
        out.push(format!("Mod id `{id}` is not a slug (a-z, 0-9 and -)"));
    }
    if m.name.trim().is_empty() {
        out.push(format!("Mod `{id}` has no name"));
    }
    if let Some(g) = &m.group
        && !groups.contains(g.as_str())
    {
        out.push(format!(
            "Mod `{id}` is in group `{g}`, which the list does not define"
        ));
    }
    if !m.enabled && m.why_off.as_deref().is_none_or(|w| w.trim().is_empty()) {
        out.push(format!(
            "Mod `{id}` is switched off and does not say why (`why_off`)"
        ));
    }
    if m.plugins.iter().any(|p| p.trim().is_empty()) {
        out.push(format!("Mod `{id}` lists a blank Plugin name"));
    }
    match &m.source {
        Source::Nexus {
            game,
            mod_id,
            file,
            md5,
            ..
        } => {
            if game.trim().is_empty() {
                out.push(format!("Mod `{id}`: the Nexus game is empty"));
            }
            if *mod_id == 0 {
                out.push(format!("Mod `{id}`: the Nexus mod id is 0"));
            }
            if *file == Some(0) {
                out.push(format!(
                    "Mod `{id}`: the Nexus file id is 0; leave it out when it is not known"
                ));
            }
            if let Some(md5) = md5
                && !is_hex(md5, 32)
            {
                out.push(format!("Mod `{id}`: the md5 is not 32 hex digits"));
            }
        }
        Source::Url { url, sha256, .. } => {
            if !url.starts_with("https://") {
                out.push(format!(
                    "Mod `{id}`: a `url` source must be an https address"
                ));
            }
            if !is_hex(sha256, 64) {
                out.push(format!(
                    "Mod `{id}`: a `url` source needs a sha256 of 64 hex digits"
                ));
            }
        }
        Source::Manual { url: Some(url), .. } => {
            if !(url.starts_with("https://") || url.starts_with("http://")) {
                out.push(format!(
                    "Mod `{id}`: the link must start with http:// or https://"
                ));
            }
        }
        Source::Manual { .. } => {}
        Source::Tool { tool, .. } | Source::Generated { tool } => {
            if tool.trim().is_empty() {
                out.push(format!("Mod `{id}`: the tool is empty"));
            }
        }
        Source::Addon { id: addon } => {
            if addon.trim().is_empty() {
                out.push(format!("Mod `{id}`: the add-on id is empty"));
            }
        }
    }
}

fn is_slug(s: &str) -> bool {
    !s.is_empty()
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

fn is_hex(s: &str, len: usize) -> bool {
    s.len() == len
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
