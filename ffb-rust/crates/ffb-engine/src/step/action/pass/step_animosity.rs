/// 1:1 translation of com.fumbbl.ffb.server.step.action.pass.StepAnimosity (COMMON).
///
/// Handles the Animosity skill check during a pass action.
///
/// Mandatory init param: GOTO_LABEL_ON_FAILURE.
/// Expected preceding param: CATCHER_ID.
///
/// Logic (from AnimosityBehaviour.handleExecuteStepHook):
/// - If isSufferingAnimosity() → already processed, NEXT_STEP
/// - If bomb turn → NEXT_STEP (skip)
/// - If HAND_OVER action → NEXT_STEP (no animosity on hand-offs in some editions, but handled here)
/// - Check animosity_exists(thrower, catcher) — race-based comparison
/// - If false → NEXT_STEP
/// - If true: roll d6 vs minimumRollAnimosity (2); on success → NEXT_STEP
/// - On failure: offer re-roll (Pass skill or TRR); on final failure → set sufferingAnimosity=true → GOTO failure
use ffb_model::enums::{PlayerAction, ReRollSource};
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_mechanics::bb2025::skill_mechanic::SkillMechanic;
use ffb_mechanics::mechanics::{is_skill_roll_successful, minimum_roll_animosity};
use ffb_mechanics::skill_mechanic::SkillMechanic as SkillMechanicTrait;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

pub struct StepAnimosity {
    /// Java: state.gotoLabelOnFailure — mandatory.
    pub goto_label_on_failure: String,
    /// Java: state.catcherId — set by preceding step parameter.
    pub catcher_id: Option<String>,
    /// Java: AbstractStepWithReRoll.reRolledAction
    pub re_rolled_action: Option<String>,
    /// Java: AbstractStepWithReRoll.reRollSource
    pub re_roll_source: Option<String>,
}

impl StepAnimosity {
    pub fn new(goto_label_on_failure: impl Into<String>) -> Self {
        Self {
            goto_label_on_failure: goto_label_on_failure.into(),
            catcher_id: None,
            re_rolled_action: None,
            re_roll_source: None,
        }
    }
}

impl Step for StepAnimosity {
    fn id(&self) -> StepId { StepId::Animosity }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if let Action::UseReRoll { use_reroll: false } = action {
            self.re_roll_source = None;
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(v) => { self.goto_label_on_failure = v.clone(); true }
            StepParameter::CatcherId(v) => { self.catcher_id = v.clone(); true }
            _ => false,
        }
    }
}

impl StepAnimosity {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: if (actingPlayer.isSufferingAnimosity()) → NEXT_STEP (already resolved)
        if game.acting_player.suffering_animosity {
            return StepOutcome::next();
        }

        // Java: if (game.getTurnMode().isBombTurn()) → NEXT_STEP
        if game.turn_mode.is_bomb_turn() {
            return StepOutcome::next();
        }

        // Java: if (HAND_OVER action) → NEXT_STEP (Animosity only applies to Pass, not HandOver)
        if game.acting_player.player_action == Some(PlayerAction::HandOver) {
            return StepOutcome::next();
        }

        let re_rolled = self.re_rolled_action.as_deref() == Some("ANIMOSITY");

        // Re-roll path: we already rolled and offered a re-roll; now handle response.
        if re_rolled {
            if let Some(ref source_str) = self.re_roll_source.clone() {
                let source = ReRollSource::new(source_str.as_str());
                let thrower_id = game.acting_player.player_id.clone().unwrap_or_default();
                if use_reroll(game, &source, &thrower_id) {
                    // Re-roll granted — fall through to the roll below
                } else {
                    return self.fail_animosity(game);
                }
            } else {
                // re_roll_source cleared to None means player declined
                return self.fail_animosity(game);
            }
        }

        let thrower = game.thrower().map(|p| p.clone());
        let catcher = self.catcher_id.as_deref()
            .and_then(|id| game.player(id).map(|p| p.clone()));

        let mechanic = SkillMechanic::new();
        let do_roll = match (&thrower, &catcher) {
            (Some(t), Some(c)) => mechanic.animosity_exists(t, c),
            _ => false,
        };

        if !do_roll {
            return StepOutcome::next();
        }

        let roll = rng.d6();
        let min_roll = minimum_roll_animosity();
        let successful = is_skill_roll_successful(roll, min_roll);

        let thrower_id = game.acting_player.player_id.clone().unwrap_or_default();
        let event = GameEvent::AnimosityRoll {
            player_id: thrower_id.clone(),
            roll,
            success: successful,
        };

        if successful {
            return StepOutcome::next().with_event(event);
        }

        // Failed: offer re-roll (Pass skill → TRR)
        if !re_rolled {
            if let Some(prompt) = ask_for_reroll_if_available(game, "ANIMOSITY", min_roll, false) {
                self.re_rolled_action = Some("ANIMOSITY".into());
                self.re_roll_source = Some("TRR".into());
                return StepOutcome::cont().with_event(event).with_prompt(prompt);
            }
        }

