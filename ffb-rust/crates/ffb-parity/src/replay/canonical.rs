//! The canonical state text of BACKLOG §O: a 1:1 port of the harness's
//! `FumbblReplayParser.canonicalCopy` / `canonicalGameJson` / `sha256Hex`
//! (`ffb-java/ffb/ffb-ai/src/main/java/com/fumbbl/ffb/ai/simulation/FumbblReplayParser.java`)
//! plus minimal-json 0.9.5's `JsonValue.toString()` printing (`JsonWriter`).
//!
//! Rules (BACKLOG §O "canonical state form"):
//! 1. every object's keys sorted, Java `String` order (UTF-16 code units), stable;
//! 2. arrays whose path matches [`UNORDERED_ARRAY`] sorted by each element's canonical text
//!    (Java `String` order), after 1+2 were applied to the element itself;
//! 3. every other array keeps its order;
//! 4. printed like minimal-json (no whitespace, its string escaping); sha256 of the UTF-8 bytes.

use regex::Regex;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::sync::OnceLock;

/// `FumbblReplayParser.UNORDERED_ARRAY`, the alternatives verbatim. Java uses
/// `matcher(path).matches()` (whole-string match), hence the anchors.
const UNORDERED_ARRAY_ALTERNATIVES: [&str; 12] = [
    r"team(Home|Away)\.specialRules",
    r"team(Home|Away)\.roster\.positionArray",
    r"team(Home|Away)\.roster\.positionArray\[\]\.skillCategories(Normal|Double)",
    r"team(Home|Away)\.playerArray\[\](\.player)?\.temporary(Skills|Modifiers|Properties)Map\..+",
    r"team(Home|Away)\.playerArray\[\](\.player)?\.usedSkills",
    r"turnData(Home|Away)\.inducementSet\.(inducementArray|cardsAvailable|cardsActive|cardsDeactivated|starPlayerPositionIds|prayers)",
    r"fieldModel\.(pushbackSquareArray|moveSquareArray|trackNumberArray|diceDecorationArray|fieldMarkerArray|playerMarkerArray)",
    r"fieldModel\.playerDataArray\[\]\.(cards|cardEffects)",
    r"fieldModel\.targetSelectionState\.usedSkills",
    r"actingPlayer\.usedSkills",
    r"gameOptions\.gameOptionArray",
    r"dialogParameter\.nrOfCardsPerType",
];

fn unordered_array() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        let alternatives = UNORDERED_ARRAY_ALTERNATIVES.join("|");
        Regex::new(&format!("^(?:{alternatives})$")).expect("UNORDERED_ARRAY compiles")
    })
}

/// Java `FumbblReplayParser.UNORDERED_ARRAY.matcher(path).matches()`.
pub fn is_unordered_array(path: &str) -> bool {
    unordered_array().is_match(path)
}

/// Java `String.compareTo`: lexicographic over UTF-16 code units.
pub fn java_string_cmp(a: &str, b: &str) -> Ordering {
    a.encode_utf16().cmp(b.encode_utf16())
}

/// A JSON value in the order minimal-json would print it. `serde_json::Value`'s map does not
/// keep member order, so the canonical form is built into this tree and printed from it.
#[derive(Debug, Clone, PartialEq)]
pub enum CanonicalJson {
    Null,
    Bool(bool),
    /// The number's text, exactly as minimal-json prints it.
    Number(String),
    String(String),
    Array(Vec<CanonicalJson>),
    Object(Vec<(String, CanonicalJson)>),
}

impl CanonicalJson {
    /// minimal-json `JsonValue.toString()` (= `JsonWriter`, no formatting).
    pub fn to_json_string(&self) -> String {
        let mut out = String::new();
        self.write(&mut out);
        out
    }

