//! 1:1 translation of `com.fumbbl.ffb.client.handler.ClientCommandHandlerModelSync`.
//!
//! Java implements `IAnimationListener` and holds `FantasyFootballClient`/
//! `UserInterface`/`ClientData` references reachable only through the (still
//! GUI-stub) client. This translation keeps the same per-command scratch fields
//! Java stores (`fSyncCommand`/`fMode`/`fBallCoordinate`/etc.) but exposes the
//! actual game-state computations as free/inherent functions taking explicit
//! `&Game`/`&ModelChangeList`/etc. parameters so they can be unit-tested without
//! a working client. GUI/sound/dialog side effects are left as `// java:` notes.

use ffb_model::enums::{ModelChangeId, NetCommandId};
use ffb_model::model::animation::Animation;
use ffb_model::model::animation_type::AnimationType;
use ffb_model::model::change::ModelChangeList;
use ffb_model::model::game::Game;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_list::ReportList;
use ffb_model::types::FieldCoordinate;
use ffb_protocol::commands::any_server_command::AnyServerCommand;

use crate::client::handler::client_command_handler::ClientCommandHandler;
use crate::client::handler::client_command_handler_mode::ClientCommandHandlerMode;

/// Java: the boolean fields computed by `findUpdates(ModelChangeList)`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ModelSyncUpdates {
    pub update_acting_player: bool,
    pub update_turn_nr: bool,
    pub update_turn_mode: bool,
    pub update_timeout: bool,
    pub clear_selected_player: bool,
    pub reload_pitch: bool,
}

/// Java: `private static final Set<ModelChangeId> IGNORE_PLAYER_MARKER`.
pub fn ignore_player_marker() -> [ModelChangeId; 2] {
    [ModelChangeId::FieldModelAddPlayerMarker, ModelChangeId::FieldModelRemovePlayerMarker]
}

#[derive(Debug, Default)]
pub struct ClientCommandHandlerModelSync {
    mode: Option<ClientCommandHandlerMode>,
    ball_coordinate: Option<FieldCoordinate>,
    bomb_coordinate: Option<FieldCoordinate>,
    thrown_player_coordinate: Option<FieldCoordinate>,
    kicked_player_coordinate: Option<FieldCoordinate>,
}

impl ClientCommandHandlerModelSync {
    pub fn new() -> Self {
        Self::default()
    }

    /// Java:
    /// ```java
    /// if ((fMode == QUEUING) || (fMode == PLAYING)) {
    ///     game.setGameTime(fSyncCommand.getGameTime());
    ///     game.setTurnTime(fSyncCommand.getTurnTime());
    /// }
    /// ```
    /// `Game` has no `game_time`/`turn_time` fields of its own in the Rust model
    /// (see `crates/ffb-model/src/model/game.rs`), so this only reports whether
    /// Java would perform the update rather than inventing fields to write to.
    pub fn should_sync_clock(mode: ClientCommandHandlerMode) -> bool {
        mode == ClientCommandHandlerMode::QUEUING || mode == ClientCommandHandlerMode::PLAYING
    }

