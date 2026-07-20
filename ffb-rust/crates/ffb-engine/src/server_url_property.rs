use std::collections::HashMap;

use crate::i_server_property::IServerProperty;

/// URL property keys for server HTTP endpoints — 1:1 translation of Java ServerUrlProperty enum.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServerUrlProperty {
    ADMIN_URL_BACKUP,
    ADMIN_URL_BLOCK,
    ADMIN_URL_CACHE,
    ADMIN_URL_CHALLENGE,
    ADMIN_URL_CLOSE,
    ADMIN_URL_CONCEDE,
    ADMIN_URL_DELETE,
    ADMIN_URL_FORCELOG,
    ADMIN_URL_LIST_ID,
    ADMIN_URL_LIST_STATUS,
    ADMIN_URL_LOGLEVEL,
    ADMIN_URL_LOGFILE,
    ADMIN_URL_MESSAGE,
    ADMIN_URL_REFRESH,
    ADMIN_URL_SCHEDULE,
    ADMIN_URL_SHUTDOWN,
    ADMIN_URL_STATS,
    ADMIN_URL_UNBLOCK,
    ADMIN_URL_UPLOAD,
    ADMIN_URL_PORTRAIT,
    ADMIN_URL_PURGE_TEST,
    ADMIN_URL_REDEPLOY,
    GAMESTATE_URL_CHALLENGE,
    GAMESTATE_URL_BEHAVIORS,
    GAMESTATE_URL_GET,
    GAMESTATE_URL_RESET,
    GAMESTATE_URL_RESULT,
    GAMESTATE_URL_SET,
    BACKUP_URL_CHALLENGE,
    BACKUP_URL_LOAD,
    BACKUP_URL_SAVE,
    FUMBBL_AUTH_CHALLENGE,
    FUMBBL_AUTH_RESPONSE,
    FUMBBL_TEAMS,
    FUMBBL_TEAM,
    FUMBBL_ROSTER,
    FUMBBL_ROSTER_TEAM,
    FUMBBL_GAMESTATE_CHECK,
    FUMBBL_GAMESTATE_CREATE,
    FUMBBL_GAMESTATE_RESUME,
    FUMBBL_GAMESTATE_UPDATE,
    FUMBBL_GAMESTATE_REMOVE,
    FUMBBL_GAMESTATE_OPTIONS,
    FUMBBL_RESULT,
    FUMBBL_TALK,
    FUMBBL_NAMEGENERATOR_BASE,
    FUMBBL_PLAYER_MARKINGS,
}