    fn write(&self, out: &mut String) {
        match self {
            CanonicalJson::Null => out.push_str("null"),
            CanonicalJson::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            CanonicalJson::Number(text) => out.push_str(text),
            CanonicalJson::String(s) => write_json_string(s, out),
            CanonicalJson::Array(elements) => {
                out.push('[');
                for (i, e) in elements.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    e.write(out);
                }
                out.push(']');
            }
            CanonicalJson::Object(members) => {
                out.push('{');
                for (i, (name, value)) in members.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    write_json_string(name, out);
                    out.push(':');
                    value.write(out);
                }
                out.push('}');
            }
        }
    }
}

const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

/// minimal-json `JsonWriter.writeJsonString`: escapes `"` and `\`, `\n` `\r` `\t` by name, every
/// other control character below 0x20 as `\u00xx` (lower-case hex), and U+2028 / U+2029 (line
/// endings in JavaScript) as ` ` / ` `. Everything else, non-ASCII included, verbatim.
pub fn write_json_string(s: &str, out: &mut String) {
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            c if (c as u32) < 0x20 => {
                let v = c as u32 as usize;
                out.push_str("\\u00");
                out.push(HEX_DIGITS[(v >> 4) & 0xf] as char);
                out.push(HEX_DIGITS[v & 0xf] as char);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

/// The number text minimal-json prints. Game JSON only carries Java ints and longs
/// (`JsonValue.valueOf(int|long)` = `Integer/Long.toString`), which `serde_json` reads back as
/// i64/u64 and prints the same way. A fraction would need Java's `Double.toString` rules, which
/// no state has needed yet (BACKLOG §O: "if a float ever appears, compare Java's form and record
/// the rule here"), so it is an error rather than a guess.
fn number_text(n: &serde_json::Number) -> Result<String, String> {
    if let Some(i) = n.as_i64() {
        Ok(i.to_string())
    } else if let Some(u) = n.as_u64() {
        Ok(u.to_string())
    } else {
        Err(format!("non-integer number {n} in a state (no canonical rule for floats yet)"))
    }
}

/// Java `canonicalCopy(value, path)`; `path = None` is Java's `null` path (keys sorted only).
pub fn canonical_copy(value: &Value, path: Option<&str>) -> Result<CanonicalJson, String> {
    Ok(match value {
        Value::Null => CanonicalJson::Null,
        Value::Bool(b) => CanonicalJson::Bool(*b),
        Value::Number(n) => CanonicalJson::Number(number_text(n)?),
        Value::String(s) => CanonicalJson::String(s.clone()),
        Value::Object(map) => {
            let mut names: Vec<&String> = map.keys().collect();
            names.sort_by(|a, b| java_string_cmp(a, b));
            let mut members = Vec::with_capacity(names.len());
            for name in names {
                let sub = path.map(|p| if p.is_empty() { name.clone() } else { format!("{p}.{name}") });
                members.push((name.clone(), canonical_copy(&map[name], sub.as_deref())?));
            }
            CanonicalJson::Object(members)
        }
        Value::Array(elements) => {
            let sub = path.map(|p| format!("{p}[]"));
            let mut out = Vec::with_capacity(elements.len());
            for e in elements {
                out.push(canonical_copy(e, sub.as_deref())?);
            }
            if path.is_some_and(is_unordered_array) {
                // Java: elements.sort(Comparator.comparing(JsonValue::toString)) -- stable.
                let mut keyed: Vec<(String, CanonicalJson)> =
                    out.into_iter().map(|e| (e.to_json_string(), e)).collect();
                keyed.sort_by(|a, b| java_string_cmp(&a.0, &b.0));
                out = keyed.into_iter().map(|(_, e)| e).collect();
            }
            CanonicalJson::Array(out)
        }
    })
}

/// Java `canonicalGameJson(game)` given `game.toJsonValue()`.
pub fn canonical_game_json(game_json: &Value) -> Result<String, String> {
    Ok(canonical_copy(game_json, Some(""))?.to_json_string())
}

/// Java `sha256Hex(data)`: lower-case hex.
pub fn sha256_hex(data: &[u8]) -> String {
    hex::encode(Sha256::digest(data))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn escaping_matches_minimal_json() {
        // Expected text printed by minimal-json 0.9.5 (Json.value(s).toString()) for chars
        // 0x00..0x7f followed by U+00E9 U+2028 U+2029 U+007F U+1F600 (probe, 2026-11-22).
        let mut s = String::new();
        for c in 0u32..0x80 {
            s.push(char::from_u32(c).unwrap());
        }
        s.push('\u{e9}');
        s.push('\u{2028}');
        s.push('\u{2029}');
        s.push('\u{7f}');
        s.push('\u{1F600}');
        let mut out = String::new();
        write_json_string(&s, &mut out);
        let expected = concat!(
            "\"\\u0000\\u0001\\u0002\\u0003\\u0004\\u0005\\u0006\\u0007\\u0008\\t\\n\\u000b\\u000c\\r",
            "\\u000e\\u000f\\u0010\\u0011\\u0012\\u0013\\u0014\\u0015\\u0016\\u0017\\u0018\\u0019",
            "\\u001a\\u001b\\u001c\\u001d\\u001e\\u001f !\\\"#$%&'()*+,-./0123456789:;<=>?@",
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\\\]^_`abcdefghijklmnopqrstuvwxyz{|}~\u{7f}",
            "\u{e9}\\u2028\\u2029\u{7f}\u{1F600}\""
        );
        assert_eq!(out, expected);
    }

    #[test]
    fn java_string_order_is_utf16() {
        // U+FF61 sorts AFTER U+1F600 in UTF-8/char order but BEFORE it in UTF-16 (surrogates
        // 0xD83D < 0xFF61).
        assert_eq!(java_string_cmp("\u{1F600}", "\u{FF61}"), Ordering::Less);
        assert_eq!("\u{1F600}".cmp("\u{FF61}"), Ordering::Greater);
        assert_eq!(java_string_cmp("a", "b"), Ordering::Less);
        assert_eq!(java_string_cmp("ab", "a"), Ordering::Greater);
    }

    #[test]
    fn unordered_paths_match_whole_path_only() {
        assert!(is_unordered_array("gameOptions.gameOptionArray"));
        assert!(is_unordered_array("teamHome.playerArray[].temporarySkillsMap.Block"));
        assert!(is_unordered_array("teamAway.playerArray[].player.usedSkills"));
        assert!(is_unordered_array("teamAway.roster.positionArray[].skillCategoriesDouble"));
        assert!(is_unordered_array("turnDataHome.inducementSet.prayers"));
        assert!(!is_unordered_array("teamHome.playerArray"));
        assert!(!is_unordered_array("x.gameOptions.gameOptionArray"));
        assert!(!is_unordered_array("teamHome.roster.positionArray[].skillArray"));
        assert!(!is_unordered_array("fieldModel.playerDataArray"));
    }

    #[test]
    fn keys_sorted_and_only_unordered_arrays_sorted() {
        let game = json!({
            "half": 2,
            "gameOptions": {"gameOptionArray": [{"id": "b"}, {"id": "a"}]},
            "fieldModel": {"bloodspotArray": [3, 1, 2], "trackNumberArray": [[2, 1], [1, 9]]},
            "big": 10000000000_i64,
            "neg": -1
        });
        let text = canonical_game_json(&game).unwrap();
        assert_eq!(
            text,
            concat!(
                "{\"big\":10000000000,\"fieldModel\":{\"bloodspotArray\":[3,1,2],",
                "\"trackNumberArray\":[[1,9],[2,1]]},\"gameOptions\":{\"gameOptionArray\":",
                "[{\"id\":\"a\"},{\"id\":\"b\"}]},\"half\":2,\"neg\":-1}"
            )
        );
        assert_eq!(sha256_hex(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    }

    #[test]
    fn floats_are_refused() {
        assert!(canonical_game_json(&json!({"x": 1.5})).is_err());
    }
}
