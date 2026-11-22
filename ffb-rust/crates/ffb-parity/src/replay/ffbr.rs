//! Reading FUMBBL replay files (`.ffbr`) and the input list, 1:1 with the harness
//! (`FumbblReplayParser.parseFile` / `runPerReplay`, BACKLOG §O).

use flate2::read::GzDecoder;
use serde_json::Value;
use std::collections::BTreeMap;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

/// Java `UtilJson.gunzip(Files.readAllBytes(file))`: GZIP, UTF-8 JSON text.
pub fn read_ffbr(path: &Path) -> Result<Value, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut text = String::new();
    GzDecoder::new(BufReader::new(file))
        .read_to_string(&mut text)
        .map_err(|e| format!("{}: gunzip: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("{}: json: {e}", path.display()))
}

/// The replay id of a `.ffbr` file name (Java `getName().replaceFirst("\\.ffbr$", "")`).
pub fn replay_id(path: &Path) -> String {
    let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    name.strip_suffix(".ffbr").map(str::to_string).unwrap_or(name)
}

/// Java `runPerReplay`'s input list: every directory contributes its `*.ffbr` files sorted by
/// name, files are taken as given, and the result is deduplicated by replay id, FIRST wins, in
/// input order (Java `LinkedHashMap.putIfAbsent`). Returned in that order.
pub fn collect_inputs(inputs: &[PathBuf]) -> Vec<(String, PathBuf)> {
    let mut seen: BTreeMap<String, ()> = BTreeMap::new();
    let mut out = Vec::new();
    for input in inputs {
        let mut files: Vec<PathBuf> = Vec::new();
        if input.is_dir() {
            if let Ok(rd) = std::fs::read_dir(input) {
                for entry in rd.flatten() {
                    let p = entry.path();
                    let is_ffbr = p.file_name().map(|n| n.to_string_lossy().ends_with(".ffbr")).unwrap_or(false);
                    if p.is_file() && is_ffbr {
                        files.push(p);
                    }
                }
            }
            // Java: files.sort(Comparator.comparing(File::getName)) -- String order of the name.
            files.sort_by(|a, b| {
                let an = a.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                let bn = b.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                super::canonical::java_string_cmp(&an, &bn)
            });
        } else {
            files.push(input.clone());
        }
        for f in files {
            let id = replay_id(&f);
            if seen.insert(id.clone(), ()).is_none() {
                out.push((id, f));
            }
        }
    }
    out
}

/// The `commandArray` of a replay root (Java `root.get("commandArray").asArray()`).
pub fn command_array(root: &Value) -> Option<&Vec<Value>> {
    root.get("commandArray").and_then(Value::as_array)
}

/// `netCommandId` of one command (Java `cmdObj.getString("netCommandId", "")`).
pub fn net_command_id(cmd: &Value) -> &str {
    cmd.get("netCommandId").and_then(Value::as_str).unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replay_id_strips_only_the_suffix() {
        assert_eq!(replay_id(Path::new("D:/x/1965890.ffbr")), "1965890");
        assert_eq!(replay_id(Path::new("D:/x/a.ffbr.bak")), "a.ffbr.bak");
    }

    #[test]
    fn inputs_deduplicate_first_wins() {
        let dir = std::env::temp_dir().join(format!("ffb_replay_inputs_{}", std::process::id()));
        let sub = dir.join("sub");
        std::fs::create_dir_all(&sub).unwrap();
        for p in [dir.join("2.ffbr"), dir.join("10.ffbr"), sub.join("2.ffbr"), sub.join("3.ffbr"), dir.join("x.txt")] {
            std::fs::write(&p, b"").unwrap();
        }
        let got = collect_inputs(&[dir.clone(), sub.clone()]);
        let ids: Vec<&str> = got.iter().map(|(id, _)| id.as_str()).collect();
        // name order "10.ffbr" < "2.ffbr" (String order, not numeric); sub/2 is a duplicate
        assert_eq!(ids, vec!["10", "2", "3"]);
        assert_eq!(got[1].1, dir.join("2.ffbr"));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn reads_gzipped_json() {
        use flate2::write::GzEncoder;
        use std::io::Write;
        let path = std::env::temp_dir().join(format!("ffb_replay_read_{}.ffbr", std::process::id()));
        let mut enc = GzEncoder::new(Vec::new(), flate2::Compression::default());
        enc.write_all("{\"commandArray\":[{\"netCommandId\":\"serverGameState\"}],\"n\":\"\u{e9}\"}".as_bytes()).unwrap();
        std::fs::write(&path, enc.finish().unwrap()).unwrap();
        let root = read_ffbr(&path).unwrap();
        let cmds = command_array(&root).unwrap();
        assert_eq!(net_command_id(&cmds[0]), "serverGameState");
        assert_eq!(root["n"], "\u{e9}");
        std::fs::remove_file(&path).unwrap();
    }
}
