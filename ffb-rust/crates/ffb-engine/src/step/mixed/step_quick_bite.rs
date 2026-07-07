/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.StepQuickBite`.
///
/// Handles the Quick Bite skill (canAttackOpponentForBallAfterCatch) after a catch.
///
/// Java logic (executeStep):
///   - If `use_skill` == None: find adjacent opponents with Quick Bite property.
///     - 0 opponents → skip (next step).
///     - 1 opponent → show DialogSkillUseParameter → CONTINUE.
///     - 2+ opponents → show DialogPlayerChoiceParameter(QUICK_BITE) → CONTINUE.
///   - If `use_skill` == Some(true): run injury (InjuryTypeQuickBite), drop player context.
///     - If armor broken: if touchback → publish TOUCHBACK; else publish PLAYER_ID + REVERT_END_TURN.
///   - Next step always.
///
/// Java fields: `catcherId`, `playerId`, `playerIds`, `useSkill`.
///
/// Java: `StepQuickBite extends AbstractStep` (mixed, BB2020 + BB2025).
use ffb_model::model::game::Game;
use ffb_model::model::property::NamedProperties;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::report::report_skill_use::ReportSkillUse;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_cards::UtilCards;
use ffb_model::util::util_player::UtilPlayer;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepQuickBite` (mixed, BB2020 + BB2025).
pub struct StepQuickBite {
    /// Java: `catcherId` (CATCHER_ID parameter — consumed on receipt)
    pub catcher_id: Option<String>,
    /// Java: `playerId` — the Quick Bite attacker chosen from the dialog
    pub player_id: Option<String>,
    /// Java: `playerIds` — candidates with unused Quick Bite skill
    pub player_ids: Vec<String>,
    /// Java: `useSkill` — None until dialog answered
    pub use_skill: Option<bool>,
}

impl StepQuickBite {
    pub fn new() -> Self {
        Self {
            catcher_id: None,
            player_id: None,
            player_ids: Vec::new(),
            use_skill: None,
        }
    }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let catcher_id = match &self.catcher_id {
            Some(id) => id.clone(),
            None => return StepOutcome::next(),
        };

        if self.use_skill.is_none() {
            // Java: Player<?>[] opponents = UtilPlayer.findAdjacentOpposingPlayersWithProperty(
            //   game, catcher, game.getFieldModel().getBallCoordinate(),
            //   NamedProperties.canAttackOpponentForBallAfterCatch, false, true)
            let ball_coord = match game.field_model.ball_coordinate {
                Some(c) => c,
                None => return StepOutcome::next(),
            };
            let opponents: Vec<String> = UtilPlayer::find_adjacent_opposing_players_with_property_ext(
                game,
                &catcher_id,
                ball_coord,
                NamedProperties::CAN_ATTACK_OPPONENT_FOR_BALL_AFTER_CATCH,
                false,
                true, // requireUnusedSkill
            ).into_iter().cloned().collect();

            if opponents.is_empty() {
                return StepOutcome::next();
            }

            self.player_ids.extend(opponents);

            // client-only: DialogSkillUseParameter / DialogPlayerChoiceParameter for QuickBite —
            //   headless falls through without dialog interaction
            return StepOutcome::next();
        } else if self.use_skill == Some(true) {
            // Java: Skill skill = player.getSkillWithProperty(NamedProperties.canAttackOpponentForBallAfterCatch)
            //       getResult().addReport(new ReportSkillUse(playerId, skill, true, SkillUse.QUICK_BITE))
            //       player.markUsed(skill, game)
            if let Some(pid) = &self.player_id {
                let skill = game.player(pid)
                    .and_then(|p| UtilCards::get_unused_skill_with_property(p, NamedProperties::CAN_ATTACK_OPPONENT_FOR_BALL_AFTER_CATCH));
                game.report_list.add(ReportSkillUse::new(Some(pid.clone()), skill.unwrap_or(ffb_model::enums::SkillId::QuickBite), true, SkillUse::QUICK_BITE));
                game.mark_skill_used(pid, ffb_model::enums::SkillId::QuickBite);
            }

            if let Some(ref player_id) = self.player_id.clone() {
                let catcher_coord = game.field_model.player_coordinate(&catcher_id)
                    .unwrap_or(ffb_model::types::FieldCoordinate::new(5, 5));
                let injury_result = crate::step::util_server_injury::handle_injury_by_name(
                    game,
                    rng,
                    "quickBite",
                    Some(player_id),
                    &catcher_id,
                    catcher_coord,
                    None,
                    None,
                    ffb_model::enums::ApothecaryMode::QuickBite,
                );
                let is_armor_broken = injury_result.injury_context.armor_broken;
                let drop_ctx = crate::drop_player_context::DropPlayerContext::with_injury(
                    injury_result,
                    catcher_id.clone(),
                    ffb_model::enums::ApothecaryMode::QuickBite,
                    true,
                );
                let mut outcome = StepOutcome::next()
                    .publish(StepParameter::DropPlayerContext(Box::new(drop_ctx)));

                if is_armor_broken {
                    // Java: setBallCoordinate to player's coordinate + PLAYER_ID + REVERT_END_TURN
                    if let Some(player_coord) = game.field_model.player_coordinate(player_id) {
                        game.field_model.ball_coordinate = Some(player_coord);
                    }
                    let attacker_on_acting_team = game.home_playing == game.team_home.has_player(player_id);
                    outcome = outcome
                        .publish(StepParameter::PlayerId(player_id.clone()))
                        .publish(StepParameter::RevertEndTurn(attacker_on_acting_team));
                }
                return outcome;
            }
        }

