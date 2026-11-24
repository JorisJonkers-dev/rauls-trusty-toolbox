// SPDX-License-Identifier: GPL-3.0-only
//! An override plugin made from the player's own copy of a mod's plugin, at the time the add-on is applied. It lists
//! the source's masters and then the source itself, so every form id keeps its meaning, and holds a copy of each
//! record of the source whose type and model (`MODL`, any case, either slash) a rule names, with the model set to the
//! rule's new one, the model's texture hashes (`MODT`) dropped and, when the rule says, the object bounds (`OBND`)
//! set. Nothing else of the source is copied.

use std::io::Read;

use serde::Deserialize;

/// One kind of record an add-on overrides, keyed by its model path.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordRule {
    /// The record type, four letters, for example `ACTI`.
    pub record_type: String,
    /// The model the source's records use, as `MODL` holds it (`meshes\` left out).
    pub model: String,
    /// The model the copies use instead.
    pub new_model: String,
    /// Half the size of the object bounds the copies get, in game units; `None` keeps the source's bounds.
    #[serde(default)]
    pub bounds: Option<i16>,
}

/// One record the plugin overrides.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Overridden {
    /// The record type.
    pub record_type: String,
    /// Its editor id.
    pub editor_id: String,
    /// Its form id, as the source's master list counts.
    pub form_id: u32,
}

/// What [`override_plugin`] made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Made {
    /// The plugin file.
    pub bytes: Vec<u8>,
    /// The records it overrides, in the source's order.
    pub records: Vec<Overridden>,
}

const HEADER: usize = 24;
const COMPRESSED: u32 = 0x0004_0000;

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

/// A subrecord: its type and its data.
type Sub<'a> = ([u8; 4], &'a [u8]);

/// The subrecords of a record body.
fn subrecords(body: &[u8]) -> Result<Vec<Sub<'_>>, String> {
    let mut out = Vec::new();
    let mut at = 0;
    while at < body.len() {
        let head = body
            .get(at..at + 6)
            .ok_or("a subrecord header runs past its record")?;
        let kind: [u8; 4] = head[..4].try_into().expect("four bytes");
        if &kind == b"XXXX" {
            return Err("a subrecord larger than 64 KiB (XXXX) is not supported".to_owned());
        }
        let size = usize::from(u16::from_le_bytes([head[4], head[5]]));
        let data = body
            .get(at + 6..at + 6 + size)
            .ok_or("a subrecord runs past its record")?;
        out.push((kind, data));
        at += 6 + size;
    }
    Ok(out)
}

fn subrecord(out: &mut Vec<u8>, kind: [u8; 4], data: &[u8]) -> Result<(), String> {
    let size = u16::try_from(data.len()).map_err(|_| "a subrecord is too large".to_owned())?;
    out.extend_from_slice(&kind);
    out.extend_from_slice(&size.to_le_bytes());
    out.extend_from_slice(data);
    Ok(())
}

/// A model path as a key: lower case, backslashes, no `meshes\` in front, no trailing zero.
#[must_use]
pub fn model_key(path: &str) -> String {
    let p = path
        .trim_end_matches('\0')
        .replace('/', "\\")
        .to_ascii_lowercase();
    p.strip_prefix("meshes\\").map(str::to_owned).unwrap_or(p)
}

fn zstring(text: &str) -> Vec<u8> {
    let mut v = text.as_bytes().to_vec();
    v.push(0);
    v
}

struct Source<'a> {
    bytes: &'a [u8],
    masters: Vec<&'a [u8]>,
    /// The version-control stamp and form version of the source's header (bytes 16 to 24).
    stamp: [u8; 8],
}

fn read_header(bytes: &[u8]) -> Result<(Source<'_>, usize), String> {
    if bytes.get(..4) != Some(b"TES4") {
        return Err("not a plugin: it does not start with TES4".to_owned());
    }
    let size = u32_at(bytes, 4).ok_or("the plugin header is cut short")? as usize;
    let body = bytes
        .get(HEADER..HEADER + size)
        .ok_or("the plugin header is cut short")?;
    let masters = subrecords(body)?
        .into_iter()
        .filter(|(k, _)| k == b"MAST")
        .map(|(_, v)| v)
        .collect();
    let stamp = bytes[16..24].try_into().expect("eight bytes");
    Ok((
        Source {
            bytes,
            masters,
            stamp,
        },
        HEADER + size,
    ))
}

