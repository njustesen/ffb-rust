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
        let get = |key: &str| properties.get(key).map(|s| s.as_str()).unwrap_or("").to_string();
        let base = get(self.base_key());
        let base = base.trim_end_matches('/').to_string();
        let port = get(self.port_key());
        let path = get(self.path_key());
        let path = path.trim_start_matches('/');
        let mut url = base;
        if !url.contains(':') || url.split(':').count() < 3 {
            if !port.is_empty() {
                url.push(':');
                url.push_str(&port);
            }
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
    use super::*;

    #[test]
    fn test_path_key_admin_backup() {
        assert_eq!(ServerUrlProperty::ADMIN_URL_BACKUP.path_key(), "admin.url.backup");
    }

    #[test]
    fn test_fumbbl_variant_uses_fumbbl_base() {
        assert_eq!(ServerUrlProperty::FUMBBL_TEAMS.base_key(), IServerProperty::FUMBBL_BASE);
    }
}