impl ServerUrlProperty {
    fn base_key(&self) -> &'static str {
        match self {
            ServerUrlProperty::FUMBBL_AUTH_CHALLENGE
            | ServerUrlProperty::FUMBBL_AUTH_RESPONSE
            | ServerUrlProperty::FUMBBL_TEAMS
            | ServerUrlProperty::FUMBBL_TEAM
            | ServerUrlProperty::FUMBBL_ROSTER
            | ServerUrlProperty::FUMBBL_ROSTER_TEAM
            | ServerUrlProperty::FUMBBL_GAMESTATE_CHECK
            | ServerUrlProperty::FUMBBL_GAMESTATE_CREATE
            | ServerUrlProperty::FUMBBL_GAMESTATE_RESUME
            | ServerUrlProperty::FUMBBL_GAMESTATE_UPDATE
            | ServerUrlProperty::FUMBBL_GAMESTATE_REMOVE
            | ServerUrlProperty::FUMBBL_GAMESTATE_OPTIONS
            | ServerUrlProperty::FUMBBL_RESULT
            | ServerUrlProperty::FUMBBL_TALK
            | ServerUrlProperty::FUMBBL_NAMEGENERATOR_BASE
            | ServerUrlProperty::FUMBBL_PLAYER_MARKINGS => IServerProperty::FUMBBL_BASE,
            _ => IServerProperty::SERVER_BASE,
        }
    }

    fn port_key(&self) -> &'static str {
        match self {
            ServerUrlProperty::FUMBBL_AUTH_CHALLENGE
            | ServerUrlProperty::FUMBBL_AUTH_RESPONSE
            | ServerUrlProperty::FUMBBL_TEAMS
            | ServerUrlProperty::FUMBBL_TEAM
            | ServerUrlProperty::FUMBBL_ROSTER
            | ServerUrlProperty::FUMBBL_ROSTER_TEAM
            | ServerUrlProperty::FUMBBL_GAMESTATE_CHECK
            | ServerUrlProperty::FUMBBL_GAMESTATE_CREATE
            | ServerUrlProperty::FUMBBL_GAMESTATE_RESUME
            | ServerUrlProperty::FUMBBL_GAMESTATE_UPDATE
            | ServerUrlProperty::FUMBBL_GAMESTATE_REMOVE
            | ServerUrlProperty::FUMBBL_GAMESTATE_OPTIONS
            | ServerUrlProperty::FUMBBL_RESULT
            | ServerUrlProperty::FUMBBL_TALK
            | ServerUrlProperty::FUMBBL_NAMEGENERATOR_BASE
            | ServerUrlProperty::FUMBBL_PLAYER_MARKINGS => IServerProperty::FUMBBL_PORT,
            _ => IServerProperty::SERVER_PORT,
        }
    }

    pub fn url(&self, properties: &HashMap<String, String>) -> String {
        Self::url_from_keys(self.base_key(), self.port_key(), self.path_key(), properties)
    }

    /// Java `ServerUrlProperty.url(Properties)` body, parameterized over the
    /// three property keys (Java tests mock getBaseKey/getPortKey/getPathKey).
    fn url_from_keys(
        base_key: &str,
        port_key: &str,
        path_key: &str,
        properties: &HashMap<String, String>,
    ) -> String {
        let get = |key: &str| properties.get(key).map(|s| s.as_str()).unwrap_or("");
        let path = get(path_key);
        // Java: if (path.startsWith("http")) return path;
        if path.starts_with("http") {
            return path.to_string();
        }
        let path = path.trim_start_matches('/');
        let base = get(base_key).trim_end_matches('/');
        let mut url = base.to_string();
        let port = get(port_key);
        // Java: if (base.split(":").length < 3 && StringTool.isProvided(port))
        if base.split(':').count() < 3 && !port.is_empty() {
            url.push(':');
            url.push_str(port);
        }
        if !path.is_empty() {
            url.push('/');
            url.push_str(path);
        }
        url
    }

    pub fn path_key(&self) -> &'static str {
        match self {
            ServerUrlProperty::ADMIN_URL_BACKUP => "admin.url.backup",
            ServerUrlProperty::ADMIN_URL_BLOCK => "admin.url.block",
            ServerUrlProperty::ADMIN_URL_CACHE => "admin.url.cache",
            ServerUrlProperty::ADMIN_URL_CHALLENGE => "admin.url.challenge",
            ServerUrlProperty::ADMIN_URL_CLOSE => "admin.url.close",
            ServerUrlProperty::ADMIN_URL_CONCEDE => "admin.url.concede",
            ServerUrlProperty::ADMIN_URL_DELETE => "admin.url.delete",
            ServerUrlProperty::ADMIN_URL_FORCELOG => "admin.url.forcelog",
            ServerUrlProperty::ADMIN_URL_LIST_ID => "admin.url.list.id",
            ServerUrlProperty::ADMIN_URL_LIST_STATUS => "admin.url.list.status",
            ServerUrlProperty::ADMIN_URL_LOGLEVEL => "admin.url.loglevel",
            ServerUrlProperty::ADMIN_URL_LOGFILE => "admin.url.logfile",
            ServerUrlProperty::ADMIN_URL_MESSAGE => "admin.url.message",
            ServerUrlProperty::ADMIN_URL_REFRESH => "admin.url.refresh",
            ServerUrlProperty::ADMIN_URL_SCHEDULE => "admin.url.schedule",
            ServerUrlProperty::ADMIN_URL_SHUTDOWN => "admin.url.shutdown",
            ServerUrlProperty::ADMIN_URL_STATS => "admin.url.stats",
            ServerUrlProperty::ADMIN_URL_UNBLOCK => "admin.url.unblock",
            ServerUrlProperty::ADMIN_URL_UPLOAD => "admin.url.upload",
            ServerUrlProperty::ADMIN_URL_PORTRAIT => "admin.url.portrait",
            ServerUrlProperty::ADMIN_URL_PURGE_TEST => "admin.url.purgetest",
            ServerUrlProperty::ADMIN_URL_REDEPLOY => "admin.url.redeploy",
            ServerUrlProperty::GAMESTATE_URL_CHALLENGE => "gamestate.url.challenge",
            ServerUrlProperty::GAMESTATE_URL_BEHAVIORS => "gamestate.url.behaviours",
            ServerUrlProperty::GAMESTATE_URL_GET => "gamestate.url.get",
            ServerUrlProperty::GAMESTATE_URL_RESET => "gamestate.url.reset",
            ServerUrlProperty::GAMESTATE_URL_RESULT => "gamestate.url.result",
            ServerUrlProperty::GAMESTATE_URL_SET => "gamestate.url.set",
            ServerUrlProperty::BACKUP_URL_CHALLENGE => "backup.url.challenge",
            ServerUrlProperty::BACKUP_URL_LOAD => "backup.url.load",
            ServerUrlProperty::BACKUP_URL_SAVE => "backup.url.save",
            ServerUrlProperty::FUMBBL_AUTH_CHALLENGE => "fumbbl.auth.challenge",
            ServerUrlProperty::FUMBBL_AUTH_RESPONSE => "fumbbl.auth.response",
            ServerUrlProperty::FUMBBL_TEAMS => "fumbbl.teams",
            ServerUrlProperty::FUMBBL_TEAM => "fumbbl.team",
            ServerUrlProperty::FUMBBL_ROSTER => "fumbbl.roster",
            ServerUrlProperty::FUMBBL_ROSTER_TEAM => "fumbbl.roster.team",
            ServerUrlProperty::FUMBBL_GAMESTATE_CHECK => "fumbbl.gamestate.check",
            ServerUrlProperty::FUMBBL_GAMESTATE_CREATE => "fumbbl.gamestate.create",
            ServerUrlProperty::FUMBBL_GAMESTATE_RESUME => "fumbbl.gamestate.resume",
            ServerUrlProperty::FUMBBL_GAMESTATE_UPDATE => "fumbbl.gamestate.update",
            ServerUrlProperty::FUMBBL_GAMESTATE_REMOVE => "fumbbl.gamestate.remove",
            ServerUrlProperty::FUMBBL_GAMESTATE_OPTIONS => "fumbbl.gamestate.options",
            ServerUrlProperty::FUMBBL_RESULT => "fumbbl.result",
            ServerUrlProperty::FUMBBL_TALK => "fumbbl.talk",
            ServerUrlProperty::FUMBBL_NAMEGENERATOR_BASE => "fumbbl.namegenerator.base",
            ServerUrlProperty::FUMBBL_PLAYER_MARKINGS => "fumbbl.playermarkings",
        }
    }
}

