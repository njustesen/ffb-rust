/// 1:1 translation of com.fumbbl.ffb.server.admin.UtilBackup.
/// Static utility for saving and loading game state backups (file, DB, S3).
///
/// Java gzips the JSON payload (`UtilJson.gzip`/`UtilJson.gunzip`) and additionally falls back
/// to a DB query and an S3 bucket if the file-system backup is missing. No gzip crate, DB layer,
/// or S3 client is wired into this workspace yet, so this ports the pure path-calculation logic
/// exactly and does file-system-only save/load with plain (uncompressed) JSON.
use ffb_model::model::game::Game;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct UtilBackup;

impl UtilBackup {
    /// Builds a hierarchical directory path from the game id digits, e.g. game id `1234567`
    /// with extension `"json"` becomes `"1/2/3/4/1234567.json"`.
    pub fn calculate_folder_path_for_game(game_id_string: &str, backup_extension: &str) -> String {
        let chars: Vec<char> = game_id_string.chars().collect();
        let mut path = String::new();
        let mut index = 0usize;
        for i in (4..=7).rev() {
            if chars.len() < i {
                path.push('0');
            } else {
                path.push(chars[index]);
                index += 1;
            }
            path.push('/');
        }
        path.push_str(game_id_string);
        path.push('.');
        path.push_str(backup_extension);
        path
    }

    fn find_backup_file(backup_dir: &str, game_id: i64, backup_extension: &str) -> Option<PathBuf> {
        if backup_dir.is_empty() || game_id <= 0 {
            return None;
        }
        let game_id_string = game_id.to_string();
        let relative = Self::calculate_folder_path_for_game(&game_id_string, backup_extension);
        Some(Path::new(backup_dir).join(relative))
    }

    /// Serializes `game_state` to JSON and writes it to the backup file under `backup_dir`.
    /// Generic over the serialized payload so callers can pass either a full `Game` or (in
    /// tests) a lightweight stand-in without needing to construct a complete `Team`/`Game`.
    pub fn save<T: Serialize>(
        game_state: &T,
        backup_dir: &str,
        backup_extension: &str,
        game_id: i64,
    ) -> Result<bool, String> {
        let backup_file = match Self::find_backup_file(backup_dir, game_id, backup_extension) {
            Some(path) => path,
            None => return Ok(false),
        };
        if let Some(parent) = backup_file.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_vec(game_state).map_err(|e| e.to_string())?;
        let mut file = std::fs::File::create(&backup_file).map_err(|e| e.to_string())?;
        file.write_all(&json).map_err(|e| e.to_string())?;
        Ok(true)
    }

    /// Reads the raw backup bytes from the backup file (plain JSON, not gzip — see module docs).
    pub fn load_as_gzip(backup_dir: &str, backup_extension: &str, game_id: i64) -> Result<Option<Vec<u8>>, String> {
        let backup_file = match Self::find_backup_file(backup_dir, game_id, backup_extension) {
            Some(path) => path,
            None => return Ok(None),
        };
        if !backup_file.exists() {
            return Ok(None);
        }
        std::fs::read(&backup_file).map(Some).map_err(|e| e.to_string())
    }

    /// Loads a `Game` from the file-system backup. Java additionally falls back to a DB query
    /// and an S3 bucket lookup when the file is missing; neither exists in this crate yet.
    pub fn load_game_state(backup_dir: &str, backup_extension: &str, game_id: i64) -> Result<Option<Game>, String> {
        Self::load_as::<Game>(backup_dir, backup_extension, game_id)
    }

    /// Generic counterpart to [`Self::load_game_state`] used by tests.
    pub fn load_as<T: DeserializeOwned>(
        backup_dir: &str,
        backup_extension: &str,
        game_id: i64,
    ) -> Result<Option<T>, String> {
        match Self::load_as_gzip(backup_dir, backup_extension, game_id)? {
            Some(bytes) => {
                let value: T = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct FakeGameState {
        id: i64,
        half: i32,
    }

    #[test]
    fn calculate_folder_path_for_seven_digit_id() {
        let path = UtilBackup::calculate_folder_path_for_game("1234567", "json");
        assert_eq!(path, "1/2/3/4/1234567.json");
    }

    #[test]
    fn calculate_folder_path_pads_short_ids_with_zeros() {
        let path = UtilBackup::calculate_folder_path_for_game("42", "json");
        assert_eq!(path, "0/0/0/0/42.json");
    }

    #[test]
    fn save_and_load_round_trip() {
        let dir = std::env::temp_dir().join(format!("ffb_backup_test_{}", std::process::id()));
        let dir_str = dir.to_string_lossy().to_string();
        let state = FakeGameState { id: 999, half: 2 };

        let saved = UtilBackup::save(&state, &dir_str, "json", 999).unwrap();
        assert!(saved);

        let loaded: FakeGameState = UtilBackup::load_as(&dir_str, "json", 999).unwrap().unwrap();
        assert_eq!(loaded, state);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_game_state_missing_file_returns_none() {
        let dir = std::env::temp_dir().join("ffb_backup_test_missing");
        let result = UtilBackup::load_game_state(&dir.to_string_lossy(), "json", 123456).unwrap();
        assert!(result.is_none());
    }
}