/// The copy of the record at `at` when a rule names it, with its header.
fn copy_record(
    src: &Source<'_>,
    at: usize,
    rules: &[RecordRule],
) -> Result<Option<(Vec<u8>, Overridden)>, String> {
    let b = src.bytes;
    let kind = &b[at..at + 4];
    let Some(rule_kind) = rules
        .iter()
        .map(|r| r.record_type.as_bytes())
        .find(|k| *k == kind)
    else {
        return Ok(None);
    };
    let size = u32_at(b, at + 4).ok_or("a record header is cut short")? as usize;
    let mut flags = u32_at(b, at + 8).ok_or("a record header is cut short")?;
    let form_id = u32_at(b, at + 12).ok_or("a record header is cut short")?;
    let raw = b
        .get(at + HEADER..at + HEADER + size)
        .ok_or("a record runs past the end of the plugin")?;
    let body = if flags & COMPRESSED == 0 {
        raw.to_vec()
    } else {
        let mut out = Vec::new();
        flate2::read::ZlibDecoder::new(raw.get(4..).ok_or("a compressed record is cut short")?)
            .read_to_end(&mut out)
            .map_err(|e| format!("a compressed record cannot be read: {e}"))?;
        flags &= !COMPRESSED;
        out
    };
    let subs = subrecords(&body)?;
    let model = subs
        .iter()
        .find(|(k, _)| k == b"MODL")
        .map(|(_, v)| model_key(&String::from_utf8_lossy(v)));
    let Some(rule) = rules.iter().find(|r| {
        r.record_type.as_bytes() == rule_kind && model.as_deref() == Some(&model_key(&r.model))
    }) else {
        return Ok(None);
    };
    let mut out = Vec::new();
    for (k, v) in &subs {
        match k {
            b"MODT" => {}
            b"MODL" => subrecord(&mut out, *k, &zstring(&rule.new_model))?,
            b"OBND" if rule.bounds.is_some() => {
                let r = rule.bounds.unwrap_or_default();
                let mut v = Vec::new();
                for n in [-r, -r, -r, r, r, r] {
                    v.extend_from_slice(&n.to_le_bytes());
                }
                subrecord(&mut out, *k, &v)?;
            }
            _ => subrecord(&mut out, *k, v)?,
        }
    }
    let editor_id = subs
        .iter()
        .find(|(k, _)| k == b"EDID")
        .map_or_else(String::new, |(_, v)| {
            String::from_utf8_lossy(v).trim_end_matches('\0').to_owned()
        });
    let mut record = Vec::with_capacity(HEADER + out.len());
    record.extend_from_slice(kind);
    record.extend_from_slice(
        &u32::try_from(out.len())
            .expect("a small record")
            .to_le_bytes(),
    );
    record.extend_from_slice(&flags.to_le_bytes());
    record.extend_from_slice(&form_id.to_le_bytes());
    record.extend_from_slice(&b[at + 16..at + 24]);
    record.extend_from_slice(&out);
    Ok(Some((
        record,
        Overridden {
            record_type: String::from_utf8_lossy(kind).into_owned(),
            editor_id,
            form_id,
        },
    )))
}

