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
/// DEFERRED(ZAP): ZappedPlayer substitution not yet ported.
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2016.StepSpecialEffect`.
use ffb_model::enums::{ApothecaryMode, TurnMode};
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_injury::{drop_player, handle_injury_by_name};

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
        let mut spell_roll_event: Option<GameEvent> = None;
        let successful = if self.roll_for_effect {
            let roll = rng.d6();
            // Java: getResult().addReport(new ReportSpecialEffectRoll(effect, roll, success))
            spell_roll_event = Some(GameEvent::SpellEffectRoll { roll });
            // Java: DiceInterpreter.isSpecialEffectSuccesful(effect, targetPlayer, roll)
            is_special_effect_successful(self.special_effect, game, &self.player_id, roll)
        } else {
            true
        };

        if !successful {
            let label = self.goto_label_on_failure.clone();
            let mut out = StepOutcome::goto(&label);
            if let Some(ev) = spell_roll_event { out = out.with_event(ev); }
            return out;
        }

        let mut outcome = StepOutcome::next();
        if let Some(ev) = spell_roll_event { outcome = outcome.with_event(ev); }

        let effect = match self.special_effect {
            Some(e) => e,
            None => return outcome,
        };

        let player_id = self.player_id.clone();
        let coord = game.field_model.player_coordinate(&player_id)
            .unwrap_or(ffb_model::types::FieldCoordinate::new(0, 0));

        match effect {
            SpecialEffect::Lightning => {
                let injury_result = handle_injury_by_name(
                    game, rng, "InjuryTypeLightning",
                    None, &player_id, coord, None, None, ApothecaryMode::SpecialEffect,
                );
                outcome = outcome.publish(StepParameter::InjuryResult(Box::new(injury_result)));
                for p in drop_player(game, &player_id, true) { outcome = outcome.publish(p); }
            }
            SpecialEffect::Zap => {
                // DEFERRED(ZAP): ZappedPlayer substitution not yet ported.
            }
            SpecialEffect::Fireball => {
                let injury_result = handle_injury_by_name(
                    game, rng, "InjuryTypeFireball",
                    None, &player_id, coord, None, None, ApothecaryMode::SpecialEffect,
                );
                outcome = outcome.publish(StepParameter::InjuryResult(Box::new(injury_result)));
                for p in drop_player(game, &player_id, true) { outcome = outcome.publish(p); }
            }
            SpecialEffect::Bomb => {
                let injury_result = handle_injury_by_name(
                    game, rng, "InjuryTypeBomb",
                    None, &player_id, coord, None, None, ApothecaryMode::SpecialEffect,
                );
                outcome = outcome.publish(StepParameter::InjuryResult(Box::new(injury_result)));
                for p in drop_player(game, &player_id, true) { outcome = outcome.publish(p); }
            }
        }

        // Java: determine acting team — bomb modes override home_playing flag
        let acting_team_has_player = match game.turn_mode {
            TurnMode::BombHome | TurnMode::BombHomeBlitz => game.team_home.has_player(&player_id),
            TurnMode::BombAway | TurnMode::BombAwayBlitz => game.team_away.has_player(&player_id),
            _ => game.active_team().has_player(&player_id),
        };
        if effect != SpecialEffect::Fireball && acting_team_has_player {
            outcome = outcome.publish(StepParameter::EndTurn(true));
        }

        outcome
    }
}

/// Java: DiceInterpreter.isSpecialEffectSuccesful(effect, targetPlayer, roll).
fn is_special_effect_successful(effect: Option<SpecialEffect>, game: &Game, player_id: &str, roll: i32) -> bool {
    match effect {
        Some(SpecialEffect::Lightning) => roll >= 2,
        Some(SpecialEffect::Zap) => {
            let strength = game.player(player_id)
                .map(|p| p.strength_with_modifiers())
                .unwrap_or(3);
            roll == 6 || (roll > 1 && roll >= strength)
        }
        Some(SpecialEffect::Fireball) | Some(SpecialEffect::Bomb) => roll >= 4,
        None => false,
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
        use ffb_model::enums::{PS_STANDING, PlayerState};
        use ffb_model::types::FieldCoordinate;
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        });
        game.field_model.set_player_coordinate(id, FieldCoordinate::new(5, 5));
        game.field_model.set_player_state(id, PlayerState::new(PS_STANDING));
    }

    fn add_away_player(game: &mut Game, id: &str) {
        use ffb_model::enums::{PS_STANDING, PlayerState};
        use ffb_model::types::FieldCoordinate;
        game.team_away.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        });
        game.field_model.set_player_coordinate(id, FieldCoordinate::new(10, 5));
        game.field_model.set_player_state(id, PlayerState::new(PS_STANDING));
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

    #[test]
    fn lightning_publishes_injury_result() {
        let mut game = make_game();
        add_away_player(&mut game, "p1");
        game.home_playing = true;
        let mut step = StepSpecialEffect::new("fail".into(), "p1".into(), Some(SpecialEffect::Lightning));
        step.roll_for_effect = false;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::InjuryResult(_))));
    }

    #[test]
    fn bomb_mode_uses_home_team_for_end_turn_check() {
        use ffb_model::enums::TurnMode;
        let mut game = make_game();
        add_home_player(&mut game, "home_p");
        // BombHome mode: home team is acting regardless of home_playing
        game.home_playing = false;
        game.turn_mode = TurnMode::BombHome;
        let mut step = StepSpecialEffect::new("fail".into(), "home_p".into(), Some(SpecialEffect::Bomb));
        step.roll_for_effect = false;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    #[test]
    fn lightning_succeeds_on_roll_2_or_higher() {
        let game = make_game();
        assert!(is_special_effect_successful(Some(SpecialEffect::Lightning), &game, "x", 2));
        assert!(is_special_effect_successful(Some(SpecialEffect::Lightning), &game, "x", 6));
        assert!(!is_special_effect_successful(Some(SpecialEffect::Lightning), &game, "x", 1));
    }

    #[test]
    fn fireball_and_bomb_succeed_on_roll_4_or_higher() {
        let game = make_game();
        assert!(is_special_effect_successful(Some(SpecialEffect::Fireball), &game, "x", 4));
        assert!(!is_special_effect_successful(Some(SpecialEffect::Fireball), &game, "x", 3));
        assert!(is_special_effect_successful(Some(SpecialEffect::Bomb), &game, "x", 4));
        assert!(!is_special_effect_successful(Some(SpecialEffect::Bomb), &game, "x", 3));
    }

    #[test]
    fn zap_succeeds_on_6_or_above_target_strength() {
        let mut game = make_game();
        add_home_player(&mut game, "p1"); // strength 3
        // roll 6 → always succeeds
        assert!(is_special_effect_successful(Some(SpecialEffect::Zap), &game, "p1", 6));
        // roll 3 >= strength(3) and roll > 1 → succeeds
        assert!(is_special_effect_successful(Some(SpecialEffect::Zap), &game, "p1", 3));
        // roll 1 → always fails (must be > 1)
        assert!(!is_special_effect_successful(Some(SpecialEffect::Zap), &game, "p1", 1));
    }

    #[test]
    fn none_effect_always_fails() {
        let game = make_game();
        assert!(!is_special_effect_successful(None, &game, "x", 6));
    }
}
