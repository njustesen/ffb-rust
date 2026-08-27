/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.StepPro`.
///
/// Handles the Pro skill re-roll (BB2020 + BB2025).
/// Given a `PLAYER_ID` init parameter, the step:
///   1. If a re-roll source is set, attempts to consume it (unsets `usedPro` on the player).
///   2. Rolls the Pro die (D6 >= 4 = success).
///   3. If unsuccessful and OLD_PRO wasn't already re-rolled, asks for a team/skill re-roll.
///   4. Publishes `SUCCESSFUL_PRO(bool)` and advances.
///
/// Java: `StepPro extends AbstractStepWithReRoll` (mixed, BB2020 + BB2025).
use ffb_model::enums::ReRollSource;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::abstract_step_with_re_roll::ReRollState;
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

// Java: RollMechanic.minimumProRoll() = 3 for BB2020/BB2025 (this step's editions).
// Only the bb2016 RollMechanic returns 4; StepPro is annotated
// @RulesCollection(BB2020) @RulesCollection(BB2025), so 3 is correct here.
const MINIMUM_PRO_ROLL: i32 = 3;
const OLD_PRO: &str = "OLD_PRO";

/// Java: `StepPro` (mixed, BB2020 + BB2025).
pub struct StepPro {
    /// Java: `playerId` (init param PLAYER_ID)
    pub player_id: Option<String>,
    /// AbstractStepWithReRoll state
    pub re_roll: ReRollState,
}

impl StepPro {
    pub fn new() -> Self {
        Self { player_id: None, re_roll: ReRollState::new() }
    }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let player_id = match &self.player_id {
            Some(id) => id.clone(),
            None => return StepOutcome::next(),
        };

        if game.player(&player_id).is_none() {
            return StepOutcome::next();
        }

        let mut do_roll = true;
        let mut successful = false;

        // Java: if (getReRollSource() != null) { if (useReRoll(source, player)) { changeUsedPro(false) } else doRoll=false }
        if let Some(ref source) = self.re_roll.re_roll_source.clone() {
            if use_reroll(game, source, &player_id, rng) {
                // Reset usedPro flag so the Pro roll can proceed
                if let Some(state) = game.field_model.player_state(&player_id) {
                    game.field_model.set_player_state(&player_id, state.change_used_pro(false));
                }
            } else {
                do_roll = false;
            }
        }

        // Java: if (doRoll) { successful = useReRoll(this, ReRollSources.PRO, player) }
        // RollMechanic.useReRoll's PRO branch first checks `canRerollOncePerTurn &&
        // !playerState.hasUsedPro()` — only then does it mark usedPro and roll. Without the
        // gate, a DECLINED team-reroll offer re-entered executeStep with doRoll=true and
        // rolled a SECOND Pro die Java never rolls (dark_elf bb2020 seed 3 i=2: the first
        // Pro roll already set the usedPro bit, so Java's re-execute is roll-free).
        if do_roll {
            use ffb_model::model::property::named_properties::NamedProperties;
            let state = game.field_model.player_state(&player_id).unwrap_or_default();
            let available = game.player(&player_id)
                .map(|p| p.has_skill_property(NamedProperties::CAN_REROLL_ONCE_PER_TURN))
                .unwrap_or(false)
                && !state.has_used_pro();
            if available {
                game.field_model.set_player_state(&player_id, state.change_used_pro(true));
                let roll = rng.d6();
                successful = roll >= MINIMUM_PRO_ROLL;
            }
        }

        // Java: if (!successful && getReRolledAction() != OLD_PRO) → ask for reroll
        let already_rerolled = self.re_roll.re_rolled_action.as_ref()
            .map(|a| a.name == OLD_PRO)
            .unwrap_or(false);

        if !successful && !already_rerolled {
            if let Some(prompt) = ask_for_reroll_if_available(game, OLD_PRO, MINIMUM_PRO_ROLL, false) {
                self.re_roll.re_rolled_action = Some(ffb_model::model::re_rolled_action::ReRolledAction::new(OLD_PRO));
                self.re_roll.re_roll_source = Some(ReRollSource::new("TRR"));
                return StepOutcome::cont().with_prompt(prompt);
            }
        }

        StepOutcome::next().publish(StepParameter::SuccessfulPro(successful))
    }
}

