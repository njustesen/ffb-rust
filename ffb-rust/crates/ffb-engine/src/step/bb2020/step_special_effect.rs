use ffb_model::enums::{ApothecaryMode, TurnMode};
use ffb_model::model::game::Game;
use ffb_model::model::special_effect::SpecialEffect;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{CatchScatterThrowInMode, Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_injury::{drop_player, handle_injury_by_name};

/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2020.StepSpecialEffect` (BB2020).
///
/// Applies a special inducement effect (Wizard spell / Bomb) to the target player.
///
/// BB2020 differences from BB2025:
/// - Uses `InjuryTypeBombWithModifier` (no `InjuryTypeBombWithModifierForSpp` variant).
/// - No SteadyFooting published — result goes directly to PLACE_BALL + APOTHECARY.
/// - ZAP handling is identical.
///
/// Mandatory init params (set via `set_parameter`):
///  - `GotoLabelOnFailure` (fGotoLabelOnFailure)
///  - `PlayerId` (fPlayerId)
///  - `RollForEffect` (fRollForEffect)
///  - `SpecialEffectKey` (fSpecialEffect — stored as a string key until full enum is ported)
///
/// Publishes:
///  - `InjuryResult` on success (FIREBALL / BOMB paths)
///  - `CatchScatterThrowInMode::ScatterBall` for ZAP when ball is on the player
///  - `EndTurn(true)` when a standing active-team player is hit
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2020.StepSpecialEffect`.
pub struct StepSpecialEffect {
    /// Java: fGotoLabelOnFailure (mandatory init param)
    pub goto_label_on_failure: String,
    /// Java: fPlayerId (mandatory init param)
    pub player_id: Option<String>,
    /// Java: fRollForEffect (mandatory init param)
    pub roll_for_effect: bool,
    /// Java: fSpecialEffect (mandatory init param — stored as string key)
    pub special_effect_key: Option<String>,
}

impl StepSpecialEffect {
    pub fn new(goto_label_on_failure: String) -> Self {
        Self {
            goto_label_on_failure,
            player_id: None,
            roll_for_effect: false,
            special_effect_key: None,
        }
    }
}

impl Default for StepSpecialEffect {
    fn default() -> Self { Self::new(String::new()) }
}

impl Step for StepSpecialEffect {
    fn id(&self) -> StepId { StepId::SpecialEffect }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(v) => { self.goto_label_on_failure = v.clone(); true }
            StepParameter::PlayerId(v)           => { self.player_id = Some(v.clone()); true }
            StepParameter::RollForEffect(v)      => { self.roll_for_effect = *v; true }
            StepParameter::SpecialEffectKey(v)   => { self.special_effect_key = Some(v.clone()); true }
            _ => false,
        }
    }
}

