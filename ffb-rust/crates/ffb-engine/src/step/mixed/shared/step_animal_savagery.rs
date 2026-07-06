/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.shared.StepAnimalSavagery`.
///
/// Handles the Animal Savagery skill (BB2020+/Mixed) — a player with this skill may
/// lash out at opponents instead of being controlled.  The step drives:
///   1. Init: records `goto_label_on_failure`, `catcher_id` (from TARGET_COORDINATE),
///      and `block_defender_id`.
///   2. On `CLIENT_PLAYER_CHOICE(ANIMAL_SAVAGERY)` → set `player_id` and execute.
///   3. On `CLIENT_USE_SKILL(canLashOutAgainstOpponents)` → set `attack_opponent` flag.
///   4. Entire execution delegated to `executeStepHooks` (not yet ported).
///
/// Java: `com.fumbbl.ffb.server.step.mixed.shared.StepAnimalSavagery`
///        extends `AbstractStepWithReRoll`.
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::enums::SkillId;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Internal state — mirrors Java inner class `StepAnimalSavagery.StepState`.
#[derive(Debug, Default)]
pub struct AnimalSavageryState {
    /// Java: `goToLabelOnFailure` (mandatory init param)
    pub goto_label_on_failure: String,
    /// Java: `playerId` — player selected via ANIMAL_SAVAGERY dialog
    pub player_id: Option<String>,
    /// Java: `thrownPlayerId`
    pub thrown_player_id: Option<String>,
    /// Java: `playerIds` — set of active players (populated by hooks)
    pub player_ids: std::collections::HashSet<String>,
    /// Java: `endTurn`
    pub end_turn: bool,
    /// Java: `catcherId` — resolved from TARGET_COORDINATE init param
    pub catcher_id: Option<String>,
    /// Java: `attackOpponent` — true when the skill use dialog approved lashing out
    pub attack_opponent: Option<bool>,
    /// Java: `blockDefenderId`
    pub block_defender_id: Option<String>,
}

/// Java: `StepAnimalSavagery` (mixed/shared, BB2020 + BB2025).
pub struct StepAnimalSavagery {
    pub state: AnimalSavageryState,
    // AbstractStepWithReRoll stubs
    pub re_rolled_action: Option<String>,
    pub re_roll_source: Option<String>,
    /// Pending coordinate from TARGET_COORDINATE init param; resolved to catcher_id in start().
    pending_target_coordinate: Option<FieldCoordinate>,
}

impl StepAnimalSavagery {
    pub fn new(goto_label_on_failure: impl Into<String>) -> Self {
        Self {
            state: AnimalSavageryState {
                goto_label_on_failure: goto_label_on_failure.into(),
                ..Default::default()
            },
            re_rolled_action: None,
            re_roll_source: None,
            pending_target_coordinate: None,
        }
    }

    fn execute_step(&mut self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: getGameState().executeStepHooks(this, state) — entire logic is in step hooks
        // no-op: Animal Savagery step hooks (SkillBehaviour registry not ported) — auto-proceeds
        StepOutcome::next()
    }
}

impl Default for StepAnimalSavagery {
    fn default() -> Self { Self::new("") }
}

impl Step for StepAnimalSavagery {
    fn id(&self) -> StepId { StepId::AnimalSavagery }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: init() resolves TARGET_COORDINATE → catcherId via FieldModel.getPlayer(coord)
        if let Some(coord) = self.pending_target_coordinate.take() {
            self.state.catcher_id = game.field_model.player_at(coord).cloned();
        }
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java CLIENT_PLAYER_CHOICE (ANIMAL_SAVAGERY) → state.playerId = chosen; EXECUTE_STEP
        // Java CLIENT_USE_SKILL (canLashOutAgainstOpponents) → state.attackOpponent = used; mark skill; EXECUTE_STEP
        match action {
            Action::PlayerChoice { player_id, mode, .. } if mode == "ANIMAL_SAVAGERY" => {
                self.state.player_id = player_id.clone();
                self.execute_step(game, rng)
            }
            Action::UseSkill { skill_id, use_skill } if *skill_id == SkillId::PrimalSavagery => {
                self.state.attack_opponent = Some(*use_skill);
                if *use_skill {
                    // Java: game.getPlayerById(commandUseSkill.getPlayerId()).markUsed(skill, game)
                    if let Some(ref pid) = self.state.player_id {
                        let is_home = game.team_home.player(pid).is_some();
                        if is_home { game.team_home.player_mut(pid).map(|p| p.used_skills.insert(SkillId::PrimalSavagery)); }
                        else { game.team_away.player_mut(pid).map(|p| p.used_skills.insert(SkillId::PrimalSavagery)); }
                    }
                    // Java: addReport(new ReportSkillUse(ANIMAL_SAVAGERY/PrimalSavagery, true))
                    let skill_event = self.state.player_id.as_ref().map(|pid| {
                        GameEvent::SkillUse {
                            player_id: pid.clone(),
                            skill_id: SkillId::PrimalSavagery as u16,
                            used: true,
                        }
                    });
                    let out = self.execute_step(game, rng);
                    if let Some(ev) = skill_event { return out.with_event(ev); }
                    return out;
                }
                self.execute_step(game, rng)
            }
            _ => StepOutcome::next(),
        }
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(v) => {
                self.state.goto_label_on_failure = v.clone();
                true
            }
            StepParameter::TargetCoordinate(coord) => {
                // Java: Player catcher = game.getFieldModel().getPlayer(coord); state.catcherId = catcher.getId()
                // set_parameter has no game reference; store and resolve in start().
                self.pending_target_coordinate = Some(*coord);
                true
            }
            StepParameter::BlockDefenderId(v) => {
                self.state.block_defender_id = Some(v.clone());
                false // Java: super.setParameter() → not consumed
            }
            StepParameter::ThrownPlayerId(v) => {
                self.state.thrown_player_id = v.clone();
                false
            }
            StepParameter::EndTurn(v) => {
                self.state.end_turn = *v;
                false
            }
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
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn id_is_animal_savagery() {
        assert_eq!(StepAnimalSavagery::new("fail").id(), StepId::AnimalSavagery);
    }

    #[test]
    fn start_returns_next_step() {
        let mut step = StepAnimalSavagery::new("fail");
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn goto_label_set_from_parameter() {
        let mut step = StepAnimalSavagery::default();
        step.set_parameter(&StepParameter::GotoLabelOnFailure("skip".into()));
        assert_eq!(step.state.goto_label_on_failure, "skip");
    }

    #[test]
    fn block_defender_id_stored() {
        let mut step = StepAnimalSavagery::default();
        step.set_parameter(&StepParameter::BlockDefenderId("def1".into()));
        assert_eq!(step.state.block_defender_id.as_deref(), Some("def1"));
    }

    #[test]
    fn thrown_player_id_stored() {
        let mut step = StepAnimalSavagery::default();
        step.set_parameter(&StepParameter::ThrownPlayerId(Some("thrown1".into())));
        assert_eq!(step.state.thrown_player_id.as_deref(), Some("thrown1"));
    }

    #[test]
    fn end_turn_stored() {
        let mut step = StepAnimalSavagery::default();
        step.set_parameter(&StepParameter::EndTurn(true));
        assert!(step.state.end_turn);
    }

    #[test]
    fn player_choice_animal_savagery_sets_player_id() {
        let mut step = StepAnimalSavagery::new("fail");
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        step.handle_command(
            &Action::PlayerChoice {
                player_id: Some("p1".into()),
                player_ids: vec![],
                mode: "ANIMAL_SAVAGERY".into(),
            },
            &mut game,
            &mut rng,
        );
        assert_eq!(step.state.player_id.as_deref(), Some("p1"));
    }

    #[test]
    fn target_coordinate_resolves_to_catcher_id_in_start() {
        use ffb_model::types::FieldCoordinate;
        use ffb_model::model::player::Player;

        let mut step = StepAnimalSavagery::new("fail");
        let mut game = make_game();
        let mut rng = GameRng::new(0);

        // Place a player at a known coordinate
        let player = Player { id: "catcher1".into(), ..Default::default() };
        game.team_home.players.push(player);
        let coord = FieldCoordinate::new(5, 5);
        game.field_model.set_player_coordinate("catcher1", coord);

        // Provide the coordinate via set_parameter
        assert!(step.set_parameter(&StepParameter::TargetCoordinate(coord)));
        // Before start(), catcher_id is not yet resolved
        assert!(step.state.catcher_id.is_none());

        step.start(&mut game, &mut rng);
        assert_eq!(step.state.catcher_id.as_deref(), Some("catcher1"));
    }

    #[test]
    fn use_skill_lash_out_sets_attack_opponent() {
        let mut step = StepAnimalSavagery::new("fail");
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        step.handle_command(
            &Action::UseSkill { skill_id: SkillId::PrimalSavagery, use_skill: true },
            &mut game,
            &mut rng,
        );
        assert_eq!(step.state.attack_opponent, Some(true));
    }
}