/// The plugin that overrides the records of `source` (the bytes of the plugin named `source_name`) that `rules`
/// name. `author` and `description` go into its header.
///
/// # Panics
///
/// Never for a plugin the game can load: its records and groups are far below 4 GiB.
///
/// # Errors
///
/// The source is not a plugin this reads, or no record of it matches a rule.
pub fn override_plugin(
    source: &[u8],
    source_name: &str,
    rules: &[RecordRule],
    author: &str,
    description: &str,
) -> Result<Made, String> {
    let (src, mut at) = read_header(source)?;
    // One group per record type, in the source's order; each keeps the stamp of the source's group.
    let mut groups: Vec<([u8; 4], [u8; 8], Vec<u8>)> = Vec::new();
    let mut records = Vec::new();
    // Walk groups depth first: (end of group, label of the top group).
    let mut stack: Vec<(usize, [u8; 4], [u8; 8])> = Vec::new();
    while at < source.len() {
        while stack.last().is_some_and(|(end, _, _)| at >= *end) {
            stack.pop();
        }
        if source.len() < at + HEADER {
            return Err("the plugin ends inside a record header".to_owned());
        }
        let size = u32_at(source, at + 4).ok_or("cut short")? as usize;
        if &source[at..at + 4] == b"GRUP" {
            if size < HEADER || at + size > source.len() {
                return Err("a group has a size that does not fit the plugin".to_owned());
            }
            let label: [u8; 4] = source[at + 8..at + 12].try_into().expect("four bytes");
            let stamp: [u8; 8] = source[at + 16..at + 24].try_into().expect("eight bytes");
            let (top_label, top_stamp) = stack.first().map_or((label, stamp), |(_, l, s)| (*l, *s));
            stack.push((at + size, top_label, top_stamp));
            at += HEADER;
            continue;
        }
        // Only records right inside a top group: a base object such as an activator, never a placed reference or
        // anything else below a cell or a world.
        if stack.len() == 1
            && let Some((record, made)) = copy_record(&src, at, rules)?
        {
            let (_, label, stamp) = stack[0];
            match groups.iter_mut().find(|(l, _, _)| *l == label) {
                Some((_, _, body)) => body.extend_from_slice(&record),
                None => groups.push((label, stamp, record)),
            }
            records.push(made);
        }
        at += HEADER + size;
    }
    if records.is_empty() {
        return Err(format!(
            "{source_name} has no record with the models this add-on replaces"
        ));
    }
    let mut header = Vec::new();
    let count = i32::try_from(records.len()).expect("few records");
    let mut hedr = 1.34_f32.to_le_bytes().to_vec();
    hedr.extend_from_slice(&count.to_le_bytes());
    hedr.extend_from_slice(&0x800_u32.to_le_bytes());
    subrecord(&mut header, *b"HEDR", &hedr)?;
    subrecord(&mut header, *b"CNAM", &zstring(author))?;
    subrecord(&mut header, *b"SNAM", &zstring(description))?;
    let own = zstring(source_name);
    for master in src.masters.iter().copied().chain([own.as_slice()]) {
        subrecord(&mut header, *b"MAST", master)?;
        subrecord(&mut header, *b"DATA", &[0; 8])?;
    }
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"TES4");
    bytes.extend_from_slice(&u32::try_from(header.len()).expect("small").to_le_bytes());
    bytes.extend_from_slice(&0_u32.to_le_bytes());
    bytes.extend_from_slice(&0_u32.to_le_bytes());
    bytes.extend_from_slice(&src.stamp);
    bytes.extend_from_slice(&header);
    for (label, stamp, body) in &groups {
        bytes.extend_from_slice(b"GRUP");
        let size = u32::try_from(HEADER + body.len()).expect("small");
        bytes.extend_from_slice(&size.to_le_bytes());
        bytes.extend_from_slice(label);
        bytes.extend_from_slice(&0_i32.to_le_bytes());
        bytes.extend_from_slice(stamp);
        bytes.extend_from_slice(body);
    }
    Ok(Made { bytes, records })
}