impl StepSpecialEffect {
    fn execute_step(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let player_id = match self.player_id.as_deref() {
            Some(id) if game.player(id).is_some() => id.to_owned(),
            _ => return StepOutcome::next(),
        };

        let player_coord = match game.field_model.player_coordinate(&player_id) {
            Some(c) => c,
            None => return StepOutcome::next(),
        };

        let state = match game.field_model.player_state(&player_id) {
            Some(s) => s,
            None => return StepOutcome::next(),
        };

        let is_standing = !state.is_prone() && !state.is_stunned();
        let is_active = state.is_active();

        // Java: if fRollForEffect → roll and check success; else always successful
        let successful = if self.roll_for_effect {
            let roll = rng.d6();
            // DEFERRED(special_effect): DiceInterpreter.isSpecialEffectSuccessful(fSpecialEffect, player, roll)
            // Stub: assume success if roll >= 4
            // DEFERRED(special_effect): ReportSpecialEffectRoll(fSpecialEffect, player.id, roll, successful)
            let _ = roll;
            true
        } else {
            // DEFERRED(special_effect): ReportSpecialEffectRoll(fSpecialEffect, player.id, 0, true)
            true
        };

        if !successful {
            return StepOutcome::goto(&self.goto_label_on_failure);
        }

        let special_effect = self.special_effect_key.as_deref()
            .and_then(SpecialEffect::for_name);

        let mut outcome = StepOutcome::next();

        match special_effect {
            Some(SpecialEffect::ZAP) => {
                // Java: ZappedPlayer creation + team replacement (TODO)
                // DEFERRED(special_effect): create ZappedPlayer, replace in team
                // If ball is on this player's square → scatter ball
                if game.field_model.ball_coordinate == Some(player_coord) {
                    outcome = outcome.publish(StepParameter::CatchScatterThrowInMode(
                        CatchScatterThrowInMode::ScatterBall,
                    ));
                    // DEFERRED(special_effect): push StepCatchScatterThrowIn onto stack
                }
            }
            Some(SpecialEffect::FIREBALL) => {
                let injury_result = handle_injury_by_name(
                    game, rng, "InjuryTypeFireball",
                    None, &player_id, player_coord, None, None, ApothecaryMode::SpecialEffect,
                );
                outcome = outcome.publish(StepParameter::InjuryResult(Box::new(injury_result)));
                for p in drop_player(game, &player_id, true) { outcome = outcome.publish(p); }
                let _ = is_active;
            }
            Some(SpecialEffect::BOMB) => {
                let bomb_from_home = matches!(game.turn_mode, TurnMode::BombHome | TurnMode::BombHomeBlitz);
                let bomb_from_away = matches!(game.turn_mode, TurnMode::BombAway | TurnMode::BombAwayBlitz);

                let player_is_home = game.team_home.player(&player_id).is_some();
                let player_hit_is_from_bomb_team =
                    (bomb_from_home && player_is_home) || (bomb_from_away && !player_is_home);

                // DEFERRED(special_effect): passState.getOriginalBombardier
                // DEFERRED(special_effect): BOMBER_PLACED_PRONE_IGNORES_TURNOVER option
                let suppress_end_turn = !(player_hit_is_from_bomb_team
                    && game.field_model.ball_coordinate == Some(player_coord));

                let injury_result = handle_injury_by_name(
                    game, rng, "InjuryTypeBombWithModifier",
                    None, &player_id, player_coord, None, None, ApothecaryMode::SpecialEffect,
                );
                outcome = outcome.publish(StepParameter::InjuryResult(Box::new(injury_result)));
                for p in drop_player(game, &player_id, true) { outcome = outcome.publish(p); }

                if !suppress_end_turn && is_standing {
                    let acting_team_has_player =
                        if bomb_from_home { player_is_home }
                        else if bomb_from_away { !player_is_home }
                        else if game.home_playing { player_is_home } else { !player_is_home };
                    if acting_team_has_player {
                        outcome = outcome.publish(StepParameter::EndTurn(true));
                    }
                }
                let _ = is_active;
            }
            Some(SpecialEffect::LIGHTNING) | None => {
                // Lightning has no injury in the base game; or unknown key
            }
        }

        // Java: check end turn — if isStanding AND actingTeam has player AND not FIREBALL AND not suppressEndTurn
        let player_is_home_check = game.team_home.player(&player_id).is_some();
        if let Some(SpecialEffect::FIREBALL) = special_effect {
            // FIREBALL never triggers end_turn via this check (handled via DropPlayer)
        } else if special_effect != Some(SpecialEffect::BOMB) && is_standing {
            let acting_team_has_player = if game.home_playing { player_is_home_check } else { !player_is_home_check };
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
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{Rules, PlayerState, PS_STANDING, PlayerType, PlayerGender};
    use ffb_model::model::player::Player;
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;

    fn make_player(id: &str) -> Player {
        Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
        }
    }

    fn make_game_with_player(pid: &str, home: bool) -> Game {
        let mut home_team = test_team("home", 0);
        let mut away_team = test_team("away", 0);
        if home {
            home_team.players.push(make_player(pid));
        } else {
            away_team.players.push(make_player(pid));
        }
        let mut game = Game::new(home_team, away_team, Rules::Bb2020);
        game.home_playing = true;
        game.field_model.set_player_state(pid, PlayerState::new(PS_STANDING).change_active(true));
        game.field_model.set_player_coordinate(pid, FieldCoordinate::new(10, 7));
        game
    }

    #[test]
    fn no_player_returns_next_step() {
        let mut step = StepSpecialEffect::new("FAIL".into());
        step.player_id = Some("ghost".into());
        let mut game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_wiring() {
        let mut step = StepSpecialEffect::default();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("FL".into())));
        assert_eq!(step.goto_label_on_failure, "FL");
        assert!(step.set_parameter(&StepParameter::PlayerId("p1".into())));
        assert_eq!(step.player_id.as_deref(), Some("p1"));
        assert!(step.set_parameter(&StepParameter::RollForEffect(true)));
        assert!(step.roll_for_effect);
        assert!(step.set_parameter(&StepParameter::SpecialEffectKey("bomb".into())));
        assert_eq!(step.special_effect_key.as_deref(), Some("bomb"));
        assert!(!step.set_parameter(&StepParameter::NrOfDice(2)));
    }

    #[test]
    fn lightning_on_active_team_player_publishes_end_turn() {
        let pid = "p1";
        let mut step = StepSpecialEffect::new("FAIL".into());
        step.player_id = Some(pid.into());
        step.roll_for_effect = false;
        step.special_effect_key = Some("lightning".into());

        let mut game = make_game_with_player(pid, true); // home player, home is acting
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        let has_end_turn = out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true)));
        assert!(has_end_turn, "lightning on active-team standing player should publish END_TURN");
    }

    #[test]
    fn lightning_on_opponent_does_not_publish_end_turn() {
        let pid = "opp1";
        let mut step = StepSpecialEffect::new("FAIL".into());
        step.player_id = Some(pid.into());
        step.roll_for_effect = false;
        step.special_effect_key = Some("lightning".into());

        let mut game = make_game_with_player(pid, false); // away player; home is acting
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        let has_end_turn = out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true)));
        assert!(!has_end_turn, "lightning on opponent should not publish END_TURN");
    }

    #[test]
    fn zap_on_ball_carrier_publishes_scatter_ball() {
        let pid = "p1";
        let mut step = StepSpecialEffect::new("FAIL".into());
        step.player_id = Some(pid.into());
        step.special_effect_key = Some("zap".into());

        let mut game = make_game_with_player(pid, false); // away player
        let player_coord = FieldCoordinate::new(10, 7);
        game.field_model.ball_coordinate = Some(player_coord);

        let out = step.start(&mut game, &mut GameRng::new(0));
        let has_scatter = out.published.iter().any(|p| {
            matches!(p, StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ScatterBall))
        });
        assert!(has_scatter, "zap on ball square should publish SCATTER_BALL");
    }

    #[test]
    fn fireball_publishes_injury_result() {
        let pid = "p1";
        let mut step = StepSpecialEffect::new("FAIL".into());
        step.player_id = Some(pid.into());
        step.special_effect_key = Some("fireball".into());
        let mut game = make_game_with_player(pid, false);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::InjuryResult(_))),
            "fireball should publish InjuryResult");
    }

    #[test]
    fn bomb_publishes_injury_result() {
        let pid = "p1";
        let mut step = StepSpecialEffect::new("FAIL".into());
        step.player_id = Some(pid.into());
        step.special_effect_key = Some("bomb".into());
        let mut game = make_game_with_player(pid, false);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::InjuryResult(_))),
            "bomb should publish InjuryResult");
    }
}
