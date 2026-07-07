/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2025.shared.StepStallingPlayer`.
///
/// Handles a stalling player: checks stalling conditions at the end of a player action
/// and may apply a penalty via `StallingExtension.handle_staller()`.
///
/// Java `start()` logic:
///  1. If acting player is null OR `enableStallingCheck` option is off →
///     clear the stalling flag and return (also covers "ending your turn" path).
///  2. `gotRid`  = stalling flag was set BUT player no longer has the ball.
///  3. `scored`  = `UtilServerSteps.checkTouchdown(gameState)`.
///  4. `noStalling` = !stalling || gotRid || scored.
///  5. Clear stalling flag (`resetStalling()`).
///  6. If `noStalling` OR player is prone/stunned → optionally log "did not stall" + return.
///  7. Otherwise → `StallingExtension.handleStaller(this, player)`.
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_player_event::ReportPlayerEvent;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_player::UtilPlayer;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_steps::check_touchdown;
use super::stalling_extension::StallingExtension;

pub struct StepStallingPlayer {
    stalling_extension: StallingExtension,
}

impl StepStallingPlayer {
    pub fn new() -> Self {
        Self { stalling_extension: StallingExtension::new() }
    }
}

impl Default for StepStallingPlayer {
    fn default() -> Self { Self::new() }
}

impl Step for StepStallingPlayer {
    fn id(&self) -> StepId { StepId::StallingPlayer }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

impl StepStallingPlayer {
    fn execute_step(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: player == null || !UtilGameOption.isOptionEnabled(game, ENABLE_STALLING_CHECK)
        let player_id = game.acting_player.player_id.clone();
        let option_enabled = game.options.is_enabled("enableStallingCheck");

        if player_id.is_none() || !option_enabled {
            // Java: getGameState().resetStalling();
            game.stalling = false;
            return StepOutcome::next();
        }

        let player_id = player_id.unwrap();

        // Java: boolean gotRid = getGameState().isStalling() && !UtilPlayer.hasBall(game, player);
        let got_rid = game.stalling && !UtilPlayer::has_ball(game, &player_id);

        // Java: boolean scored = UtilServerSteps.checkTouchdown(getGameState());
        let scored = check_touchdown(game);

        // Java: boolean noStalling = !getGameState().isStalling() || gotRid || scored;
        let no_stalling = !game.stalling || got_rid || scored;

        // Java: getGameState().resetStalling();
        game.stalling = false;

        // Java: if (noStalling || game.getFieldModel().getPlayerState(player).isProneOrStunned())
        let is_prone_or_stunned = game.field_model
            .player_state(&player_id)
            .map(|s| s.is_prone_or_stunned())
            .unwrap_or(false);

        if no_stalling || is_prone_or_stunned {
            // Java: if (gotRid || scored) { addReport(new ReportPlayerEvent(id, "did not stall after all")) }
            if got_rid || scored {
                game.report_list.add(ReportPlayerEvent::new(
                    Some(player_id.clone()),
                    Some("did not stall after all".into()),
                ));
                return StepOutcome::next()
                    .with_event(GameEvent::PlayerNote { player_id: player_id.clone(), note: "did not stall after all".into() });
            }
            return StepOutcome::next();
        }

        // Java: stallingExtension.handleStaller(this, player);
        let turn_nr = game.turn_data().turn_nr;
        let stalling_ev = self.stalling_extension.handle_staller(game, &player_id, turn_nr, rng);

        StepOutcome::next().with_event(stalling_ev)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::{Rules, PlayerAction, PS_STANDING, PS_PRONE};
    use ffb_model::types::FieldCoordinate;
    use ffb_model::enums::PlayerState;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_player_to_home(game: &mut Game, id: &str, pos: FieldCoordinate, state: u32) {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender};
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
            passing: 4, armour: 9, starting_skills: vec![], extra_skills: vec![],
            temporary_skills: vec![], used_skills: std::collections::HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.field_model.set_player_coordinate(id, pos);
        game.field_model.set_player_state(id, PlayerState::new(state));
    }

    #[test]
    fn start_returns_next() {
        let mut game = make_game();
        let mut step = StepStallingPlayer::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_returns_false() {
        let mut step = StepStallingPlayer::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(false)));
    }

    #[test]
    fn null_player_resets_stalling_and_returns_next() {
        let mut game = make_game();
        game.stalling = true;
        game.options.set("enableStallingCheck", "true");
        // acting player is None (null in Java)
        let mut step = StepStallingPlayer::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!game.stalling, "stalling flag must be cleared");
    }

    #[test]
    fn option_disabled_resets_stalling_and_returns_next() {
        let mut game = make_game();
        game.stalling = true;
        let pos = FieldCoordinate::new(5, 5);
        add_player_to_home(&mut game, "h1", pos, PS_STANDING);
        game.acting_player.set_player("h1".into(), PlayerAction::Move);
        // enableStallingCheck not set → disabled
        let mut step = StepStallingPlayer::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!game.stalling, "stalling flag must be cleared");
    }

    #[test]
    fn no_stalling_flag_returns_next_without_penalty() {
        let mut game = make_game();
        game.options.set("enableStallingCheck", "true");
        let pos = FieldCoordinate::new(5, 5);
        add_player_to_home(&mut game, "h1", pos, PS_STANDING);
        game.acting_player.set_player("h1".into(), PlayerAction::Move);
        // stalling = false → noStalling = true
        let mut step = StepStallingPlayer::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!game.game_result.home.stalled);
    }

    #[test]
    fn got_rid_of_ball_resets_without_penalty() {
        let mut game = make_game();
        game.options.set("enableStallingCheck", "true");
        let pos = FieldCoordinate::new(5, 5);
        add_player_to_home(&mut game, "h1", pos, PS_STANDING);
        game.acting_player.set_player("h1".into(), PlayerAction::Move);
        game.stalling = true;
        // ball not at player → gotRid = true → noStalling = true
        game.field_model.ball_in_play = true;
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(10, 10));
        let mut step = StepStallingPlayer::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!game.stalling);
        // got_rid means no penalty
        assert!(!game.game_result.home.stalled);
    }

    #[test]
    fn stalling_detected_with_ball_applies_penalty() {
        let mut game = make_game();
        game.options.set("enableStallingCheck", "true");
        let pos = FieldCoordinate::new(5, 5);
        add_player_to_home(&mut game, "h1", pos, PS_STANDING);
        game.acting_player.set_player("h1".into(), PlayerAction::Move);
        game.stalling = true;
        // ball at player → not got_rid; not in endzone → not scored
        game.field_model.ball_in_play = true;
        game.field_model.ball_coordinate = Some(pos);
        let mut step = StepStallingPlayer::new();
        let out = step.start(&mut game, &mut GameRng::new(42));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!game.stalling, "stalling must be reset");
        // handle_staller was called → team marked stalled
        assert!(game.game_result.home.stalled);
    }

    #[test]
    fn prone_player_skips_penalty_even_when_stalling() {
        let mut game = make_game();
        game.options.set("enableStallingCheck", "true");
        let pos = FieldCoordinate::new(5, 5);
        add_player_to_home(&mut game, "h1", pos, PS_PRONE);
        game.acting_player.set_player("h1".into(), PlayerAction::Move);
        game.stalling = true;
        game.field_model.ball_in_play = true;
        game.field_model.ball_coordinate = Some(pos);
        let mut step = StepStallingPlayer::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        // prone → no penalty
        assert!(!game.game_result.home.stalled);
    }

    #[test]
    fn got_rid_adds_player_event_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        game.options.set("enableStallingCheck", "true");
        let pos = FieldCoordinate::new(5, 5);
        add_player_to_home(&mut game, "h1", pos, PS_STANDING);
        game.acting_player.set_player("h1".into(), PlayerAction::Move);
        game.stalling = true;
        // ball not at player → gotRid = true
        game.field_model.ball_in_play = true;
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(10, 10));
        let mut step = StepStallingPlayer::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::PLAYER_EVENT),
            "PLAYER_EVENT report expected when got_rid=true");
    }

    #[test]
    fn no_stalling_no_got_rid_does_not_add_player_event_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        game.options.set("enableStallingCheck", "true");
        let pos = FieldCoordinate::new(5, 5);
        add_player_to_home(&mut game, "h1", pos, PS_STANDING);
        game.acting_player.set_player("h1".into(), PlayerAction::Move);
        // stalling = false, ball elsewhere → noStalling=true but gotRid=false
        game.stalling = false;
        let mut step = StepStallingPlayer::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.report_list.has_report(ReportId::PLAYER_EVENT),
            "PLAYER_EVENT must NOT be added when no stalling and no got_rid");
    }
}
