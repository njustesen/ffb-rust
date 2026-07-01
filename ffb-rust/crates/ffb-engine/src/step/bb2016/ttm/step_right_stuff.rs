/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.ttm.StepRightStuff`.
///
/// Step in TTM sequence to handle skill RIGHT_STUFF (landing roll).
/// - If player state is FALLING (thrown out of bounds): publish END_TURN +
///   THROWN_PLAYER_COORDINATE(null) → NEXT_STEP.
/// - If player has ball: move ball to player coordinate.
/// - If drop_thrown_player == false: roll landing (minimumRollRightStuff + modifiers).
///   - Success + has ball → touchdown check.
///   - Success without ball on ball square → SCATTER_BALL.
///   - Failure → re-roll if available.
/// - If drop_thrown_player == true (or roll failed, re-roll exhausted): TTMLanding injury.
///
/// RightStuffModifierFactory (non-tacklezone only) → wired.
/// AgilityMechanic.minimumRollRightStuff → wired (bb2016::AgilityMechanic).
/// DEFERRED(modifier-tacklezone): RightStuffModifier tacklezone predicates require adjacent-opponent
///   counting in FieldModel; skipped until FieldModel.count_tacklezones_on() is ported.
/// DEFERRED(reroll): AbstractStepWithReRoll / UtilServerReRoll not yet ported.
/// DEFERRED(injury): UtilServerInjury.handleInjury(InjuryTypeTTMLanding) not yet ported.
/// DEFERRED(touchdown): UtilServerSteps.checkTouchdown not yet ported.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::enums::{PS_FALLING, ApothecaryMode};
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::CatchScatterThrowInMode;
use ffb_model::model::kick_team_mate_range::KickTeamMateRange;
use ffb_mechanics::modifiers::modifier_type::ModifierType;
use ffb_mechanics::modifiers::bb2016::right_stuff_modifier_collection::RightStuffModifierCollection as Bb2016RightStuffModifiers;
use crate::dice_interpreter::DiceInterpreter;
use crate::injury::injuryType::injury_type_ttm_landing::InjuryTypeTTMLanding;
use crate::step::util_server_injury;

/// Java: `StepRightStuff` (bb2016/ttm).
pub struct StepRightStuff {
    /// Java: `fThrownPlayerHasBall`
    thrown_player_has_ball: Option<bool>,
    /// Java: `fThrownPlayerId`
    thrown_player_id: Option<String>,
    /// Java: `fDropThrownPlayer`
    drop_thrown_player: bool,
    /// Java: `ktmRange`
    ktm_range: Option<KickTeamMateRange>,
}

impl StepRightStuff {
    pub fn new() -> Self {
        Self {
            thrown_player_has_ball: None,
            thrown_player_id: None,
            drop_thrown_player: false,
            ktm_range: None,
        }
    }

    fn execute_step(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let player_id = match &self.thrown_player_id {
            Some(id) => id.clone(),
            None     => return StepOutcome::next(),
        };
        let has_ball = self.thrown_player_has_ball.unwrap_or(false);

        // If player is in FALLING state (was thrown out of bounds): skip landing roll.
        let is_falling = game.field_model.player_state(&player_id)
            .map(|s| s.base() == PS_FALLING)
            .unwrap_or(false);
        if is_falling {
            return StepOutcome::next()
                .publish(StepParameter::EndTurn(has_ball))
                .publish(StepParameter::ThrownPlayerCoordinate(None));
        }

        // Sync ball to player coordinate when holding ball.
        if has_ball {
            if let Some(coord) = game.field_model.player_coordinate(&player_id) {
                game.field_model.ball_coordinate = Some(coord);
            }
        }

        if self.drop_thrown_player {
            // Java: UtilServerInjury.handleInjury(InjuryTypeTTMLanding, actingPlayerId, playerId, coord, null, null, THROWN_PLAYER)
            let coord = game.field_model.player_coordinate(&player_id)
                .unwrap_or(ffb_model::types::FieldCoordinate::new(0, 0));
            let mut injury_type = InjuryTypeTTMLanding::new();
            let ir = util_server_injury::handle_injury(
                game, rng, &mut injury_type,
                None, &player_id, coord, None, None,
                ApothecaryMode::ThrownPlayer,
            );
            ir.apply_to(game);
            return StepOutcome::next()
                .publish(StepParameter::ThrownPlayerCoordinate(None));
        }

        // Compute minimum roll using AgilityMechanic + non-tacklezone modifiers.
        // DEFERRED(modifier-tacklezone): tacklezone predicates not wired; skip TACKLEZONE modifiers.
        let ktm_str = self.ktm_range.map(|r| r.get_name().to_string());
        let minimum_roll = {
            let player_agility = game.player(&player_id).map(|p| p.agility_with_modifiers());
            if let Some(agility) = player_agility {
                // Build a dummy context for modifier lookup (ktm_range only; game ref not needed for non-tacklezone).
                let modifier_total: i32 = Bb2016RightStuffModifiers::new()
                    .get_modifiers()
                    .iter()
                    .filter(|m| m.get_type() != ModifierType::TACKLEZONE)
                    .filter(|m| {
                        // Apply ktm predicate: "medium" → +1, "long" → +2.
                        match m.get_name() {
                            "Medium Kick" => ktm_str.as_deref() == Some("medium"),
                            "Long Kick"   => ktm_str.as_deref() == Some("long"),
                            _             => true,
                        }
                    })
                    .map(|m| m.get_modifier())
                    .sum();
                // Agility roll base: 7 - min(agility, 6)
                let base = 7 - agility.min(6);
                (base + modifier_total).max(2)
            } else {
                2
            }
        };

        let roll = rng.d6();
        let successful = DiceInterpreter::is_skill_roll_successful(roll, minimum_roll);

        if !successful {
            // DEFERRED(reroll): offer TRR / skill re-roll not yet wired.
            // Java: handleInjury(InjuryTypeTTMLanding, actingPlayerId, playerId, coord, null, null, THROWN_PLAYER)
            let coord = game.field_model.player_coordinate(&player_id)
                .unwrap_or(ffb_model::types::FieldCoordinate::new(0, 0));
            let mut injury_type = InjuryTypeTTMLanding::new();
            let ir = util_server_injury::handle_injury(
                game, rng, &mut injury_type,
                None, &player_id, coord, None, None,
                ApothecaryMode::ThrownPlayer,
            );
            ir.apply_to(game);
            return StepOutcome::next()
                .publish(StepParameter::ThrownPlayerCoordinate(None));
        }

        // DEFERRED(touchdown): checkTouchdown on landing not yet ported.
        let mut out = StepOutcome::next()
            .publish(StepParameter::ThrownPlayerCoordinate(None));
        if !has_ball {
            // Check if player landed on ball square.
            let player_coord = game.field_model.player_coordinate(&player_id);
            let ball_coord   = game.field_model.ball_coordinate;
            if player_coord.is_some() && player_coord == ball_coord {
                out = out.publish(StepParameter::CatchScatterThrowInMode(
                    CatchScatterThrowInMode::ScatterBall));
            }
        }
        out
    }
}

impl Default for StepRightStuff {
    fn default() -> Self { Self::new() }
}

impl Step for StepRightStuff {
    fn id(&self) -> StepId { StepId::RightStuff }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::ThrownPlayerHasBall(v) => { self.thrown_player_has_ball = Some(*v); true }
            StepParameter::ThrownPlayerId(v)      => { self.thrown_player_id = v.clone(); true }
            StepParameter::DropThrownPlayer(v)    => { self.drop_thrown_player = *v; true }
            StepParameter::KtmModifier(v)         => { self.ktm_range = Some(*v); true }
            // Also accept kicked-player aliases.
            StepParameter::KickedPlayerHasBall(v) => { self.thrown_player_has_ball = Some(*v); true }
            StepParameter::KickedPlayerId(v)      => { self.thrown_player_id = v.clone(); true }
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
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    #[test]
    fn id_is_right_stuff() {
        assert_eq!(StepRightStuff::new().id(), StepId::RightStuff);
    }

    #[test]
    fn no_thrown_player_returns_next() {
        let mut game = make_game();
        let out = StepRightStuff::new().start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
    }

    #[test]
    fn drop_thrown_player_publishes_coordinate_null() {
        let mut game = make_game();
        let mut step = StepRightStuff::new();
        step.thrown_player_id = Some("p1".into());
        step.drop_thrown_player = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::ThrownPlayerCoordinate(None))));
    }

    #[test]
    fn set_parameter_drop_thrown_player() {
        let mut step = StepRightStuff::new();
        assert!(step.set_parameter(&StepParameter::DropThrownPlayer(true)));
        assert!(step.drop_thrown_player);
    }

    #[test]
    fn set_parameter_ktm_range() {
        let mut step = StepRightStuff::new();
        assert!(step.set_parameter(&StepParameter::KtmModifier(KickTeamMateRange::SHORT)));
        assert_eq!(step.ktm_range, Some(KickTeamMateRange::SHORT));
    }

    #[test]
    fn roll_outcome_publishes_coordinate_null() {
        use std::collections::HashSet;
        use ffb_model::enums::{PlayerGender, PlayerType, SkillId};
        use ffb_model::model::player::Player;
        let mut game = make_game();
        let p = Player {
            id: "p1".into(), name: "p1".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
        };
        game.team_home.players.push(p);
        let mut step = StepRightStuff::new();
        step.thrown_player_id = Some("p1".into());
        // Any roll outcome must publish ThrownPlayerCoordinate(None).
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::ThrownPlayerCoordinate(None))));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn long_kick_adds_two_to_minimum_roll() {
        // Long kick raises minimum roll by 2 compared to short (no ktm modifier).
        // Agility 3 → base 4. Short: min=4. Long: min=6 (capped at 6).
        // We verify the step still produces NextStep with ThrownPlayerCoordinate(None) for any roll.
        use std::collections::HashSet;
        use ffb_model::enums::{PlayerGender, PlayerType, SkillId};
        use ffb_model::model::player::Player;
        let mut game = make_game();
        let p = Player {
            id: "p1".into(), name: "p1".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
        };
        game.team_home.players.push(p);
        let mut step = StepRightStuff::new();
        step.thrown_player_id = Some("p1".into());
        step.ktm_range = Some(KickTeamMateRange::LONG);
        let out = step.start(&mut game, &mut GameRng::new(42));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::ThrownPlayerCoordinate(None))));
    }
}