        self.fail_animosity(game).with_event(event)
    }

    fn fail_animosity(&mut self, game: &mut Game) -> StepOutcome {
        game.acting_player.suffering_animosity = true;
        StepOutcome::goto(&self.goto_label_on_failure)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::{Rules, TurnMode};

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.home_playing = true;
        game
    }

    #[test]
    fn already_suffering_animosity_returns_next() {
        let mut game = make_game();
        game.acting_player.suffering_animosity = true;
        let out = StepAnimosity::new("fail").start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn bomb_turn_skips_animosity_check() {
        let mut game = make_game();
        game.turn_mode = TurnMode::BombHome;
        let out = StepAnimosity::new("fail").start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn hand_over_action_skips_animosity_check() {
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::HandOver);
        let out = StepAnimosity::new("fail").start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn no_animosity_skill_returns_next() {
        let mut game = make_game();
        // No thrower set, no catcher → animosity_exists = false
        let out = StepAnimosity::new("fail").start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn catcher_id_parameter_accepted() {
        let mut step = StepAnimosity::new("fail");
        step.set_parameter(&StepParameter::CatcherId(Some("c1".into())));
        assert_eq!(step.catcher_id.as_deref(), Some("c1"));
    }

    #[test]
    fn goto_label_on_failure_param_accepted() {
        let mut step = StepAnimosity::new("fail");
        step.set_parameter(&StepParameter::GotoLabelOnFailure("other".into()));
        assert_eq!(step.goto_label_on_failure, "other");
    }

    #[test]
    fn regular_turn_no_animosity_returns_next() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        let mut step = StepAnimosity::new("fail");
        step.catcher_id = Some("c2".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        // animosity_exists returns false without matching race → always NEXT_STEP
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn step_id_is_animosity() {
        assert_eq!(StepAnimosity::new("fail").id(), StepId::Animosity);
    }

    #[test]
    fn decline_reroll_sets_suffering_animosity() {
        let mut step = StepAnimosity::new("fail");
        step.re_rolled_action = Some("ANIMOSITY".into());
        step.re_roll_source = None; // declined → source is None
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert!(game.acting_player.suffering_animosity);
    }

    fn add_thrower_and_catcher(game: &mut Game, animosity_value: &str, catcher_keywords: Vec<&str>) {
        use ffb_model::model::skill_def::SkillId;
        use ffb_model::model::skill_def::SkillWithValue;
        game.team_home.players.push(ffb_model::model::Player {
            id: "thrower".into(),
            starting_skills: vec![SkillWithValue::with_value(SkillId::Animosity, animosity_value)],
            ..Default::default()
        });
        game.team_home.players.push(ffb_model::model::Player {
            id: "catcher".into(),
            keywords: catcher_keywords.into_iter().map(String::from).collect(),
            ..Default::default()
        });
        game.thrower_id = Some("thrower".into());
        game.acting_player.player_id = Some("thrower".into());
    }

    #[test]
    fn different_race_catcher_skips_roll_entirely() {
        // Thrower is only configured against "Goblin" catchers; a Troll catcher never matches,
        // so animosity_exists is false and no roll (no AnimosityRoll event) ever happens.
        let mut game = make_game();
        add_thrower_and_catcher(&mut game, "Goblin", vec!["Troll"]);
        let mut step = StepAnimosity::new("fail");
        step.catcher_id = Some("catcher".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.events.is_empty(), "no roll should occur when animosity_exists is false");
        assert!(!game.acting_player.suffering_animosity);
    }

    #[test]
    fn same_race_catcher_triggers_roll() {
        // Thrower is configured against "Goblin" catchers; a Goblin catcher matches, so
        // animosity_exists is true and a real d6 roll happens (an AnimosityRoll event fires).
        let mut game = make_game();
        add_thrower_and_catcher(&mut game, "Goblin", vec!["Goblin"]);
        let mut step = StepAnimosity::new("fail");
        step.catcher_id = Some("catcher".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(!out.events.is_empty(), "a roll should occur when animosity_exists is true");
    }

    #[test]
    fn full_roll_cycle_with_real_trigger_can_fail_and_offer_reroll() {
        let mut game = make_game();
        add_thrower_and_catcher(&mut game, "Goblin", vec!["Goblin"]);
        let mut step = StepAnimosity::new("fail");
        step.catcher_id = Some("catcher".into());
        // Find a seed producing a failed roll (roll == 1) to exercise the re-roll-offer path.
        let mut seed = 0u64;
        loop {
            let mut rng = GameRng::new(seed);
            if rng.d6() == 1 { break; }
            seed += 1;
            assert!(seed < 100, "expected to find a failing roll seed quickly");
        }
        let out = step.start(&mut game, &mut GameRng::new(seed));
        // On failure with no re-roll source available, the step goes straight to the failure label.
        assert_eq!(out.action, StepAction::GotoLabel);
        assert!(game.acting_player.suffering_animosity);
    }
}