    /// Java: `private void findUpdates(ModelChangeList pModelChangeList)`.
    pub fn find_updates(model_change_list: &ModelChangeList) -> ModelSyncUpdates {
        let mut updates = ModelSyncUpdates::default();
        for change in model_change_list.iter() {
            match change.change_id {
                ModelChangeId::ActingPlayerMarkSkillUsed
                | ModelChangeId::ActingPlayerSetCurrentMove
                | ModelChangeId::ActingPlayerSetDodging
                | ModelChangeId::ActingPlayerSetGoingForIt
                | ModelChangeId::ActingPlayerSetHasBlocked
                | ModelChangeId::ActingPlayerSetHasFed
                | ModelChangeId::ActingPlayerSetHasFouled
                | ModelChangeId::ActingPlayerSetHasMoved
                | ModelChangeId::ActingPlayerSetHasTriggeredEffect
                | ModelChangeId::ActingPlayerSetHasPassed
                | ModelChangeId::ActingPlayerSetJumping
                | ModelChangeId::ActingPlayerSetPlayerAction
                | ModelChangeId::ActingPlayerSetPlayerId
                | ModelChangeId::ActingPlayerSetStandingUp
                | ModelChangeId::ActingPlayerSetStrength
                | ModelChangeId::ActingPlayerSetSufferingAnimosity
                | ModelChangeId::ActingPlayerSetSufferingBloodLust => {
                    updates.update_acting_player = true;
                }
                ModelChangeId::TurnDataSetTurnNr => {
                    updates.update_turn_nr = true;
                }
                ModelChangeId::GameSetTurnMode => {
                    updates.update_turn_mode = true;
                }
                ModelChangeId::GameOptionsAddOption => {
                    // java: if ((gameOption != null) && (PITCH_URL == gameOption.getId())) { fReloadPitch = true; }
                    // `IGameOption`/`GameOptionId` payload isn't decoded from the generic
                    // `serde_json::Value` change payload here (out of scope for this
                    // translation); the change itself is still recognized.
                }
                _ => {}
            }
        }
        // Java: `case GAME_SET_TIMEOUT_POSSIBLE: fUpdateTimeout = true;` and
        // `case GAME_SET_DEFENDER_ID: fClearSelectedPlayer = (modelChange.getValue() != null);`
        // have no matching `ModelChangeId` variants in the Rust enum yet (see
        // `crates/ffb-model/src/enums/model_change.rs`) — `update_timeout`/
        // `clear_selected_player` therefore always stay `false` here (documented gap,
        // not invented).
        updates
    }

    /// Java: `handleExtraEffects(ReportList pReportList)`.
    /// `IReport` (`crates/ffb-model/src/report/i_report.rs`) has no downcasting support
    /// (no `std::any::Any` bound), so a `Box<dyn IReport>` can't be cast back to
    /// `ReportBlockChoice` the way Java casts `IReport` to its concrete subclass. This
    /// only reports whether any block-choice report is present; extracting its fields
    /// to update `ClientData.blockRolls` (itself a GUI stub) is left as a `// java:` note.
    pub fn has_block_choice_report(report_list: &ReportList) -> bool {
        report_list.get_reports().iter().any(|r| r.get_id() == ReportId::BLOCK_CHOICE)
    }

    /// Java: `boolean waitForAnimation = (animation != null) && ((fMode == PLAYING) ||
    /// ((fMode == REPLAYING) && getClient().getReplayer().isReplayingSingleSpeedForward()));`
    pub fn should_wait_for_animation(
        animation: Option<&Animation>,
        mode: ClientCommandHandlerMode,
        is_replaying_single_speed_forward: bool,
    ) -> bool {
        animation.is_some()
            && (mode == ClientCommandHandlerMode::PLAYING
                || (mode == ClientCommandHandlerMode::REPLAYING && is_replaying_single_speed_forward))
    }

    /// Java: the `switch (animation.getAnimationType())` block that hides the ball/bomb
    /// before playing an animation. `THROW_TEAM_MATE`/`KICK_TEAM_MATE` need
    /// `animation.getThrownPlayerId()`, which the Rust `Animation` struct
    /// (`crates/ffb-model/src/model/animation.rs`) has no field for — those two arms
    /// are left untranslated (documented gap) rather than invented.
    pub fn prepare_for_animation(&mut self, animation: &Animation, game: &mut Game) {
        match animation.get_animation_type() {
            Some(AnimationType::THROW_BOMB) | Some(AnimationType::HAIL_MARY_BOMB) => {
                game.field_model.range_ruler = None;
                self.bomb_coordinate = game.field_model.bomb_coordinate;
                game.field_model.bomb_coordinate = None;
            }
            Some(AnimationType::PASS) | Some(AnimationType::KICK) | Some(AnimationType::HAIL_MARY_PASS) => {
                game.field_model.range_ruler = None;
                self.ball_coordinate = game.field_model.ball_coordinate;
                game.field_model.ball_coordinate = None;
            }
            // java: case THROW_TEAM_MATE: ... game.getFieldModel().remove(thrownPlayer); break;
            // java: case KICK_TEAM_MATE: ... game.getFieldModel().remove(kickedPlayer); break;
            _ => {}
        }
    }

