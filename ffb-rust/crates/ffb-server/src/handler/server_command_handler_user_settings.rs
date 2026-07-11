/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerUserSettings.
use std::sync::{Arc, Mutex};
use ffb_model::enums::NetCommandId;
use ffb_protocol::commands::client_command_user_settings::ClientCommandUserSettings;
use crate::db::db_connection_manager::DbConnectionManager;
use crate::db::delete::db_user_settings_delete::DbUserSettingsDelete;
use crate::db::insert::db_user_settings_insert_parameter::DbUserSettingsInsertParameter;
use crate::model::received_command::SessionId;
use crate::net::session_manager::SessionManager;

/// Java: `ServerCommandHandlerUserSettings extends ServerCommandHandler`.
pub struct ServerCommandHandlerUserSettings {
    session_manager: Arc<Mutex<SessionManager>>,
    db_connection_manager: Arc<Mutex<DbConnectionManager>>,
}

impl ServerCommandHandlerUserSettings {
    /// Java: `protected ServerCommandHandlerUserSettings(FantasyFootballServer pServer)`
    pub fn new(
        session_manager: Arc<Mutex<SessionManager>>,
        db_connection_manager: Arc<Mutex<DbConnectionManager>>,
    ) -> Self {
        Self { session_manager, db_connection_manager }
    }

    /// Java: `getId()` — returns `NetCommandId.CLIENT_USER_SETTINGS`.
    pub fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientUserSettings
    }

    /// Java: `handleCommand(ReceivedCommand)`.
    ///
    /// ```java
    /// String coach = getServer().getSessionManager().getCoachForSession(pReceivedCommand.getSession());
    /// if (StringTool.isProvided(coach)) {
    ///     DbTransaction transaction = new DbTransaction();
    ///     transaction.add(new DbUserSettingsDeleteParameter(coach));
    ///     DbUserSettingsInsertParameterList insertParameterList = new DbUserSettingsInsertParameterList();
    ///     for (Map.Entry<String, String> entry : userSettingsCommand.getSettings().entrySet()) {
    ///         insertParameterList.add(new DbUserSettingsInsertParameter(coach, entry.getKey(), entry.getValue()));
    ///     }
    ///     transaction.add(insertParameterList);
    ///     getServer().getDbUpdater().add(transaction);
    /// }
    /// return true;
    /// ```
    ///
    /// Java enqueues the delete+insert pair as one `DbTransaction` on an async `DbUpdater`
    /// worker thread; here the same delete-then-inserts pair is run inline against a
    /// connection checked out from the shared `DbConnectionManager` pool (commit happens
    /// via `close_db_connection`, matching `DbTransaction`'s single-commit semantics).
    pub async fn handle_command(
        &self,
        session_id: SessionId,
        user_settings_command: &ClientCommandUserSettings,
    ) -> bool {
        let coach = {
            let sm = self.session_manager.lock().unwrap();
            sm.get_coach_for_session(session_id).map(|c| c.to_string())
        };

        if let Some(coach) = coach {
            if !coach.is_empty() {
                let mut parameters: Vec<DbUserSettingsInsertParameter> = user_settings_command
                    .settings
                    .iter()
                    .map(|(name, value)| {
                        DbUserSettingsInsertParameter::new(coach.clone(), name.clone(), value.clone())
                    })
                    .collect();

                // `DbConnectionManager` is cloned out from behind the `std::sync::Mutex`
                // before any `.await` (see that struct's own `Clone` doc comment, and
                // `ServerCommandHandlerDeleteGame::handle_command` for the same pattern) so
                // this handler's future stays `Send` — required now that it's reachable from
                // `ServerCommandHandlerFactory::handle_command`, which runs inside
                // `tokio::spawn(dispatch_loop(...))`.
                let manager = self.db_connection_manager.lock().unwrap().clone();
                if manager.pool_ready() {
                    if let Ok(mut conn) = manager.open_db_connection().await {
                        let _ = DbUserSettingsDelete::new().execute(&mut conn, &coach).await;
                        for parameter in &mut parameters {
                            let _ = parameter.execute(&mut conn).await;
                        }
                        let _ = manager.close_db_connection(conn).await;
                    }
                }
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::ClientMode;
    use tokio::sync::mpsc;

    fn setup() -> (Arc<Mutex<SessionManager>>, Arc<Mutex<DbConnectionManager>>) {
        (
            Arc::new(Mutex::new(SessionManager::new())),
            Arc::new(Mutex::new(DbConnectionManager::new())),
        )
    }

    #[test]
    fn construct() {
        let (sm, dcm) = setup();
        let _ = ServerCommandHandlerUserSettings::new(sm, dcm);
    }

    #[test]
    fn get_id_is_client_user_settings() {
        let (sm, dcm) = setup();
        let handler = ServerCommandHandlerUserSettings::new(sm, dcm);
        assert_eq!(handler.get_id(), NetCommandId::ClientUserSettings);
    }

    #[tokio::test]
    async fn unknown_session_is_a_noop() {
        let (sm, dcm) = setup();
        let handler = ServerCommandHandlerUserSettings::new(sm, dcm);
        let result = handler.handle_command(99, &ClientCommandUserSettings::new()).await;
        assert!(result);
    }

    /// Without a live DB pool configured (`DbConnectionManager::new()` has none), the DB
    /// persistence step is skipped entirely (`pool_ready()` is false) — the handler still
    /// extracts the coach/settings and returns `true`, matching the "no DB in this run"
    /// degradation used throughout this crate's DB-backed handlers.
    #[tokio::test]
    async fn known_coach_without_db_pool_is_still_a_noop_returning_true() {
        let (sm, dcm) = setup();
        let (tx, _rx) = mpsc::unbounded_channel();
        sm.lock().unwrap().add_session(1, 100, "Coach1".into(), ClientMode::PLAYER, true, vec![], tx);
        let handler = ServerCommandHandlerUserSettings::new(sm, dcm);
        let mut cmd = ClientCommandUserSettings::new();
        cmd.settings.insert("soundVolume".into(), "80".into());
        let result = handler.handle_command(1, &cmd).await;
        assert!(result);
    }

    #[tokio::test]
    async fn empty_coach_skips_db_step() {
        let (sm, dcm) = setup();
        let (tx, _rx) = mpsc::unbounded_channel();
        sm.lock().unwrap().add_session(1, 100, "".into(), ClientMode::PLAYER, true, vec![], tx);
        let handler = ServerCommandHandlerUserSettings::new(sm, dcm);
        let result = handler.handle_command(1, &ClientCommandUserSettings::new()).await;
        assert!(result);
    }
}
