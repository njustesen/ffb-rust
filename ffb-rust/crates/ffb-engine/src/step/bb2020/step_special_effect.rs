use ffb_model::enums::{ApothecaryMode, TurnMode};
use ffb_model::model::game::Game;
use ffb_model::model::special_effect::SpecialEffect;
use ffb_model::option::game_option_id::BOMBER_PLACED_PRONE_IGNORES_TURNOVER;
use ffb_model::option::util_game_option::is_option_enabled;
use ffb_model::report::report_special_effect_roll::ReportSpecialEffectRoll;
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
    /// Java: passState.getOriginalBombardier() — wired when bomb sequence initializes this step.
    pub original_bombardier: Option<String>,
}

impl StepSpecialEffect {
    pub fn new(goto_label_on_failure: String) -> Self {
        Self {
            goto_label_on_failure,
            player_id: None,
            roll_for_effect: false,
            special_effect_key: None,
            original_bombardier: None,
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
            StepParameter::OriginalBombardier(v) => { self.original_bombardier = v.clone(); true }
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

        let special_effect = self.special_effect_key.as_deref()
            .and_then(SpecialEffect::for_name);

        // Java: if fRollForEffect → roll and check success; else always successful
        let (successful, effect_roll, effect_success) = if self.roll_for_effect {
            let roll = rng.d6();
            let success = is_special_effect_successful(special_effect, game, &player_id, roll);
            (success, roll, success)
        } else {
            (true, 0, true)
        };
        // Java: getResult().addReport(new ReportSpecialEffectRoll(fSpecialEffect, player.getId(), roll, successful))
        if let Some(effect) = special_effect {
            game.report_list.add(ReportSpecialEffectRoll::new(
                effect,
                Some(player_id.clone()),
                effect_roll,
                effect_success,
            ));
        }

        if !successful {
            return StepOutcome::goto(&self.goto_label_on_failure);
        }

        let mut outcome = StepOutcome::next();

        match special_effect {
            Some(SpecialEffect::ZAP) => {
                // Java: ZappedPlayer.init(rosterPlayer, game) + team.addPlayer(zappedPlayer)
                // BB2020 zap stats: MA=5, ST=1, AG=2, PA=0, AV=5
                if let Some(player) = game.player(&player_id).cloned() {
                    let position = ffb_model::model::zapped_position::ZappedPosition::new_bb2020(
                        player.position_id.clone(), player.name.clone(),
                    );
                    game.add_zapped_player(ffb_model::model::zapped_player::ZappedPlayer::new(player, position));
                }
                if game.field_model.ball_coordinate == Some(player_coord) {
                    outcome = outcome.publish(StepParameter::CatchScatterThrowInMode(
                        CatchScatterThrowInMode::ScatterBall,
                    ));
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
                // Sync original_bombardier to game state for apply_to() downstream.
                game.original_bombardier = self.original_bombardier.clone();

                let bomb_from_home = matches!(game.turn_mode, TurnMode::BombHome | TurnMode::BombHomeBlitz);
                let bomb_from_away = matches!(game.turn_mode, TurnMode::BombAway | TurnMode::BombAwayBlitz);

                let player_is_home = game.team_home.player(&player_id).is_some();
                let player_hit_is_from_bomb_team =
                    (bomb_from_home && player_is_home) || (bomb_from_away && !player_is_home);

                let mut suppress_end_turn = !(player_hit_is_from_bomb_team
                    && game.field_model.ball_coordinate == Some(player_coord));
                // Java: if player == originalBombardier && !bomberTurnoverIgnored → suppressEndTurn=false
                if let Some(ref orig_id) = self.original_bombardier {
                    if &player_id == orig_id && !is_option_enabled(game, BOMBER_PLACED_PRONE_IGNORES_TURNOVER) {
                        suppress_end_turn = false;
                    }
                }

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

/// 1:1 translation of `DiceInterpreter.isSpecialEffectSuccessful`.
fn is_special_effect_successful(effect: Option<SpecialEffect>, game: &Game, player_id: &str, roll: i32) -> bool {
    match effect {
        Some(SpecialEffect::LIGHTNING) => roll >= 2,
        Some(SpecialEffect::ZAP) => {
            let strength = game.player(player_id)
                .map(|p| p.strength_with_modifiers())
                .unwrap_or(3);
            roll == 6 || (roll > 1 && roll >= strength)
        }
        Some(SpecialEffect::FIREBALL) | Some(SpecialEffect::BOMB) => roll >= 4,
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{Rules, PlayerState, PS_STANDING, PlayerType, PlayerGender};
    use ffb_model::model::player::Player;
    use ffb_model::report::report_id::ReportId;
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
            is_big_guy: false,
            ..Default::default()
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

    #[test]
    fn roll_for_effect_lightning_fail_goto_label() {
        use ffb_model::util::rng::GameRng;
        let pid = "p1";
        let mut step = StepSpecialEffect::new("FAIL".into());
        step.player_id = Some(pid.into());
        step.roll_for_effect = true;
        step.special_effect_key = Some("lightning".into());
        let mut game = make_game_with_player(pid, true);
        // Lightning requires roll >= 2; seed 0 produces roll=1 on first d6
        let mut rng = GameRng::new(0);
        let first_roll = rng.d6();
        let should_fail = first_roll < 2;
        let mut rng2 = GameRng::new(0);
        let out = step.start(&mut game, &mut rng2);
        if should_fail {
            assert_eq!(out.action, StepAction::GotoLabel,
                "lightning roll<2 should goto failure label");
        } else {
            assert_eq!(out.action, StepAction::NextStep);
        }
    }

    #[test]
    fn is_special_effect_successful_lightning_threshold() {
        let game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020);
        assert!(is_special_effect_successful(Some(SpecialEffect::LIGHTNING), &game, "x", 2));
        assert!(is_special_effect_successful(Some(SpecialEffect::LIGHTNING), &game, "x", 6));
        assert!(!is_special_effect_successful(Some(SpecialEffect::LIGHTNING), &game, "x", 1));
    }

    #[test]
    fn is_special_effect_successful_fireball_threshold() {
        let game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020);
        assert!(is_special_effect_successful(Some(SpecialEffect::FIREBALL), &game, "x", 4));
        assert!(!is_special_effect_successful(Some(SpecialEffect::FIREBALL), &game, "x", 3));
        assert!(is_special_effect_successful(Some(SpecialEffect::BOMB), &game, "x", 4));
        assert!(!is_special_effect_successful(Some(SpecialEffect::BOMB), &game, "x", 3));
    }

    #[test]
    fn is_special_effect_successful_zap_strength_based() {
        let mut game = make_game_with_player("p1", true); // str=3
        // roll==6 always succeeds
        assert!(is_special_effect_successful(Some(SpecialEffect::ZAP), &game, "p1", 6));
        // roll >= strength (3) succeeds
        assert!(is_special_effect_successful(Some(SpecialEffect::ZAP), &game, "p1", 3));
        // roll < strength fails
        assert!(!is_special_effect_successful(Some(SpecialEffect::ZAP), &game, "p1", 2));
        // roll==1 always fails (roll > 1 guard)
        assert!(!is_special_effect_successful(Some(SpecialEffect::ZAP), &game, "p1", 1));
    }

    #[test]
    fn set_parameter_original_bombardier_accepted() {
        let mut step = StepSpecialEffect::default();
        assert!(step.set_parameter(&StepParameter::OriginalBombardier(Some("bomb1".into()))));
        assert_eq!(step.original_bombardier.as_deref(), Some("bomb1"));
    }

    #[test]
    fn bomb_original_bombardier_same_as_player_suppresses_end_turn_false() {
        // When player is their own bomb (originalBombardier == player) and the option is disabled,
        // suppressEndTurn becomes false → EndTurn published for standing acting-team player
        let pid = "bomb1";
        let mut step = StepSpecialEffect::new("FAIL".into());
        step.player_id = Some(pid.into());
        step.roll_for_effect = false;
        step.special_effect_key = Some("bomb".into());
        step.original_bombardier = Some(pid.into());

        let mut game = make_game_with_player(pid, true); // home player, home playing
        game.turn_mode = TurnMode::BombHome;
        // BOMBER_PLACED_PRONE_IGNORES_TURNOVER defaults to false → !false = true → suppressEndTurn=false
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))),
            "original bombardier hits themselves → suppressEndTurn=false → EndTurn published");
    }

    #[test]
    fn no_roll_for_effect_adds_report_with_roll_zero_and_successful_true() {
        // Java: else { getResult().addReport(new ReportSpecialEffectRoll(fSpecialEffect, player.getId(), 0, true)) }
        let pid = "p1";
        let mut step = StepSpecialEffect::new("FAIL".into());
        step.player_id = Some(pid.into());
        step.roll_for_effect = false;
        step.special_effect_key = Some("lightning".into());

        let mut game = make_game_with_player(pid, false);
        step.start(&mut game, &mut GameRng::new(0));

        assert!(
            game.report_list.has_report(ReportId::SPELL_EFFECT_ROLL),
            "report_list should contain a ReportSpecialEffectRoll when roll_for_effect=false"
        );
    }

    #[test]
    fn roll_for_effect_adds_report_with_actual_roll() {
        // Java: getResult().addReport(new ReportSpecialEffectRoll(fSpecialEffect, player.getId(), roll, successful))
        let pid = "p1";
        let mut step = StepSpecialEffect::new("FAIL".into());
        step.player_id = Some(pid.into());
        step.roll_for_effect = true;
        step.special_effect_key = Some("fireball".into()); // always-success threshold=4; seed chosen to pass

        let mut game = make_game_with_player(pid, false);
        step.start(&mut game, &mut GameRng::new(42));

        assert!(
            game.report_list.has_report(ReportId::SPELL_EFFECT_ROLL),
            "report_list should contain a ReportSpecialEffectRoll when roll_for_effect=true"
        );
    }
}