    /// Java: the `animationFinished()` restore switch (the part not requiring
    /// `animation.getThrownPlayerId()` — see `prepare_for_animation`).
    pub fn restore_after_animation(&self, animation: &Animation, game: &mut Game) {
        match animation.get_animation_type() {
            Some(AnimationType::THROW_BOMB) | Some(AnimationType::HAIL_MARY_BOMB) => {
                game.field_model.bomb_coordinate = self.bomb_coordinate;
            }
            Some(AnimationType::PASS) | Some(AnimationType::KICK) | Some(AnimationType::HAIL_MARY_PASS) => {
                game.field_model.ball_coordinate = self.ball_coordinate;
            }
            // java: case THROW_TEAM_MATE / KICK_TEAM_MATE / TRICKSTER — need
            // `animation.getThrownPlayerId()`/`getOldPlayerState()`, not present on the
            // Rust `Animation` struct; left untranslated (documented gap).
            _ => {}
        }
    }
}

impl ClientCommandHandler for ClientCommandHandlerModelSync {
    /// Java: `getId()`.
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerModelSync
    }

    /// Java: `handleNetCommand(NetCommand, ClientCommandHandlerMode)`.
    fn handle_net_command(&mut self, net_command: &AnyServerCommand, mode: ClientCommandHandlerMode) -> bool {
        self.mode = Some(mode);

        if let AnyServerCommand::ServerModelSync(sync_command) = net_command {
            // java: if ((fMode == QUEUING) || (fMode == PLAYING)) { game.setGameTime(...); game.setTurnTime(...); }
            let _ = Self::should_sync_clock(mode);

            if mode == ClientCommandHandlerMode::QUEUING {
                return true;
            }

            // java: modelChangeList.applyTo(game, ...); userInterface.getStatusReport().report(...);
            let _updates = Self::find_updates(sync_command.get_model_changes());
            let _has_block_choice = Self::has_block_choice_report(sync_command.get_report_list());

            let animation = sync_command.get_animation();
            let wait_for_animation = Self::should_wait_for_animation(Some(animation), mode, false);

            // java: prepare-for-animation switch / updateUserinterface() / startAnimation(animation) or playSound(...)

            return !wait_for_animation;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{ModelChangeDataType, Rules};
    use ffb_model::model::change::model_change::ModelChange;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team {
            id: id.into(),
            name: "Team".into(),
            race: "Human".into(),
            roster_id: "human".into(),
            coach: "Coach".into(),
            rerolls: 0,
            apothecaries: 0,
            bribes: 0,
            master_chefs: 0,
            prayers_to_nuffle: 0,
            bloodweiser_kegs: 0,
            riotous_rookies: 0,
            cheerleaders: 0,
            assistant_coaches: 0,
            fan_factor: 0,
            dedicated_fans: 0,
            team_value: 0,
            treasury: 0,
            special_rules: vec![],
            players: vec![],
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_game() -> Game {
        Game::new(make_team("home"), make_team("away"), Rules::Bb2020)
    }

    #[test]
    fn get_id_is_server_model_sync() {
        assert_eq!(ClientCommandHandlerModelSync::new().get_id(), NetCommandId::ServerModelSync);
    }

    #[test]
    fn should_sync_clock_true_for_queuing_and_playing() {
        assert!(ClientCommandHandlerModelSync::should_sync_clock(ClientCommandHandlerMode::QUEUING));
        assert!(ClientCommandHandlerModelSync::should_sync_clock(ClientCommandHandlerMode::PLAYING));
        assert!(!ClientCommandHandlerModelSync::should_sync_clock(ClientCommandHandlerMode::REPLAYING));
    }

    #[test]
    fn find_updates_detects_acting_player_change() {
        let mut list = ModelChangeList::default();
        list.add(ModelChange::new(ModelChangeId::ActingPlayerSetHasMoved, ModelChangeDataType::Boolean, serde_json::json!(true)));
        let updates = ClientCommandHandlerModelSync::find_updates(&list);
        assert!(updates.update_acting_player);
        assert!(!updates.update_turn_nr);
    }

    #[test]
    fn find_updates_detects_turn_nr_and_turn_mode() {
        let mut list = ModelChangeList::default();
        list.add(ModelChange::new(ModelChangeId::TurnDataSetTurnNr, ModelChangeDataType::Integer, serde_json::json!(3)));
        list.add(ModelChange::new(ModelChangeId::GameSetTurnMode, ModelChangeDataType::String, serde_json::json!("KICKOFF")));
        let updates = ClientCommandHandlerModelSync::find_updates(&list);
        assert!(updates.update_turn_nr);
        assert!(updates.update_turn_mode);
    }

    #[test]
    fn find_updates_ignores_unrelated_changes() {
        let mut list = ModelChangeList::default();
        list.add(ModelChange::new(ModelChangeId::FieldModelSetBallCoordinate, ModelChangeDataType::String, serde_json::Value::Null));
        let updates = ClientCommandHandlerModelSync::find_updates(&list);
        assert_eq!(updates, ModelSyncUpdates::default());
    }

    #[test]
    fn has_block_choice_report_false_for_empty_list() {
        let list = ReportList::new();
        assert!(!ClientCommandHandlerModelSync::has_block_choice_report(&list));
    }

    #[test]
    fn should_wait_for_animation_false_without_animation() {
        assert!(!ClientCommandHandlerModelSync::should_wait_for_animation(None, ClientCommandHandlerMode::PLAYING, false));
    }

    #[test]
    fn should_wait_for_animation_true_when_playing() {
        let animation = Animation::new().with_type(AnimationType::PASS);
        assert!(ClientCommandHandlerModelSync::should_wait_for_animation(Some(&animation), ClientCommandHandlerMode::PLAYING, false));
    }

    #[test]
    fn should_wait_for_animation_respects_replaying_single_speed_flag() {
        let animation = Animation::new().with_type(AnimationType::PASS);
        assert!(!ClientCommandHandlerModelSync::should_wait_for_animation(Some(&animation), ClientCommandHandlerMode::REPLAYING, false));
        assert!(ClientCommandHandlerModelSync::should_wait_for_animation(Some(&animation), ClientCommandHandlerMode::REPLAYING, true));
    }

    #[test]
    fn prepare_and_restore_pass_animation_round_trips_ball_coordinate() {
        let mut handler = ClientCommandHandlerModelSync::new();
        let mut game = make_game();
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(5, 5));
        let animation = Animation::new().with_type(AnimationType::PASS);

        handler.prepare_for_animation(&animation, &mut game);
        assert!(game.field_model.ball_coordinate.is_none());

        handler.restore_after_animation(&animation, &mut game);
        assert_eq!(game.field_model.ball_coordinate, Some(FieldCoordinate::new(5, 5)));
    }

    #[test]
    fn prepare_and_restore_bomb_animation_round_trips_bomb_coordinate() {
        let mut handler = ClientCommandHandlerModelSync::new();
        let mut game = make_game();
        game.field_model.bomb_coordinate = Some(FieldCoordinate::new(1, 1));
        let animation = Animation::new().with_type(AnimationType::THROW_BOMB);

        handler.prepare_for_animation(&animation, &mut game);
        assert!(game.field_model.bomb_coordinate.is_none());

        handler.restore_after_animation(&animation, &mut game);
        assert_eq!(game.field_model.bomb_coordinate, Some(FieldCoordinate::new(1, 1)));
    }

    #[test]
    fn handle_net_command_short_circuits_when_queuing() {
        use ffb_model::model::change::ModelChangeList;
        use ffb_model::model::SoundId;
        use ffb_model::report::report_list::ReportList;
        use ffb_protocol::commands::server_command_model_sync::ServerCommandModelSync;

        let mut handler = ClientCommandHandlerModelSync::new();
        let cmd = AnyServerCommand::ServerModelSync(ServerCommandModelSync::new(
            ModelChangeList::default(),
            ReportList::new(),
            Animation::new(),
            SoundId::TOUCHDOWN,
            0,
            0,
        ));
        assert!(handler.handle_net_command(&cmd, ClientCommandHandlerMode::QUEUING));
    }

    #[test]
    fn handle_net_command_is_a_no_op_for_a_mismatched_command_type() {
        use ffb_model::model::SoundId;
        use ffb_protocol::commands::server_command_sound::ServerCommandSound;
        let mut handler = ClientCommandHandlerModelSync::new();
        let cmd = AnyServerCommand::ServerSound(ServerCommandSound::new(SoundId::TOUCHDOWN));
        assert!(handler.handle_net_command(&cmd, ClientCommandHandlerMode::PLAYING));
    }
}
