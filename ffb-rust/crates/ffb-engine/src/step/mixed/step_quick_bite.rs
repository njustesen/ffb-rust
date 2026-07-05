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
use ffb_model::util::rng::GameRng;
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

    fn execute_step(&mut self, game: &mut Game) -> StepOutcome {
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

            // DEFERRED(dialog port): UtilServerDialog.showDialog
            //   - 1 opponent: DialogSkillUseParameter(firstOpponent.getId(), skill, 0) → CONTINUE
            //   - 2+ opponents: DialogPlayerChoiceParameter(team_id, QUICK_BITE, opponents, null, 1) → CONTINUE
            // Until dialog is ported, fall through to NEXT_STEP so we don't block the game.
            return StepOutcome::next();
        } else if self.use_skill == Some(true) {
            // Java: player.markUsed(skill, game)
            if let Some(pid) = &self.player_id {
                game.mark_skill_used(pid, ffb_model::enums::SkillId::QuickBite);
            }

            // DEFERRED(UtilServerInjury port): UtilServerInjury.handleInjury(this, InjuryTypeQuickBite,
            //   player, catcher, fieldModel.getPlayerCoordinate(catcher), null, null, ApothecaryMode.QUICK_BITE)
            // publishParameter(DROP_PLAYER_CONTEXT, new DropPlayerContext(injuryResult, false, false, null,
            //   catcherId, ApothecaryMode.QUICK_BITE, true))
            // if (injuryResult.injuryContext().isArmorBroken()) {
            //   game.fieldModel.setBallCoordinate(game.fieldModel.getPlayerCoordinate(player))
            //   if (touchback) → publish TOUCHBACK
            //   else → publish PLAYER_ID(playerId) + REVERT_END_TURN(true if player.team == actingTeam)
            // }
        }

        StepOutcome::next()
    }
}

impl Default for StepQuickBite {
    fn default() -> Self { Self::new() }
}

impl Step for StepQuickBite {
    fn id(&self) -> StepId { StepId::QuickBite }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
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
        self.execute_step(game)
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
}
