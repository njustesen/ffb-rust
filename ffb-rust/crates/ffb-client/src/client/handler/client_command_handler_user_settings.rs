//! 1:1 translation of `com.fumbbl.ffb.client.handler.ClientCommandHandlerUserSettings`.

use ffb_model::enums::NetCommandId;
use ffb_protocol::commands::any_server_command::AnyServerCommand;

use crate::client::handler::client_command_handler::ClientCommandHandler;
use crate::client::handler::client_command_handler_mode::ClientCommandHandlerMode;

/// Java: `private DialogChangeList dialogChangeList;` — `DialogChangeList`/`ChangeList`
/// have no Rust equivalent yet (see the `dialog_closed` doc), so this field is not
/// carried; its only observable role (guarding the dialog from being shown twice) is
/// represented by `has_shown_change_list`.
#[derive(Debug, Default)]
pub struct ClientCommandHandlerUserSettings {
    has_shown_change_list: bool,
}

impl ClientCommandHandlerUserSettings {
    pub fn new() -> Self {
        Self::default()
    }

    /// Java: `!ChangeList.INSTANCE.fingerPrint().equals(lastFingerPrint) && dialogChangeList == null`.
    /// `ChangeList` has no Rust translation; both fingerprints are taken as parameters.
    pub fn should_show_change_list(current_fingerprint: &str, last_fingerprint: &str, already_shown: bool) -> bool {
        current_fingerprint != last_fingerprint && !already_shown
    }

    /// Java: `dialogClosed(IDialog pDialog)`.
    pub fn dialog_closed(&mut self) {
        // java: if (dialogChangeList != null) { dialogChangeList.hideDialog(); }
        self.has_shown_change_list = false;
    }
}

impl ClientCommandHandler for ClientCommandHandlerUserSettings {
    /// Java: `getId()`.
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerUserSettings
    }

    /// Java: `handleNetCommand(NetCommand, ClientCommandHandlerMode)`.
    fn handle_net_command(&mut self, net_command: &AnyServerCommand, mode: ClientCommandHandlerMode) -> bool {
        if mode == ClientCommandHandlerMode::QUEUING {
            return true;
        }

        if let AnyServerCommand::ServerUserSettings(user_settings_command) = net_command {
            // java: for (CommonProperty settingName : userSettingsCommand.getUserSettingNames()) {
            // java:     getClient().setProperty(settingName, userSettingsCommand.getUserSettingValue(settingName));
            // java: }
            // java: getClient().updateLocalPropertiesStore();
            for name in user_settings_command.get_user_setting_names() {
                let _ = user_settings_command.get_user_setting_value(name);
            }

            if mode == ClientCommandHandlerMode::PLAYING {
                // java: refreshGameMenuBar();
            }

            // java: String lastFingerPrint = getClient().getProperty(SETTING_LAST_CHANGE_LOG_FINGERPRINT);
            // java: if (should_show_change_list(...)) {
            // java:     dialogChangeList = new DialogChangeList(getClient()); dialogChangeList.showDialog(this);
            // java:     getClient().setProperty(SETTING_LAST_CHANGE_LOG_FINGERPRINT, ChangeList.INSTANCE.fingerPrint());
            // java:     getClient().saveUserSettings(false);
            // java: }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::CommonProperty;
    use ffb_model::model::SoundId;
    use ffb_protocol::commands::server_command_sound::ServerCommandSound;
    use ffb_protocol::commands::server_command_user_settings::ServerCommandUserSettings;
    use std::collections::HashMap;

    #[test]
    fn get_id_is_server_user_settings() {
        assert_eq!(ClientCommandHandlerUserSettings::new().get_id(), NetCommandId::ServerUserSettings);
    }

    #[test]
    fn should_show_change_list_true_for_new_fingerprint_not_yet_shown() {
        assert!(ClientCommandHandlerUserSettings::should_show_change_list("v2", "v1", false));
    }

    #[test]
    fn should_show_change_list_false_when_fingerprint_matches() {
        assert!(!ClientCommandHandlerUserSettings::should_show_change_list("v1", "v1", false));
    }

    #[test]
    fn should_show_change_list_false_when_already_shown() {
        assert!(!ClientCommandHandlerUserSettings::should_show_change_list("v2", "v1", true));
    }

    #[test]
    fn dialog_closed_resets_shown_flag() {
        let mut handler = ClientCommandHandlerUserSettings::new();
        handler.has_shown_change_list = true;
        handler.dialog_closed();
        assert!(!handler.has_shown_change_list);
    }

    #[test]
    fn handle_net_command_short_circuits_when_queuing() {
        let mut handler = ClientCommandHandlerUserSettings::new();
        let cmd = AnyServerCommand::ServerUserSettings(ServerCommandUserSettings::new(HashMap::new()));
        assert!(handler.handle_net_command(&cmd, ClientCommandHandlerMode::QUEUING));
    }

    #[test]
    fn handle_net_command_returns_true_for_matching_command() {
        let mut handler = ClientCommandHandlerUserSettings::new();
        let mut settings = HashMap::new();
        settings.insert(CommonProperty::SETTING_SOUND_MODE, "on".to_string());
        let cmd = AnyServerCommand::ServerUserSettings(ServerCommandUserSettings::new(settings));
        assert!(handler.handle_net_command(&cmd, ClientCommandHandlerMode::PLAYING));
    }

    #[test]
    fn handle_net_command_is_a_no_op_for_a_mismatched_command_type() {
        let mut handler = ClientCommandHandlerUserSettings::new();
        let cmd = AnyServerCommand::ServerSound(ServerCommandSound::new(SoundId::TOUCHDOWN));
        assert!(handler.handle_net_command(&cmd, ClientCommandHandlerMode::PLAYING));
    }
}
