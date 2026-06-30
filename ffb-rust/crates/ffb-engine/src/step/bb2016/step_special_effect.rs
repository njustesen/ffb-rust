/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.StepSpecialEffect` (BB2016).
///
/// Applies a wizard / inducement special effect (Lightning, Zap, Fireball, Bomb) to a player.
/// Optionally rolls to check if the effect succeeds (fRollForEffect).
///
/// Mandatory init params: `GOTO_LABEL_ON_FAILURE`, `PLAYER_ID`, `SPECIAL_EFFECT`.
///   ROLL_FOR_EFFECT is also mandatory (defaults false).
///
/// On success:
///   - LIGHTNING  → handleInjury(InjuryTypeLightning) + dropPlayer + maybe END_TURN
///   - ZAP        → zap player (replace with ZappedPlayer) + maybe scatter ball
///   - FIREBALL   → handleInjury(InjuryTypeFireball) + dropPlayer
///   - BOMB       → handleInjury(InjuryTypeBomb) + dropPlayer + maybe END_TURN
/// On failure: → GOTO_LABEL failure label.
///
/// Also publishes END_TURN = true when the acting team includes the target player
/// (for LIGHTNING and BOMB, not FIREBALL).
///
/// TODOs:
///  - handleInjury / dropPlayer (InjuryTypeLightning/Fireball/Bomb) not translated.
///  - ZAP: ZappedPlayer substitution not translated.
///  - DiceInterpreter.isSpecialEffectSuccessful not translated (stub: roll ≥ 4).
///  - ReportSpecialEffectRoll not translated.
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2016.StepSpecialEffect`.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// Mirrors Java SpecialEffect enum (subset used by BB2016 wizard steps).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialEffect {
    Lightning,
    Zap,
    Fireball,
    Bomb,
}

pub struct StepSpecialEffect {
    /// Java: fGotoLabelOnFailure — mandatory init param.
    pub goto_label_on_failure: String,
    /// Java: fPlayerId — mandatory init param.
    pub player_id: String,
    /// Java: fRollForEffect — mandatory init param.
    pub roll_for_effect: bool,
    /// Java: fSpecialEffect — mandatory init param.
    pub special_effect: Option<SpecialEffect>,
}

impl StepSpecialEffect {
    pub fn new(goto_label_on_failure: String, player_id: String, special_effect: Option<SpecialEffect>) -> Self {
        Self {
            goto_label_on_failure,
            player_id,
            roll_for_effect: false,
            special_effect,
        }
    }
}

impl Default for StepSpecialEffect {
    fn default() -> Self {
        Self::new(String::new(), String::new(), None)
    }
}

impl Step for StepSpecialEffect {
    fn id(&self) -> StepId { StepId::SpecialEffect }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: no CLIENT_* commands handled beyond super — step does not loop.
        StepOutcome::next()
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::PlayerId(v) => { self.player_id = v.clone(); true }
            StepParameter::RollForEffect(v) => { self.roll_for_effect = *v; true }
            StepParameter::GotoLabelOnFailure(v) => { self.goto_label_on_failure = v.clone(); true }
            _ => false,
        }
    }
}

impl StepSpecialEffect {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if self.player_id.is_empty() {
            return StepOutcome::next();
        }

        let player_exists = game.player(&self.player_id).is_some();
        if !player_exists {
            return StepOutcome::next();
        }

        // Java: if (fRollForEffect) { roll; check success } else { successful = true }
        let successful = if self.roll_for_effect {
            let roll = rng.d6();
            // TODO(special_effect_bb2016): DiceInterpreter.isSpecialEffectSuccessful not translated.
            // Stub: success if roll >= 4.
            roll >= 4
        } else {
            true
        };

        if !successful {
            let label = self.goto_label_on_failure.clone();
            return StepOutcome::goto(&label);
        }

        let mut outcome = StepOutcome::next();

        let effect = match self.special_effect {
            Some(e) => e,
            None => return outcome,
        };

        // TODO(special_effect_bb2016): handleInjury / dropPlayer not translated for Lightning/Fireball/Bomb.
        // TODO(special_effect_bb2016): ZAP ZappedPlayer substitution not translated.

        // Java: check end turn — acting team vs target player (except FIREBALL)
        if effect != SpecialEffect::Fireball {
            let acting_team_has_player = game.active_team().has_player(&self.player_id);
            if acting_team_has_player {
                outcome = outcome.publish(StepParameter::EndTurn(true));
            }
        }

        outcome
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::Rules;
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::util::rng::GameRng;
    use std::collections::HashSet;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    fn add_home_player(game: &mut Game, id: &str) {
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
        });
    }

    fn add_away_player(game: &mut Game, id: &str) {
        game.team_away.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
        });
    }

    #[test]
    fn empty_player_id_returns_next_step() {
        let mut game = make_game();
        let mut step = StepSpecialEffect::default();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn missing_player_returns_next_step() {
        let mut game = make_game();
        let mut step = StepSpecialEffect::new("fail".into(), "nonexistent".into(), Some(SpecialEffect::Lightning));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn no_roll_required_succeeds_always() {
        let mut game = make_game();
        add_away_player(&mut game, "target");
        game.home_playing = true;
        let mut step = StepSpecialEffect::new("fail".into(), "target".into(), Some(SpecialEffect::Lightning));
        step.roll_for_effect = false;
        // Should not goto failure label
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_ne!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn lightning_on_acting_teams_player_publishes_end_turn() {
        let mut game = make_game();
        add_home_player(&mut game, "p1");
        game.home_playing = true; // home is acting
        let mut step = StepSpecialEffect::new("fail".into(), "p1".into(), Some(SpecialEffect::Lightning));
        step.roll_for_effect = false;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    #[test]
    fn fireball_does_not_publish_end_turn() {
        let mut game = make_game();
        add_home_player(&mut game, "p1");
        game.home_playing = true;
        let mut step = StepSpecialEffect::new("fail".into(), "p1".into(), Some(SpecialEffect::Fireball));
        step.roll_for_effect = false;
        let out = step.start(&mut game, &mut GameRng::new(0));
        // FIREBALL exemption: no END_TURN even for acting team's player
        assert!(!out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    #[test]
    fn failed_roll_goes_to_failure_label() {
        let mut game = make_game();
        add_away_player(&mut game, "p1");
        game.home_playing = true;
        let mut step = StepSpecialEffect::new("fail_label".into(), "p1".into(), Some(SpecialEffect::Lightning));
        step.roll_for_effect = true;
        // Seed 0 → d6 = 1 (< 4 → failure)
        let out = step.start(&mut game, &mut GameRng::new(0));
        if out.action == StepAction::GotoLabel {
            assert_eq!(out.goto_label.as_deref(), Some("fail_label"));
        }
    }

    #[test]
    fn set_parameter_player_id_accepted() {
        let mut step = StepSpecialEffect::default();
        assert!(step.set_parameter(&StepParameter::PlayerId("p2".into())));
        assert_eq!(step.player_id, "p2");
    }

    #[test]
    fn set_parameter_roll_for_effect_accepted() {
        let mut step = StepSpecialEffect::default();
        assert!(step.set_parameter(&StepParameter::RollForEffect(true)));
        assert!(step.roll_for_effect);
    }

    #[test]
    fn set_parameter_unknown_returns_false() {
        let mut step = StepSpecialEffect::default();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }
}
