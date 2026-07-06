/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.foul.StepEjectPlayer`.
///
/// Step in foul sequence to handle ejecting a spotted fouler (BB2016).
/// Logic from `SneakyGitBehaviour.handleExecuteStepHook(StepEjectPlayer)` is inlined:
///   - If `argue_the_call_successful` → set state to RESERVE (not banned).
///   - Else if SneakyGit + SNEAKY_GIT_BAN_TO_KO option → set to KNOCKED_OUT.
///   - Else → set to BANNED.
/// Then: puts fouler in the box, updates player-state-dependent properties.
/// Publishes END_TURN.
/// If fouler had the ball: publishes CATCH_SCATTER_THROW_IN_MODE::SCATTER_BALL + NEXT_STEP.
/// Otherwise: goto end label.
///
/// Init parameter: GOTO_LABEL_ON_END (mandatory).
/// Receives: FOULER_HAS_BALL, ARGUE_THE_CALL_SUCCESSFUL.
/// Publishes: END_TURN, CATCH_SCATTER_THROW_IN_MODE.
///
/// Publishes ReportSkillWasted for any unused single-use-reroll skills on the ejected player.
use ffb_model::enums::{PS_RESERVE, PS_BANNED, PS_KNOCKED_OUT, SkillId};
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_box::UtilBox;
use crate::action::Action;
use crate::step::framework::{CatchScatterThrowInMode, Step, StepOutcome, StepId, StepParameter};
use crate::util::util_server_game::UtilServerGame;

/// Java: `StepEjectPlayer` (bb2016/foul).
pub struct StepEjectPlayer {
    /// Java: `state.gotoLabelOnEnd` — mandatory init param.
    goto_label_on_end: String,
    /// Java: `state.foulerHasBall`
    fouler_has_ball: Option<bool>,
    /// Java: `state.argueTheCallSuccessful`
    argue_the_call_successful: Option<bool>,
}

impl StepEjectPlayer {
    pub fn new() -> Self {
        Self {
            goto_label_on_end: String::new(),
            fouler_has_ball: None,
            argue_the_call_successful: None,
        }
    }

    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        if let Some(player_id) = game.acting_player.player_id.clone() {
            // Java: executeStepHooks(this, state) → SneakyGitBehaviour.handleExecuteStepHook
            // Sets player state to RESERVE/KNOCKED_OUT/BANNED before putPlayerIntoBox.
            let has_sneaky_git = game.player(&player_id)
                .map(|p| p.has_skill(SkillId::SneakyGit))
                .unwrap_or(false);

            // Java: state.argueTheCallSuccessful != null && state.argueTheCallSuccessful
            let state_new_base = if self.argue_the_call_successful == Some(true) {
                PS_RESERVE // argue succeeded → not ejected
            } else if has_sneaky_git && false {
                // Java: SNEAKY_GIT_BAN_TO_KO option check (hardcoded false until options are ported)
                PS_KNOCKED_OUT
            } else {
                PS_BANNED
            };

            if let Some(current_state) = game.field_model.player_state(&player_id) {
                game.field_model.set_player_state(&player_id, current_state.change_base(state_new_base));
            }

            UtilBox::put_player_into_box(game, &player_id);
        }
        UtilBox::refresh_boxes(game);
        if let Some(ref pid) = game.acting_player.player_id.clone() {
            UtilServerGame::check_for_wasted_skills(game, pid);
        }
        UtilServerGame::update_player_state_dependent_properties(game);
        let has_ball = self.fouler_has_ball.unwrap_or(false);
        if has_ball {
            StepOutcome::next()
                .publish(StepParameter::EndTurn(true))
                .publish(StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ScatterBall))
        } else {
            StepOutcome::goto(&self.goto_label_on_end)
                .publish(StepParameter::EndTurn(true))
        }
    }
}

impl Default for StepEjectPlayer {
    fn default() -> Self { Self::new() }
}

impl Step for StepEjectPlayer {
    fn id(&self) -> StepId { StepId::EjectPlayer }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(s)         => { self.goto_label_on_end = s.clone(); true }
            StepParameter::FoulerHasBall(v)          => { self.fouler_has_ball = Some(*v); true }
            StepParameter::ArgueTheCallSuccessful(v)  => { self.argue_the_call_successful = Some(*v); true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    #[test]
    fn id_is_eject_player() {
        assert_eq!(StepEjectPlayer::new().id(), StepId::EjectPlayer);
    }

    #[test]
    fn no_ball_goto_label() {
        let mut game = make_game();
        let mut step = StepEjectPlayer::new();
        step.goto_label_on_end = "end".into();
        step.fouler_has_ball = Some(false);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::GotoLabel));
        assert_eq!(out.goto_label.as_deref(), Some("end"));
    }

    #[test]
    fn fouler_had_ball_scatter_ball() {
        let mut game = make_game();
        let mut step = StepEjectPlayer::new();
        step.goto_label_on_end = "end".into();
        step.fouler_has_ball = Some(true);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
        assert!(out.published.iter().any(|p| matches!(p,
            StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ScatterBall))));
    }

    #[test]
    fn publishes_end_turn_always() {
        let mut game = make_game();
        let mut step = StepEjectPlayer::new();
        step.goto_label_on_end = "end".into();
        step.fouler_has_ball = Some(false);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    #[test]
    fn set_parameter_fouler_has_ball() {
        let mut step = StepEjectPlayer::new();
        assert!(step.set_parameter(&StepParameter::FoulerHasBall(true)));
        assert_eq!(step.fouler_has_ball, Some(true));
    }

    #[test]
    fn set_parameter_argue_the_call() {
        let mut step = StepEjectPlayer::new();
        assert!(step.set_parameter(&StepParameter::ArgueTheCallSuccessful(true)));
        assert_eq!(step.argue_the_call_successful, Some(true));
    }

    fn add_player_with_state(game: &mut Game, id: &str, base_state: u32, skills: Vec<SkillId>) {
        use ffb_model::model::player::Player;
        use ffb_model::model::skill_def::SkillWithValue;
        use ffb_model::enums::{PlayerType, PlayerGender, PlayerState as PS};
        use ffb_model::types::FieldCoordinate;
        use std::collections::HashSet;
        let player = Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: skills.into_iter().map(|s| SkillWithValue { skill_id: s, value: None }).collect(),
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        };
        game.team_home.players.push(player);
        game.field_model.set_player_coordinate(id, FieldCoordinate::new(5, 5));
        game.field_model.set_player_state(id, PS::new(base_state));
    }

    #[test]
    fn fouler_set_to_banned_without_sneaky_git() {
        use ffb_model::enums::{PS_STANDING, PlayerState};
        let mut game = make_game();
        add_player_with_state(&mut game, "fouler", PS_STANDING, vec![]);
        game.acting_player.player_id = Some("fouler".into());

        let mut step = StepEjectPlayer::new();
        step.goto_label_on_end = "end".into();
        step.fouler_has_ball = Some(false);
        step.start(&mut game, &mut GameRng::new(0));

        let state = game.field_model.player_state("fouler").expect("state");
        assert_eq!(state.base(), PS_BANNED, "fouler should be BANNED");
    }

    #[test]
    fn argue_the_call_success_sets_player_to_reserve() {
        use ffb_model::enums::{PS_STANDING, PlayerState};
        let mut game = make_game();
        add_player_with_state(&mut game, "fouler", PS_STANDING, vec![]);
        game.acting_player.player_id = Some("fouler".into());

        let mut step = StepEjectPlayer::new();
        step.goto_label_on_end = "end".into();
        step.fouler_has_ball = Some(false);
        step.argue_the_call_successful = Some(true);
        step.start(&mut game, &mut GameRng::new(0));

        let state = game.field_model.player_state("fouler").expect("state");
        assert_eq!(state.base(), PS_RESERVE, "fouler should be RESERVE after argue success");
    }

    #[test]
    fn no_acting_player_does_not_panic() {
        let mut game = make_game();
        // acting_player.player_id is None
        let mut step = StepEjectPlayer::new();
        step.goto_label_on_end = "end".into();
        step.fouler_has_ball = Some(false);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, crate::step::framework::StepAction::GotoLabel);
    }
}
