/// 1:1 translation of com.fumbbl.ffb.server.handler.RedeployHandler.
///
/// Java resolves the redeploy file path / default branch / exit code from
/// `server.getProperty(IServerProperty.SERVER_REDEPLOY_*)`. The Rust MVP has
/// no server-properties/ini-file system (`FantasyFootballServer.getProperty`)
/// yet, so callers pass those three values directly instead.
pub struct RedeployHandler;

impl RedeployHandler {
    pub fn new() -> Self {
        Self
    }

    /// Java: `redeploy(FantasyFootballServer, String)` — triggers a server
    /// redeploy on the given branch.
    ///
    /// Resolves the branch to use, writes it to the redeploy file, and exits
    /// the process with the configured exit code so the surrounding
    /// supervisor script can restart the server on the new branch. Java logs
    /// I/O failures via `server.getDebugLog().logWithOutGameId(e)`; the Rust
    /// MVP has no `DebugLog`, so failures are logged via the `log` crate
    /// instead.
    pub fn redeploy(&self, branch: Option<&str>, default_branch: &str, redeploy_file: &str, exit_code: i32) {
        let chosen = self.resolve_branch(branch, default_branch);
        match self.write_redeploy_file(redeploy_file, chosen) {
            Ok(()) => std::process::exit(exit_code),
            Err(e) => log::error!("RedeployHandler: failed to write redeploy file {}: {}", redeploy_file, e),
        }
    }

    /// Java: `if (!StringTool.isProvided(pBranch) || !isValidBranch(pBranch)) { pBranch = ...; }`.
    fn resolve_branch<'a>(&self, branch: Option<&'a str>, default_branch: &'a str) -> &'a str {
        match branch {
            Some(b) if !b.is_empty() && self.is_valid_branch(b) => b,
            _ => default_branch,
        }
    }

    /// Java: `Files.write(Paths.get(file.toURI()), branch.getBytes(UTF_8), TRUNCATE_EXISTING)`
    /// (preceded by `file.createNewFile()` / `file.setWritable(true)`, both no-ops on Rust's
    /// truncating `fs::write`, which creates the file if it does not exist).
    fn write_redeploy_file(&self, path: &str, branch: &str) -> std::io::Result<()> {
        std::fs::write(path, branch.as_bytes())
    }

    /// Java: `isValidBranch(String)` — `BRANCH_PATTERN.matcher(branch).matches()`
    /// where `BRANCH_PATTERN = Pattern.compile("[-_a-zA-Z0-9]+")`.
    fn is_valid_branch(&self, branch: &str) -> bool {
        !branch.is_empty()
            && branch
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    }
}

impl Default for RedeployHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let _ = RedeployHandler::new();
    }

    #[test]
    fn is_valid_branch_accepts_alphanumeric_dash_underscore() {
        let h = RedeployHandler::new();
        assert!(h.is_valid_branch("feature-branch_1"));
        assert!(h.is_valid_branch("main"));
    }

    #[test]
    fn is_valid_branch_rejects_invalid_characters() {
        let h = RedeployHandler::new();
        assert!(!h.is_valid_branch("../etc/passwd"));
        assert!(!h.is_valid_branch("branch with spaces"));
        assert!(!h.is_valid_branch(""));
    }

    #[test]
    fn resolve_branch_uses_provided_valid_branch() {
        let h = RedeployHandler::new();
        assert_eq!(h.resolve_branch(Some("feature-1"), "main"), "feature-1");
    }

    #[test]
    fn resolve_branch_falls_back_to_default_when_invalid() {
        let h = RedeployHandler::new();
        assert_eq!(h.resolve_branch(Some("bad branch!"), "main"), "main");
    }

    #[test]
    fn resolve_branch_falls_back_to_default_when_none() {
        let h = RedeployHandler::new();
        assert_eq!(h.resolve_branch(None, "main"), "main");
    }

    #[test]
    fn write_redeploy_file_writes_branch_contents() {
        let h = RedeployHandler::new();
        let path = std::env::temp_dir().join(format!("ffb-redeploy-test-{}.txt", std::process::id()));
        let path_str = path.to_str().unwrap();
        h.write_redeploy_file(path_str, "my-branch").unwrap();
        let contents = std::fs::read_to_string(&path).unwrap();
        assert_eq!(contents, "my-branch");
        let _ = std::fs::remove_file(&path);
    }
}