/// A synthetic plugin for tests: a header with `masters`, then one top group per record type holding `records`
/// (type, form id, editor id, model), each with an `OBND` and a `MODT`. Its own form ids use the index after the
/// masters.
#[doc(hidden)]
#[must_use]
pub fn synthetic_plugin(masters: &[&str], records: &[(&str, u32, &str, &str)]) -> Vec<u8> {
    let mut header = Vec::new();
    let mut hedr = 1.34_f32.to_le_bytes().to_vec();
    hedr.extend_from_slice(&u32::try_from(records.len()).unwrap().to_le_bytes());
    hedr.extend_from_slice(&0x800_u32.to_le_bytes());
    subrecord(&mut header, *b"HEDR", &hedr).unwrap();
    for m in masters {
        subrecord(&mut header, *b"MAST", &zstring(m)).unwrap();
        subrecord(&mut header, *b"DATA", &[0; 8]).unwrap();
    }
    let mut out = b"TES4".to_vec();
    out.extend_from_slice(&u32::try_from(header.len()).unwrap().to_le_bytes());
    out.extend_from_slice(&1_u32.to_le_bytes());
    out.extend_from_slice(&[0; 4]);
    out.extend_from_slice(&[0, 0, 0, 0, 15, 0, 0, 0]);
    out.extend_from_slice(&header);
    let mut kinds: Vec<&str> = Vec::new();
    for (k, ..) in records {
        if !kinds.contains(k) {
            kinds.push(k);
        }
    }
    for kind in kinds {
        let mut body = Vec::new();
        for (_, id, edid, model) in records.iter().filter(|r| r.0 == kind) {
            let mut sub = Vec::new();
            subrecord(&mut sub, *b"EDID", &zstring(edid)).unwrap();
            subrecord(
                &mut sub,
                *b"OBND",
                &[0xf6, 0xff, 0xf6, 0xff, 0xf6, 0xff, 10, 0, 10, 0, 10, 0],
            )
            .unwrap();
            subrecord(&mut sub, *b"MODL", &zstring(model)).unwrap();
            subrecord(&mut sub, *b"MODT", &[1, 2, 3, 4]).unwrap();
            body.extend_from_slice(kind.as_bytes());
            body.extend_from_slice(&u32::try_from(sub.len()).unwrap().to_le_bytes());
            body.extend_from_slice(&0_u32.to_le_bytes());
            body.extend_from_slice(&id.to_le_bytes());
            body.extend_from_slice(&[0, 0, 0, 0, 15, 0, 0, 0]);
            body.extend_from_slice(&sub);
        }
        out.extend_from_slice(b"GRUP");
        out.extend_from_slice(&u32::try_from(HEADER + body.len()).unwrap().to_le_bytes());
        out.extend_from_slice(kind.as_bytes());
        out.extend_from_slice(&0_i32.to_le_bytes());
        out.extend_from_slice(&[7, 0, 0, 0, 0, 0, 0, 0]);
        out.extend_from_slice(&body);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const GLOW: &str = "Effects\\Ambient\\FXGlowSimpFillRnd.nif";

    fn rules() -> Vec<RecordRule> {
        vec![RecordRule {
            record_type: "ACTI".to_owned(),
            model: "effects\\ambient\\fxglowsimpfillrnd.nif".to_owned(),
            new_model: "raul\\effects\\raulsmallglow01.nif".to_owned(),
            bounds: Some(36),
        }]
    }

    fn source() -> Vec<u8> {
        synthetic_plugin(
            &["FalloutNV.esm", "Fallout3.esm"],
            &[
                ("ACTI", 0x0200_0801, "LampGlow", GLOW),
                ("ACTI", 0x0200_0802, "OtherActivator", "clutter\\box.nif"),
                (
                    "STAT",
                    0x0200_0803,
                    "LampPost",
                    "meshes/effects/ambient/fxglowsimpfillrnd.nif",
                ),
                (
                    "ACTI",
                    0x0200_0804,
                    "LampGlowBig",
                    "meshes/Effects/Ambient/fxglowsimpfillrnd.nif",
                ),
            ],
        )
    }

    fn header_strings(bytes: &[u8], kind: [u8; 4]) -> Vec<String> {
        let size = u32_at(bytes, 4).unwrap() as usize;
        subrecords(&bytes[HEADER..HEADER + size])
            .unwrap()
            .into_iter()
            .filter(|(k, _)| *k == kind)
            .map(|(_, v)| String::from_utf8_lossy(v).trim_end_matches('\0').to_owned())
            .collect()
    }

    #[test]
    fn only_records_of_the_rule_type_and_model_are_copied_with_their_form_ids() {
        let made =
            override_plugin(&source(), "Street Lights.esm", &rules(), "raul", "test").unwrap();
        let ids: Vec<(String, u32)> = made
            .records
            .iter()
            .map(|r| (r.editor_id.clone(), r.form_id))
            .collect();
        assert_eq!(
            ids,
            [
                ("LampGlow".to_owned(), 0x0200_0801),
                ("LampGlowBig".to_owned(), 0x0200_0804)
            ]
        );
        assert_eq!(
            header_strings(&made.bytes, *b"MAST"),
            ["FalloutNV.esm", "Fallout3.esm", "Street Lights.esm"],
            "the source's masters, then the source: index 2 is the source in both"
        );
    }

    #[test]
    fn the_copies_have_the_new_model_no_texture_hashes_and_the_rule_bounds() {
        let made =
            override_plugin(&source(), "Street Lights.esm", &rules(), "raul", "test").unwrap();
        let (_, end) = read_header(&made.bytes).unwrap();
        assert_eq!(&made.bytes[end..end + 4], b"GRUP");
        assert_eq!(&made.bytes[end + 8..end + 12], b"ACTI");
        assert_eq!(
            &made.bytes[end + 16..end + 24],
            &[7, 0, 0, 0, 0, 0, 0, 0],
            "the group stamp is kept"
        );
        let group = u32_at(&made.bytes, end + 4).unwrap() as usize;
        assert_eq!(end + group, made.bytes.len());
        let first = end + HEADER;
        let size = u32_at(&made.bytes, first + 4).unwrap() as usize;
        let subs = subrecords(&made.bytes[first + HEADER..first + HEADER + size]).unwrap();
        let kinds: Vec<&[u8]> = subs.iter().map(|(k, _)| &k[..]).collect();
        assert_eq!(kinds, [&b"EDID"[..], b"OBND", b"MODL"]);
        assert_eq!(subs[2].1, b"raul\\effects\\raulsmallglow01.nif\0");
        let bounds: Vec<i16> = subs[1]
            .1
            .chunks(2)
            .map(|c| i16::from_le_bytes([c[0], c[1]]))
            .collect();
        assert_eq!(bounds, [-36, -36, -36, 36, 36, 36]);
        let hedr = &header_strings(&made.bytes, *b"CNAM");
        assert_eq!(hedr, &["raul"]);
    }

    #[test]
    fn a_source_without_those_records_is_refused_and_a_broken_one_too() {
        let other = synthetic_plugin(&["FalloutNV.esm"], &[("ACTI", 0x0100_0801, "X", "a.nif")]);
        assert!(
            override_plugin(&other, "Other.esp", &rules(), "raul", "t")
                .unwrap_err()
                .contains("no record")
        );
        assert!(override_plugin(b"not a plugin", "x.esp", &rules(), "raul", "t").is_err());
        let mut cut = source();
        cut.truncate(cut.len() - 10);
        assert!(override_plugin(&cut, "x.esp", &rules(), "raul", "t").is_err());
    }

    #[test]
    fn a_compressed_record_is_read_and_written_uncompressed() {
        let plain = source();
        // Compress the first ACTI record of the first group.
        let (_, end) = read_header(&plain).unwrap();
        let first = end + HEADER;
        let size = u32_at(&plain, first + 4).unwrap() as usize;
        let body = &plain[first + HEADER..first + HEADER + size];
        let mut z = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        std::io::Write::write_all(&mut z, body).unwrap();
        let mut packed = u32::try_from(body.len()).unwrap().to_le_bytes().to_vec();
        packed.extend_from_slice(&z.finish().unwrap());
        let mut record = plain[first..first + 8].to_vec();
        record[4..8].copy_from_slice(&u32::try_from(packed.len()).unwrap().to_le_bytes());
        record.extend_from_slice(&COMPRESSED.to_le_bytes());
        record.extend_from_slice(&plain[first + 12..first + 24]);
        record.extend_from_slice(&packed);
        let mut bytes = plain[..first].to_vec();
        bytes.extend_from_slice(&record);
        bytes.extend_from_slice(&plain[first + HEADER + size..]);
        let group = u32_at(&plain, end + 4).unwrap() as usize - size + packed.len();
        bytes[end + 4..end + 8].copy_from_slice(&u32::try_from(group).unwrap().to_le_bytes());
        let a = override_plugin(&plain, "S.esm", &rules(), "raul", "t").unwrap();
        let b = override_plugin(&bytes, "S.esm", &rules(), "raul", "t").unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn a_matching_record_below_a_nested_group_is_not_copied() {
        // The source's ACTI group, then a top group whose child group holds a record of the same type and model.
        let plain = source();
        let (_, end) = read_header(&plain).unwrap();
        let acti_group = u32_at(&plain, end + 4).unwrap() as usize;
        let record = &plain
            [end + HEADER..end + HEADER + 24 + u32_at(&plain, end + HEADER + 4).unwrap() as usize];
        let group = |label: &[u8], kind: i32, body: &[u8]| {
            let mut g = b"GRUP".to_vec();
            g.extend_from_slice(&u32::try_from(HEADER + body.len()).unwrap().to_le_bytes());
            g.extend_from_slice(label);
            g.extend_from_slice(&kind.to_le_bytes());
            g.extend_from_slice(&[0; 8]);
            g.extend_from_slice(body);
            g
        };
        let nested = group(b"CELL", 0, &group(&[1, 0, 0, 0], 6, record));
        let mut bytes = plain[..end + acti_group].to_vec();
        bytes.extend_from_slice(&nested);
        let made = override_plugin(&bytes, "S.esm", &rules(), "raul", "t").unwrap();
        assert_eq!(made.records.len(), 2, "only the two in the ACTI group");
    }

    #[test]
    fn model_keys_ignore_case_slashes_and_the_meshes_folder() {
        assert_eq!(model_key("Meshes/Effects/A.NIF\0"), "effects\\a.nif");
        assert_eq!(model_key("effects\\a.nif"), "effects\\a.nif");
    }
}
