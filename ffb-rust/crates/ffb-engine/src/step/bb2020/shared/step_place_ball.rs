use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::report::report_skill_use::ReportSkillUse;
use ffb_model::types::FieldCoordinate;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{CatchScatterThrowInMode, StepId, StepParameter};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2020.shared.StepPlaceBall.
/// Places the ball at a player/coordinate (BB2020); handles the Safe Pair of Hands skill dialog.
/// The dialog (Phase::Select/Place) is not yet fully ported — skill use auto-declines.
pub struct StepPlaceBall {
    /// Java: playerId
    pub player_id: Option<String>,
    /// Java: catchScatterThrowInMode
    pub catch_scatter_throw_in_mode: Option<CatchScatterThrowInMode>,
    /// Java: phase (Phase enum) — stored as name until Phase enum is ported
    pub phase_name: String,
    /// Java: ballCarrierTeamTurn
    pub ball_carrier_team_turn: bool,
    /// Java: revertEndTurn
    pub revert_end_turn: bool,
    /// Java: selectedCoordinate
    pub selected_coordinate: Option<FieldCoordinate>,
}

impl StepPlaceBall {
    pub fn new() -> Self {
        Self {
            player_id: None,
            catch_scatter_throw_in_mode: None,
            phase_name: "ASK".to_string(),
            ball_carrier_team_turn: false,
            revert_end_turn: false,
            selected_coordinate: None,
        }
    }
}

impl Default for StepPlaceBall {
    fn default() -> Self { Self::new() }
}

impl Step for StepPlaceBall {
    fn id(&self) -> StepId { StepId::PlaceBall }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::UseSkill { skill_id, use_skill } => {
                let has_prop = skill_id.properties().contains(&NamedProperties::CAN_PLACE_BALL_WHEN_KNOCKED_DOWN_OR_PLACED_PRONE);
                if has_prop {
                    if let Some(pid) = self.player_id.clone() {
                        game.report_list.add(ReportSkillUse::new(
                            Some(pid),
                            *skill_id,
                            *use_skill,
                            SkillUse::PLACE_BALL,
                        ));
                    }
                }
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::PlayerId(v) => { self.player_id = Some(v.clone()); true }
            StepParameter::CatchScatterThrowInMode(v) => { self.catch_scatter_throw_in_mode = Some(*v); true }
            StepParameter::DroppedBallCarrier(_) => true, // consumed
            _ => false,
        }
    }
}

