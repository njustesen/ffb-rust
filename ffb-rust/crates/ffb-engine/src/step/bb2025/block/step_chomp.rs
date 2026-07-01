use ffb_model::enums::ReRollSource;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.block.StepChomp.
///
/// Handles the Chomp skill (`canPinPlayers`) for the Old World Ogre.
///
/// Execution flow (Java `executeStep`):
/// 1. Default to `NEXT_STEP`.
/// 2. If attacker has `canPinPlayers`, `using_chomp == true`, and attacker is NOT standing up:
///    a. Override to `GOTO_LABEL gotoLabelOnEnd` (defender is ignored, block sequence ends).
///    b. If re-roll accepted (re-rolled action == CHOMP): use re-roll or return (no further roll).
///    c. Roll d6; minimum_roll = 3.
///    d. On success: `game.fieldModel.addChomp(attacker, defender)` → keeps GOTO_LABEL.
///    e. On failure: if not yet re-rolled, ask for re-roll (TODO). If re-roll available → `CONTINUE`.
///
/// `game.field_model.add_chomp` wired; re-roll wired via util_server_re_roll;
/// `actingPlayer.standing_up` guard wired.
pub struct StepChomp {
    pub goto_label_on_end: String,
    pub using_chomp: bool,
    // AbstractStepWithReRoll stub
    pub re_rolled_action: Option<String>,
    pub re_roll_source: Option<String>,
}

impl StepChomp {
    pub fn new(goto_label_on_end: String) -> Self {
        Self {
            goto_label_on_end,
            using_chomp: false,
            re_rolled_action: None,
            re_roll_source: None,
        }
    }
}

impl Step for StepChomp {
    fn id(&self) -> StepId { StepId::Chomp }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::UseReRoll { use_reroll: false } => {
                self.re_roll_source = None;
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::UsingChomp(v) => { self.using_chomp = *v; true }
            StepParameter::GotoLabelOnEnd(v) => { self.goto_label_on_end = v.clone(); true }
            _ => false,
        }
    }
}

impl StepChomp {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: getResult().setNextAction(NEXT_STEP) — default
        let attacker_has_chomp = game.acting_player.player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.has_skill_property(NamedProperties::CAN_PIN_PLAYERS))
            .unwrap_or(false);

        let is_standing_up = game.acting_player.standing_up;

        if !attacker_has_chomp || !self.using_chomp || is_standing_up {
            return StepOutcome::next();
        }

        // Java: getResult().setNextAction(GOTO_LABEL, gotoLabelOnEnd)
        // The block sequence is skipped; we jump to the end label regardless.
        let label = self.goto_label_on_end.clone();

        let attacker_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::goto(&label),
        };

        // Java: if (CHOMP == reRolledAction) { if (source != null && useReRoll) → fall through to roll }
        if self.re_rolled_action.as_deref() == Some("CHOMP") {
            if let Some(ref source_name) = self.re_roll_source.clone() {
                let source = ReRollSource::new(source_name.as_str());
                if !use_reroll(game, &source, &attacker_id) {
                    return StepOutcome::goto(&label);
                }
            } else {
                // Player declined re-roll
                return StepOutcome::goto(&label);
            }
        }

        let roll = rng.d6();
        let minimum_roll = 3;
        let successful = roll >= minimum_roll;

        if successful {
            // Java: game.getFieldModel().addChomp(actingPlayer.getPlayer(), game.getDefender())
            if let Some(defender_id) = game.defender_id.clone() {
                game.field_model.add_chomp(&attacker_id, &defender_id);
            }
            StepOutcome::goto(&label)
        } else if self.re_rolled_action.is_none() {
            // Java: UtilServerReRoll.askForReRollIfAvailable(state, player, CHOMP, minimumRoll, false)
            if let Some(prompt) = ask_for_reroll_if_available(game, "CHOMP", minimum_roll, false) {
                self.re_rolled_action = Some("CHOMP".into());
                self.re_roll_source = Some("TRR".into());
                return StepOutcome::cont().with_prompt(prompt);
            }
            StepOutcome::goto(&label)
        } else {
            // Already offered re-roll and still failed
            StepOutcome::goto(&label)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn not_using_chomp_returns_next_step() {
        let mut step = StepChomp::new("end".into());
        step.using_chomp = false;
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(1));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn using_chomp_but_no_skill_returns_next_step() {
        let mut step = StepChomp::new("end".into());
        step.using_chomp = true;
        // Player not in game → no canPinPlayers → next_step
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(1));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_using_chomp_accepted() {
        let mut step = StepChomp::new("end".into());
        assert!(!step.using_chomp);
        step.set_parameter(&StepParameter::UsingChomp(true));
        assert!(step.using_chomp);
    }

    #[test]
    fn set_parameter_goto_label_on_end_accepted() {
        let mut step = StepChomp::new("end".into());
        step.set_parameter(&StepParameter::GotoLabelOnEnd("new_end".into()));
        assert_eq!(step.goto_label_on_end, "new_end");
    }

    #[test]
    fn decline_reroll_clears_re_roll_source() {
        let mut step = StepChomp::new("end".into());
        step.re_rolled_action = Some("CHOMP".into());
        step.re_roll_source = Some("TRR".into());
        let mut game = make_game();
        step.handle_command(
            &Action::UseReRoll { use_reroll: false },
            &mut game,
            &mut GameRng::new(0),
        );
        assert!(step.re_roll_source.is_none(), "declined re-roll should clear source");
    }
}