#[cfg(test)]
mod tests {
    // Mirrors ffb-java ffb-server ServerUrlPropertyTest (Java: ServerUrlPropertyTest.java).
    // Java mocks getBaseKey/getPortKey/getPathKey; Rust exercises url_from_keys directly.
    use super::*;

    const BASE: &str = "base";
    const BASE_WITH_PORT: &str = "baseWithPort";
    const BASE_WITH_SLASH: &str = "baseWithSlash";
    const PORT: &str = "port";
    const PATH: &str = "path";
    const PATH_WITH_SLASH: &str = "pathWithSlash";
    const PATH_HAS_URL: &str = "pathHasUrl";
    const SLASHES_ONLY: &str = "slashesOnly";

    fn setup_props() -> HashMap<String, String> {
        let mut props = HashMap::new();
        props.insert(BASE.to_string(), "https://host".to_string());
        props.insert(BASE_WITH_PORT.to_string(), "https://host:8080".to_string());
        props.insert(BASE_WITH_SLASH.to_string(), "https://host/".to_string());
        props.insert(PORT.to_string(), "8000".to_string());
        props.insert(PATH.to_string(), PATH.to_string());
        props.insert(PATH_WITH_SLASH.to_string(), "//path".to_string());
        props.insert(PATH_HAS_URL.to_string(), "https://otherhost/path".to_string());
        props
    }

    fn url(base_key: &str, port_key: &str, path_key: &str) -> String {
        ServerUrlProperty::url_from_keys(base_key, port_key, path_key, &setup_props())
    }

    /// Java: `urlReturnsAssembledValue`.
    #[test]
    fn url_returns_assembled_value() {
        assert_eq!("https://host:8000/path", url(BASE, PORT, PATH));
    }

    /// Java: `urlHandlesUndefinedPort`.
    #[test]
    fn url_handles_undefined_port() {
        assert_eq!("https://host/path", url(BASE, "unknownPort", PATH));
    }

    /// Java: `urlHandlesUndefinedPath`.
    #[test]
    fn url_handles_undefined_path() {
        assert_eq!("https://host:8000", url(BASE, PORT, "unknownPath"));
    }

    /// Java: `urlHandlesSlashAsPathAndPrefix`.
    #[test]
    fn url_handles_slash_as_path_and_prefix() {
        assert_eq!("https://host:8000", url(BASE, PORT, SLASHES_ONLY));
    }

    /// Java: `urlIgnoresDuplicateSlashes`.
    #[test]
    fn url_ignores_duplicate_slashes() {
        assert_eq!("https://host/path", url(BASE_WITH_SLASH, "unknownPort", PATH_WITH_SLASH));
    }

    /// Java: `urlIgnoresPortIfPresentInBase`.
    #[test]
    fn url_ignores_port_if_present_in_base() {
        assert_eq!("https://host:8080/path", url(BASE_WITH_PORT, PORT, PATH));
    }

    /// Java: `urlIgnoresOtherValuesIfPathIsFullUrl`.
    #[test]
    fn url_ignores_other_values_if_path_is_full_url() {
        assert_eq!("https://otherhost/path", url(BASE, PORT, PATH_HAS_URL));
    }

    /// Java: `urlRemovesSlashFromBase`.
    #[test]
    fn url_removes_slash_from_base() {
        assert_eq!("https://host:8000/path", url(BASE_WITH_SLASH, PORT, PATH));
    }

    // ── Rust-only: enum key wiring (no Java counterpart) ────────────────────

    #[test]
    fn test_path_key_admin_backup() {
        assert_eq!(ServerUrlProperty::ADMIN_URL_BACKUP.path_key(), "admin.url.backup");
    }

    #[test]
    fn test_fumbbl_variant_uses_fumbbl_base() {
        assert_eq!(ServerUrlProperty::FUMBBL_TEAMS.base_key(), IServerProperty::FUMBBL_BASE);
    }
}