impl StepPlaceBall {
    /// Java: executeStep() with Phase::ASK fast-path.
    ///
    /// Java only publishes DROPPED_BALL_CARRIER (always `null`) from `leave()`, which is
    /// reached at the end of the SELECT/PLACE/DONE phases — NOT from the two early-return
    /// guards (`playerId == null || mode != SCATTER_BALL` in executeStep(), or
    /// `skill == null || cannotUseSkill` in setup()), which just set NEXT_STEP and return
    /// without touching the parameter at all.
    fn execute_step(&self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: if (playerId == null || catchScatterThrowInMode != SCATTER_BALL) → NEXT_STEP
        // (no DROPPED_BALL_CARRIER publish)
        let player_id = match self.player_id.as_deref() {
            Some(id) if self.catch_scatter_throw_in_mode == Some(CatchScatterThrowInMode::ScatterBall) => id,
            _ => return StepOutcome::next(),
        };

        // Java Phase::ASK → setup(): check canPlaceBallWhenKnockedDownOrPlacedProne skill.
        let has_skill = game.player(player_id)
            .map(|p| p.has_skill_property(NamedProperties::CAN_PLACE_BALL_WHEN_KNOCKED_DOWN_OR_PLACED_PRONE))
            .unwrap_or(false);
        let can_use = if has_skill {
            game.field_model.player_state(player_id)
                .map(|s| !s.is_hypnotized() && !s.is_confused())
                .unwrap_or(false)
        } else {
            false
        };

        if !can_use {
            // Java: setup() returns early (skill == null || cannotUseSkill) — no publish.
            return StepOutcome::next();
        }

        // Skill available but dialog infra not yet ported: auto-decline (conservative).
        // Java would show DialogSkillUseParameter and wait for CLIENT_USE_SKILL response;
        // a decline reaches Phase::DONE → leave(), which DOES publish DROPPED_BALL_CARRIER=null.
        StepOutcome::next().publish(StepParameter::DroppedBallCarrier(None))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, CatchScatterThrowInMode};
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2020)
    }

    #[test]
    fn start_returns_next() {
        let mut game = make_game();
        let mut step = StepPlaceBall::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn no_player_id_does_not_publish_dropped_ball_carrier() {
        // Regression: Java's executeStep() early-return guard (playerId == null ||
        // mode != SCATTER_BALL) just sets NEXT_STEP and returns — it never touches
        // DROPPED_BALL_CARRIER. The old Rust code incorrectly published
        // DroppedBallCarrier(None) unconditionally, even when playerId was never set.
        let mut game = make_game();
        let mut step = StepPlaceBall::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(
            !out.published.iter().any(|p| matches!(p, StepParameter::DroppedBallCarrier(_))),
            "must not publish DroppedBallCarrier when playerId was never set"
        );
    }

    #[test]
    fn no_skill_available_does_not_publish_dropped_ball_carrier() {
        // Regression: Java's setup() early-return guard (skill == null || cannotUseSkill)
        // also just sets NEXT_STEP without touching DROPPED_BALL_CARRIER.
        let mut game = make_game();
        let mut step = StepPlaceBall::new();
        step.player_id = Some("p1".into());
        step.catch_scatter_throw_in_mode = Some(CatchScatterThrowInMode::ScatterBall);
        // "p1" has no skills at all, so canPlaceBallWhenKnockedDownOrPlacedProne is absent.
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(
            !out.published.iter().any(|p| matches!(p, StepParameter::DroppedBallCarrier(_))),
            "must not publish DroppedBallCarrier when the placement skill is unavailable"
        );
    }

    #[test]
    fn skill_available_auto_decline_publishes_dropped_ball_carrier_none() {
        // When the skill IS available (and the dialog auto-declines, since Select/Place
        // phases aren't ported), Java's decline path reaches Phase::DONE -> leave(), which
        // DOES publish DROPPED_BALL_CARRIER = null.
        use ffb_model::enums::SkillId;
        use ffb_model::model::skill_def::SkillWithValue;
        let mut game = make_game();
        let mut player = ffb_model::model::player::Player::default();
        player.id = "p1".into();
        player.starting_skills.push(SkillWithValue::new(SkillId::SafePairOfHands));
        game.team_home.players.push(player);
        game.field_model.set_player_state("p1", ffb_model::enums::PlayerState::new(ffb_model::enums::PS_STANDING));
        let mut step = StepPlaceBall::new();
        step.player_id = Some("p1".into());
        step.catch_scatter_throw_in_mode = Some(CatchScatterThrowInMode::ScatterBall);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::DroppedBallCarrier(None))));
    }

    #[test]
    fn set_parameter_player_id_accepted() {
        let mut step = StepPlaceBall::new();
        assert!(step.set_parameter(&StepParameter::PlayerId("p1".into())));
        assert_eq!(step.player_id.as_deref(), Some("p1"));
    }

    #[test]
    fn set_parameter_catch_scatter_mode_accepted() {
        let mut step = StepPlaceBall::new();
        assert!(step.set_parameter(&StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ScatterBall)));
        assert_eq!(step.catch_scatter_throw_in_mode, Some(CatchScatterThrowInMode::ScatterBall));
    }

    #[test]
    fn handle_command_use_skill_adds_skill_use_report() {
        use ffb_model::enums::SkillId;
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        let mut step = StepPlaceBall::new();
        step.player_id = Some("p1".into());
        // SafePairOfHands has CAN_PLACE_BALL_WHEN_KNOCKED_DOWN_OR_PLACED_PRONE
        let action = crate::action::Action::UseSkill { skill_id: SkillId::SafePairOfHands, use_skill: true };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::SKILL_USE), "expected SKILL_USE report on use-skill command");
    }

    #[test]
    fn handle_command_no_skill_no_report() {
        use ffb_model::enums::SkillId;
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        let mut step = StepPlaceBall::new();
        step.player_id = Some("p1".into());
        let action = crate::action::Action::UseSkill { skill_id: SkillId::Block, use_skill: true };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert!(!game.report_list.has_report(ReportId::SKILL_USE), "no SKILL_USE report for irrelevant skill");
    }
}