impl Default for StepPro {
    fn default() -> Self { Self::new() }
}

impl Step for StepPro {
    fn id(&self) -> StepId { StepId::Pro }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::UseReRoll { use_reroll: false } => {
                self.re_roll.re_roll_source = None;
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::PlayerId(v) => { self.player_id = Some(v.clone()); true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{test_team, StepAction};
    use ffb_model::enums::{Rules, PS_STANDING, PlayerState, PlayerAction};
    use ffb_model::model::game::Game;
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_player(game: &mut Game, id: &str) {
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            // Java useReRoll(PRO) requires canRerollOncePerTurn — the step's roll is gated on it.
            starting_skills: vec![ffb_model::model::skill_def::SkillWithValue::new(ffb_model::enums::SkillId::Pro)],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
});
        game.field_model.set_player_coordinate(id, FieldCoordinate::new(5, 5));
        game.field_model.set_player_state(id, PlayerState::new(PS_STANDING));
        game.acting_player.set_player(id.into(), PlayerAction::Block);
    }

    #[test]
    fn no_player_id_returns_next_step() {
        let mut step = StepPro::new();
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn roll_4_or_more_publishes_successful_pro_true() {
        let mut step = StepPro::new();
        step.player_id = Some("p1".into());
        let mut game = make_game();
        add_player(&mut game, "p1");
        // Use a seed that gives d6 = 4 or more
        // Seed 5 gives d6 >= 4 in GameRng
        let mut rng = GameRng::new(5);
        let roll = rng.d6();
        let succeeds = roll >= MINIMUM_PRO_ROLL;
        let mut rng2 = GameRng::new(5);
        let out = step.start(&mut game, &mut rng2);
        if succeeds {
            assert!(out.published.iter().any(|p| matches!(p, StepParameter::SuccessfulPro(true))));
        } else {
            // Still publishes SuccessfulPro(false) when no reroll available
            assert!(out.published.iter().any(|p| matches!(p, StepParameter::SuccessfulPro(false))));
        }
    }

    #[test]
    fn minimum_pro_roll_is_three_not_four() {
        // Java: RollMechanic.minimumProRoll() returns 3 for BB2020 and BB2025;
        // only BB2016 returns 4, and StepPro is BB2020/BB2025-only.
        assert_eq!(MINIMUM_PRO_ROLL, 3);
    }

    #[test]
    fn used_pro_is_marked_true_regardless_of_roll_outcome() {
        // Java: RollMechanic.useReRoll sets `changeUsedPro(true)` unconditionally
        // before rolling the Pro skill die, whether the roll subsequently
        // succeeds or fails.
        for seed in 0..20u64 {
            let mut step = StepPro::new();
            step.player_id = Some("p1".into());
            let mut game = make_game();
            add_player(&mut game, "p1");
            let mut rng = GameRng::new(seed);
            step.start(&mut game, &mut rng);
            let state = game.field_model.player_state("p1").expect("player state");
            assert!(state.has_used_pro(), "seed {seed}: used_pro must be true after a Pro roll attempt");
        }
    }

    #[test]
    fn set_parameter_player_id() {
        let mut step = StepPro::new();
        let accepted = step.set_parameter(&StepParameter::PlayerId("pid".into()));
        assert!(accepted);
        assert_eq!(step.player_id, Some("pid".into()));
    }

    #[test]
    fn set_parameter_rejects_unknown() {
        let mut step = StepPro::new();
        let rejected = !step.set_parameter(&StepParameter::EndTurn(true));
        assert!(rejected);
    }

    #[test]
    fn decline_reroll_clears_source() {
        let mut step = StepPro::new();
        step.player_id = Some("p1".into());
        step.re_roll.re_roll_source = Some(ReRollSource::new("TRR"));
        let mut game = make_game();
        add_player(&mut game, "p1");
        step.handle_command(&Action::UseReRoll { use_reroll: false }, &mut game, &mut GameRng::new(0));
        assert!(step.re_roll.re_roll_source.is_none());
    }
}
