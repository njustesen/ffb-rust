/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.foul.StepFoulChainsaw`.
///
/// Step in foul sequence to handle skill CHAINSAW (BB2016).
/// - If actor does NOT have blocksLikeChainsaw property → NEXT_STEP.
/// - Rolls chainsaw (D6 vs minimum 2).
/// - On success: publish USING_CHAINSAW + NEXT_STEP.
/// - On failure: ask for re-roll (CHAINSAW) if available.
/// - If re-roll exhausted: apply InjuryTypeChainsaw to the attacker → goto failure.
///
/// Init parameter: GOTO_LABEL_ON_FAILURE (mandatory).
/// Publishes: USING_CHAINSAW, END_TURN, INJURY_RESULT.
///
use ffb_model::enums::{ApothecaryMode, ReRollSource};
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::re_rolled_action::ReRolledAction;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::injury::InjuryTypeChainsawImpl;
use crate::step::abstract_step_with_re_roll::ReRollState;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};
use crate::step::util_server_injury::{drop_player, handle_injury};

/// Java: `StepFoulChainsaw` (bb2016/foul).
pub struct StepFoulChainsaw {
    /// Java: `fGotoLabelOnFailure` — mandatory init param.
    goto_label_on_failure: String,
    /// Java: re-roll state (extends AbstractStepWithReRoll)
    re_roll_state: ReRollState,
    /// Java: local roll variable (0 = not yet rolled)
    roll: i32,
}

impl StepFoulChainsaw {
    pub fn new() -> Self {
        Self {
            goto_label_on_failure: String::new(),
            re_roll_state: ReRollState::default(),
            roll: 0,
        }
    }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let acting_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        let has_chainsaw = game.player(&acting_id)
            .map(|p| p.has_skill_property(NamedProperties::BLOCKS_LIKE_CHAINSAW))
            .unwrap_or(false);
        if !has_chainsaw {
            return StepOutcome::next();
        }

        let mut drop_chainsaw_player = false;
        let mut pending_event: Option<GameEvent> = None;

        // Java: if (CHAINSAW == getReRolledAction()) { if (source==null || !useReRoll) drop=true }
        let already_rerolled = self.re_roll_state.re_rolled_action
            .as_ref().map(|a| a.name == "CHAINSAW").unwrap_or(false);
        if already_rerolled {
            let consumed = self.re_roll_state.re_roll_source.as_ref()
                .map(|s| use_reroll(game, s, &acting_id))
                .unwrap_or(false);
            if !consumed {
                drop_chainsaw_player = true;
            }
        }

        if !drop_chainsaw_player {
            if self.roll == 0 {
                self.roll = rng.d6(); // Java: rollChainsaw()
            }
            // Java: minimumRollChainsaw() = 2
            let minimum_roll = 2;
            let successful = self.roll >= minimum_roll;
            let rerolled = already_rerolled && self.re_roll_state.re_roll_source.is_some();

            // Java: getResult().addReport(new ReportChainsawRoll(actingPlayer.getPlayerId(), successful, roll, minimumRoll, reRolled, null))
            let chainsaw_event = GameEvent::ChainsawRoll {
                player_id: acting_id.clone(),
                roll: self.roll,
                minimum_roll,
                success: successful,
                rerolled,
            };

            if successful {
                return StepOutcome::next()
                    .with_event(chainsaw_event)
                    .publish(StepParameter::UsingChainsaw(true));
            } else {
                // Ask for re-roll
                if let Some(prompt) = ask_for_reroll_if_available(game, "CHAINSAW", minimum_roll, false) {
                    self.re_roll_state.re_rolled_action = Some(ReRolledAction::new("CHAINSAW"));
                    self.re_roll_state.re_roll_source = Some(ReRollSource::new("TRR"));
                    self.roll = 0;
                    return StepOutcome::cont().with_event(chainsaw_event).with_prompt(prompt);
                }
                pending_event = Some(chainsaw_event);
                drop_chainsaw_player = true;
            }
        }

        if drop_chainsaw_player {
            let attacker_coord = game.field_model.player_coordinate(&acting_id)
                .unwrap_or(ffb_model::types::FieldCoordinate::new(0, 0));
            let mut injury_type = InjuryTypeChainsawImpl::new();
            let injury_result = handle_injury(
                game, rng, &mut injury_type,
                None, &acting_id,
                attacker_coord, None, None, ApothecaryMode::Attacker,
            );
            let mut outcome = StepOutcome::goto(&self.goto_label_on_failure);
            if let Some(ev) = pending_event { outcome = outcome.with_event(ev); }
            if injury_result.injury_context().armor_broken {
                for p in drop_player(game, &acting_id, false) { outcome = outcome.publish(p); }
                outcome = outcome.publish(StepParameter::EndTurn(true));
            }
            outcome = outcome.publish(StepParameter::InjuryResult(Box::new(injury_result)));
            return outcome;
        }

        StepOutcome::cont()
    }
}

impl Default for StepFoulChainsaw {
    fn default() -> Self { Self::new() }
}

impl Step for StepFoulChainsaw {
    fn id(&self) -> StepId { StepId::FoulChainsaw }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if let Action::UseReRoll { use_reroll: false } = action {
            self.re_roll_state.re_roll_source = None;
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(s) => { self.goto_label_on_failure = s.clone(); true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{Rules, PS_STANDING, PlayerType, PlayerGender};
    use ffb_model::model::player::Player;
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    fn add_player(game: &mut Game, id: &str) {
        let p = Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
        };
        game.team_home.players.push(p);
        game.field_model.set_player_coordinate(id, FieldCoordinate::new(5, 5));
        game.field_model.set_player_state(id, ffb_model::enums::PlayerState::new(PS_STANDING));
        game.acting_player.player_id = Some(id.into());
    }

    #[test]
    fn no_acting_player_returns_next() {
        let mut game = make_game();
        let mut step = StepFoulChainsaw::new();
        step.goto_label_on_failure = "fail".into();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn player_without_chainsaw_returns_next() {
        let mut game = make_game();
        add_player(&mut game, "p1");
        // No chainsaw skill → NEXT_STEP
        let mut step = StepFoulChainsaw::new();
        step.goto_label_on_failure = "fail".into();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_goto_label_on_failure() {
        let mut step = StepFoulChainsaw::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("fail".into())));
        assert_eq!(step.goto_label_on_failure, "fail");
    }

    #[test]
    fn id_is_foul_chainsaw() {
        assert_eq!(StepFoulChainsaw::new().id(), StepId::FoulChainsaw);
    }
}