        StepOutcome::next()
    }
}

impl Default for StepQuickBite {
    fn default() -> Self { Self::new() }
}

impl Step for StepQuickBite {
    fn id(&self) -> StepId { StepId::QuickBite }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            // Java: CLIENT_PLAYER_CHOICE — array of selected player IDs
            Action::PlayerChoice { player_ids, .. } => {
                if let Some(pid) = player_ids.first() {
                    if self.player_ids.contains(pid) {
                        self.player_id = Some(pid.clone());
                        self.use_skill = Some(true);
                    } else {
                        self.use_skill = Some(false);
                    }
                } else {
                    self.use_skill = Some(false);
                }
            }
            // Java: CLIENT_USE_SKILL(canAttackOpponentForBallAfterCatch)
            Action::UseSkill { use_skill, .. } => {
                self.use_skill = Some(*use_skill);
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            // Java: CATCHER_ID — consumed immediately
            StepParameter::CatcherId(v) => { self.catcher_id = v.clone(); true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{test_team, StepAction};
    use ffb_model::enums::Rules;
    use ffb_model::model::game::Game;
    use ffb_model::util::rng::GameRng;
    use ffb_mechanics::skills::SkillId;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn id_is_quick_bite() {
        assert_eq!(StepQuickBite::new().id(), StepId::QuickBite);
    }

    #[test]
    fn start_returns_next_step() {
        let mut step = StepQuickBite::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_catcher_id() {
        let mut step = StepQuickBite::new();
        let accepted = step.set_parameter(&StepParameter::CatcherId(Some("catcher".into())));
        assert!(accepted);
        assert_eq!(step.catcher_id, Some("catcher".into()));
    }

    #[test]
    fn handle_use_skill_true_sets_use_skill() {
        let mut step = StepQuickBite::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let action = Action::UseSkill { skill_id: SkillId::Block, use_skill: true };
        step.handle_command(&action, &mut game, &mut rng);
        assert_eq!(step.use_skill, Some(true));
    }

    #[test]
    fn handle_use_skill_false_sets_use_skill() {
        let mut step = StepQuickBite::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let action = Action::UseSkill { skill_id: SkillId::Block, use_skill: false };
        step.handle_command(&action, &mut game, &mut rng);
        assert_eq!(step.use_skill, Some(false));
    }

    #[test]
    fn quick_bite_report_added_when_use_skill_true() {
        let mut step = StepQuickBite::new();
        step.catcher_id = Some("catcher".into());
        step.player_id = Some("qb_player".into());
        step.use_skill = Some(true);
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert!(game.report_list.has_report(ffb_model::report::report_id::ReportId::SKILL_USE));
    }

    #[test]
    fn no_quick_bite_report_when_use_skill_none() {
        let mut step = StepQuickBite::new();
        // use_skill stays None → opponent-search path → no report
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert!(!game.report_list.has_report(ffb_model::report::report_id::ReportId::SKILL_USE));
    }
}
