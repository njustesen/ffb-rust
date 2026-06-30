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
/// Injury pipeline / UtilServerInjury not yet fully ported — stubbed.
///
/// Java: `StepQuickBite extends AbstractStep` (mixed, BB2020 + BB2025).
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
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

    fn execute_step(&mut self, _game: &mut Game) -> StepOutcome {
        // Java: full logic requires UtilPlayer.findAdjacentOpposingPlayersWithProperty,
        //   UtilServerInjury.handleInjury, and dialog utilities.
        // TODO(skill/injury port):
        //   1. Find adjacent opponents with canAttackOpponentForBallAfterCatch
        //   2. Show appropriate dialog
        //   3. If use_skill=true: run injury, publish DropPlayerContext
        //   4. If armor broken: publish Touchback or PlayerId+RevertEndTurn
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
