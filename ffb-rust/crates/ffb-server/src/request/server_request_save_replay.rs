/// 1:1 translation of com.fumbbl.ffb.server.request.ServerRequestSaveReplay.
/// Gets the GameState from cache or DB, calls UtilBackup.save(), then queues deletion.
pub struct ServerRequestSaveReplay {
    pub game_id: i64,
    request_url: String,
}

impl ServerRequestSaveReplay {
    pub fn new(game_id: i64) -> Self {
        Self { game_id, request_url: String::new() }
    }

    pub fn get_game_id(&self) -> i64 {
        self.game_id
    }

    pub fn get_request_url(&self) -> &str {
        &self.request_url
    }

    pub fn set_request_url(&mut self, url: String) {
        self.request_url = url;
    }

    /// Backs up `game_state_payload` (serialized to JSON, see
    /// `crate::admin::util_backup::UtilBackup::save`) under `backup_dir`. On success, Java also
    /// flips the game's status to `BACKUPED`, queues DB updates, and enqueues a
    /// `ServerRequestLoadReplay(DELETE_GAME)` follow-up request; that cache/DB/queue plumbing
    /// does not exist here, so the caller is expected to do that with the returned `bool`.
    pub fn process<T: serde::Serialize>(
        &self,
        game_state_payload: &T,
        backup_dir: &str,
        backup_extension: &str,
    ) -> Result<bool, String> {
        crate::admin::util_backup::UtilBackup::save(game_state_payload, backup_dir, backup_extension, self.game_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(serde::Serialize)]
    struct FakeGameState {
        half: i32,
    }

    #[test]
    fn construct() {
        let r = ServerRequestSaveReplay::new(42);
        assert_eq!(r.get_game_id(), 42);
    }

    #[test]
    fn process_saves_backup_file() {
        let dir = std::env::temp_dir().join(format!("ffb_save_replay_test_{}", std::process::id()));
        let dir_str = dir.to_string_lossy().to_string();
        let r = ServerRequestSaveReplay::new(42);
        let backed_up = r.process(&FakeGameState { half: 1 }, &dir_str, "json").unwrap();
        assert!(backed_up);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn process_with_empty_backup_dir_does_nothing() {
        let r = ServerRequestSaveReplay::new(42);
        let backed_up = r.process(&FakeGameState { half: 1 }, "", "json").unwrap();
        assert!(!backed_up);
    }
}
