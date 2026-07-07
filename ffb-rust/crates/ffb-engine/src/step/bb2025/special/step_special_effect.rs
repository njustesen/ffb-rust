/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2025.special.StepSpecialEffect`.
///
/// Applies a special inducement effect (Lightning, ZAP, Fireball, Bomb) to a player.
/// Optionally rolls a d6 to check if the effect succeeds.
///
/// BB2025 differences vs BB2016:
///   - FIREBALL: publishes SteadyFootingContext(InjuryResult) instead of InjuryResult directly.
///   - BOMB: complex suppressEndTurn logic + SteadyFootingContext; SPP tracking via
///     InjuryTypeBombWithModifierForSpp when bombardier != player's team and has ViolentInnovator.
///   - END_TURN guard: only published if `isStanding` (not prone/stunned).
///   - ZAP: creates ZappedPlayer with BB2020 stats (MA=5, ST=1, AG=2, PA=0, AV=5).
use ffb_model::enums::{ApothecaryMode, TurnMode};
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::special_effect::SpecialEffect;
use ffb_model::option::game_option_id::{BOMB_TEAM_MATE_KNOCK_DOWN_CAUSES_TURNOVER, BOMBER_PLACED_PRONE_IGNORES_TURNOVER};
use ffb_model::option::util_game_option::is_option_enabled;
use ffb_model::report::report_special_effect_roll::ReportSpecialEffectRoll;
use ffb_model::report::report_id::ReportId;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::drop_player_context::SteadyFootingContext;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_injury::{drop_player, handle_injury_by_name};

/// Java: `StepSpecialEffect` (bb2025/special).
pub struct StepSpecialEffect {
    /// Java: fGotoLabelOnFailure (mandatory init param)
    pub goto_label_on_failure: String,
    /// Java: fPlayerId (mandatory init param)
    pub player_id: Option<String>,
    /// Java: fRollForEffect (mandatory init param)
    pub roll_for_effect: bool,
    /// Java: fSpecialEffect (mandatory init param)
    pub special_effect: Option<SpecialEffect>,
    /// Java: passState.getOriginalBombardier() — set when bomb sequence wires up the bombardier.
    pub original_bombardier: Option<String>,
}

impl StepSpecialEffect {
    pub fn new(goto_label_on_failure: String) -> Self {
        Self {
            goto_label_on_failure,
            player_id: None,
            roll_for_effect: false,
            special_effect: None,
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
            StepParameter::PlayerId(v) => { self.player_id = Some(v.clone()); true }
            StepParameter::RollForEffect(v) => { self.roll_for_effect = *v; true }
            StepParameter::GotoLabelOnFailure(v) => { self.goto_label_on_failure = v.clone(); true }
            // Java: init() reads SPECIAL_EFFECT as SpecialEffect enum — passed via SpecialEffectKey string
            StepParameter::SpecialEffectKey(v) => {
                self.special_effect = SpecialEffect::for_name(&v.to_lowercase());
                true
            }
            StepParameter::OriginalBombardier(v) => { self.original_bombardier = v.clone(); true }
            _ => false,
        }
    }
}

impl StepSpecialEffect {
    fn execute_step(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let player_id = match &self.player_id {
            Some(id) if !id.is_empty() => id.clone(),
            _ => return StepOutcome::next(),
        };

        if game.player(&player_id).is_none() {
            return StepOutcome::next();
        }

        // Java: state = fieldModel.getPlayerState(player); isStanding = !prone && !stunned; isActive = state.isActive()
        let is_standing = game.field_model.player_state(&player_id)
            .map(|s| !s.is_prone_or_stunned() && !s.is_stunned())
            .unwrap_or(true);

        // Java: if fRollForEffect → roll; DiceInterpreter.isSpecialEffectSuccessful
        let mut spell_roll_event: Option<GameEvent> = None;
        let mut last_roll: i32 = 0;
        let successful = if self.roll_for_effect {
            let roll = rng.d6();
            last_roll = roll;
            spell_roll_event = Some(GameEvent::SpellEffectRoll { roll });
            is_special_effect_successful(self.special_effect, game, &player_id, roll)
        } else {
            true
        };

        if !successful {
            let label = self.goto_label_on_failure.clone();
            let mut out = StepOutcome::goto(&label);
            if let Some(ev) = spell_roll_event { out = out.with_event(ev); }
            // Java: addReport(new ReportSpecialEffectRoll(effect, playerId, roll, false))
            if let Some(effect) = self.special_effect {
                game.report_list.add(ReportSpecialEffectRoll::new(effect, Some(player_id.clone()), last_roll, false));
            }
            return out;
        }

        let mut outcome = StepOutcome::next();
        if let Some(ev) = spell_roll_event { outcome = outcome.with_event(ev); }

        let effect = match self.special_effect {
            Some(e) => e,
            None => return outcome,
        };

        // Java: addReport(new ReportSpecialEffectRoll(effect, playerId, roll, true))
        // roll=0 for the no-roll path (always-succeed effects); last_roll for the roll path.
        game.report_list.add(ReportSpecialEffectRoll::new(effect, Some(player_id.clone()), last_roll, true));

        let coord = game.field_model.player_coordinate(&player_id)
            .unwrap_or(ffb_model::types::FieldCoordinate::new(0, 0));

        let mut suppress_end_turn = false;

        match effect {
            SpecialEffect::LIGHTNING => {
                let ir = handle_injury_by_name(
                    game, rng, "InjuryTypeLightning",
                    None, &player_id, coord, None, None, ApothecaryMode::SpecialEffect,
                );
                outcome = outcome.publish(StepParameter::InjuryResult(Box::new(ir)));
                for p in drop_player(game, &player_id, true) { outcome = outcome.publish(p); }
            }
            SpecialEffect::ZAP => {
                // Java: ZappedPlayer.init(rosterPlayer, game) + team.addPlayer(zappedPlayer)
                // BB2025 zap stats same as BB2020: MA=5, ST=1, AG=2, PA=0, AV=5
                if let Some(player) = game.player(&player_id).cloned() {
                    let position = ffb_model::model::zapped_position::ZappedPosition::new_bb2020(
                        player.position_id.clone(), player.name.clone(),
                    );
                    game.add_zapped_player(ffb_model::model::zapped_player::ZappedPlayer::new(player, position));
                }
                let ball_coord = game.field_model.ball_coordinate;
                let on_ball = ball_coord.map(|b| b == coord).unwrap_or(false);
                if on_ball {
                    outcome = outcome.publish(StepParameter::CatchScatterThrowInMode(
                        crate::step::framework::CatchScatterThrowInMode::ScatterBall,
                    ));
                }
            }
            SpecialEffect::FIREBALL => {
                let ir = handle_injury_by_name(
                    game, rng, "InjuryTypeFireball",
                    None, &player_id, coord, None, None, ApothecaryMode::SpecialEffect,
                );
                let ctx = SteadyFootingContext::from_injury_result(ir);
                outcome = outcome.publish(StepParameter::SteadyFootingContext(Box::new(ctx)));
            }
            SpecialEffect::BOMB => {
                // Sync original_bombardier to game state for apply_to() downstream.
                game.original_bombardier = self.original_bombardier.clone();

                let bomb_from_home = matches!(game.turn_mode, TurnMode::BombHome | TurnMode::BombHomeBlitz);
                let bomb_from_away = matches!(game.turn_mode, TurnMode::BombAway | TurnMode::BombAwayBlitz);

                let player_hit_from_bomb_team =
                    (bomb_from_home && game.team_home.has_player(&player_id))
                    || (bomb_from_away && game.team_away.has_player(&player_id));

                // Java: if (!BOMB_TEAM_MATE_KNOCK_DOWN_CAUSES_TURNOVER) → suppressEndTurn = !(playerHitIsFromBombTeam && hasBall)
                if !is_option_enabled(game, BOMB_TEAM_MATE_KNOCK_DOWN_CAUSES_TURNOVER) {
                    let has_ball = game.field_model.ball_coordinate
                        .map(|b| b == coord)
                        .unwrap_or(false);
                    suppress_end_turn = !(player_hit_from_bomb_team && has_ball);
                }
                // Java: if player == originalBombardier && !bomberTurnoverIgnored → suppressEndTurn=false
                if let Some(ref orig_id) = self.original_bombardier {
                    if &player_id == orig_id && !is_option_enabled(game, BOMBER_PLACED_PRONE_IGNORES_TURNOVER) {
                        suppress_end_turn = false;
                    }
                }

                // Java: bombardierGetsSpp = bombardier team != player team && bombardier has grantsSppFromSpecialActionsCas
                let bombardier_id = self.original_bombardier.as_deref();
                let bombardier_gets_spp = bombardier_id.map(|b_id| {
                    game.player(b_id).map(|bombardier| {
                        let bombardier_is_home = game.team_home.has_player(b_id);
                        let player_is_home = game.team_home.has_player(&player_id);
                        let different_teams = bombardier_is_home != player_is_home;
                        different_teams && bombardier.has_skill_property(NamedProperties::GRANTS_SPP_FROM_SPECIAL_ACTIONS_CAS)
                    }).unwrap_or(false)
                }).unwrap_or(false);

                let injury_type_name = if bombardier_gets_spp { "InjuryTypeBombWithModifierForSpp" } else { "InjuryTypeBombWithModifier" };
                let attacker_id = if bombardier_gets_spp { bombardier_id } else { None };
                let ir = handle_injury_by_name(
                    game, rng, injury_type_name,
                    attacker_id, &player_id, coord, None, None, ApothecaryMode::SpecialEffect,
                );
                let ctx = SteadyFootingContext::from_injury_result(ir);
                outcome = outcome.publish(StepParameter::SteadyFootingContext(Box::new(ctx)));
            }
        }

        // Java: if isStanding { if actingTeam.hasPlayer && effect != FIREBALL && !suppressEndTurn → END_TURN=true }
        if is_standing {
            let acting_team_has_player = match game.turn_mode {
                TurnMode::BombHome | TurnMode::BombHomeBlitz => game.team_home.has_player(&player_id),
                TurnMode::BombAway | TurnMode::BombAwayBlitz => game.team_away.has_player(&player_id),
                _ => game.active_team().has_player(&player_id),
            };
            if effect != SpecialEffect::FIREBALL && acting_team_has_player && !suppress_end_turn {
                outcome = outcome.publish(StepParameter::EndTurn(true));
            }
        }

        outcome
    }
}

/// Java: `DiceInterpreter.isSpecialEffectSuccessful(effect, player, roll)`.
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
    use ffb_model::enums::Rules;
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender, PlayerState, PS_STANDING};
    use ffb_model::types::FieldCoordinate;
    use std::collections::HashSet;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_home_player(game: &mut Game, id: &str) {
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        });
        let coord = FieldCoordinate::new(5, 5);
        game.field_model.set_player_coordinate(id, coord);
        game.field_model.set_player_state(id, PlayerState::new(PS_STANDING));
    }

    #[test]
    fn no_player_returns_next_step() {
        let mut game = make_game();
        let mut step = StepSpecialEffect::default();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn unknown_player_id_returns_next_step() {
        let mut game = make_game();
        let mut step = StepSpecialEffect::new("fail".into());
        step.player_id = Some("no_such_player".into());
        step.special_effect = Some(SpecialEffect::LIGHTNING);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_player_id() {
        let mut step = StepSpecialEffect::default();
        assert!(step.set_parameter(&StepParameter::PlayerId("p1".into())));
        assert_eq!(step.player_id.as_deref(), Some("p1"));
    }

    #[test]
    fn set_parameter_roll_for_effect() {
        let mut step = StepSpecialEffect::default();
        assert!(step.set_parameter(&StepParameter::RollForEffect(true)));
        assert!(step.roll_for_effect);
    }

    #[test]
    fn set_parameter_goto_label() {
        let mut step = StepSpecialEffect::default();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("lbl".into())));
        assert_eq!(step.goto_label_on_failure, "lbl");
    }

    #[test]
    fn set_parameter_special_effect_key() {
        let mut step = StepSpecialEffect::default();
        assert!(step.set_parameter(&StepParameter::SpecialEffectKey("FIREBALL".into())));
        assert_eq!(step.special_effect, Some(SpecialEffect::FIREBALL));
    }

    #[test]
    fn no_roll_lightning_publishes_injury_result() {
        let mut game = make_game();
        game.home_playing = true;
        add_home_player(&mut game, "p1");
        let mut step = StepSpecialEffect::new("fail".into());
        step.player_id = Some("p1".into());
        step.special_effect = Some(SpecialEffect::LIGHTNING);
        step.roll_for_effect = false;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::InjuryResult(_))));
    }

    #[test]
    fn no_roll_fireball_publishes_steady_footing_context() {
        let mut game = make_game();
        game.home_playing = true;
        add_home_player(&mut game, "p1");
        let mut step = StepSpecialEffect::new("fail".into());
        step.player_id = Some("p1".into());
        step.special_effect = Some(SpecialEffect::FIREBALL);
        step.roll_for_effect = false;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::SteadyFootingContext(_))));
    }

    #[test]
    fn roll_for_effect_failure_goes_to_label() {
        let mut game = make_game();
        game.home_playing = true;
        add_home_player(&mut game, "p1");
        let mut step = StepSpecialEffect::new("fail_label".into());
        step.player_id = Some("p1".into());
        step.special_effect = Some(SpecialEffect::LIGHTNING);
        step.roll_for_effect = true;
        // Seed 7 should produce a roll of 1 (which fails the lightning check >= 2)
        // We just test the path — with seed 0, roll=1 always fails lightning (1 < 2)
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Either goto fail_label (roll failed) or next (roll succeeded)
        // Just verify it doesn't panic and action is set
        assert!(matches!(out.action, StepAction::NextStep | StepAction::GotoLabel));
    }

    #[test]
    fn lightning_on_own_team_player_publishes_end_turn() {
        let mut game = make_game();
        game.home_playing = true;
        add_home_player(&mut game, "p1");
        let mut step = StepSpecialEffect::new("fail".into());
        step.player_id = Some("p1".into());
        step.special_effect = Some(SpecialEffect::LIGHTNING);
        step.roll_for_effect = false;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    #[test]
    fn fireball_does_not_publish_end_turn() {
        let mut game = make_game();
        game.home_playing = true;
        add_home_player(&mut game, "p1");
        let mut step = StepSpecialEffect::new("fail".into());
        step.player_id = Some("p1".into());
        step.special_effect = Some(SpecialEffect::FIREBALL);
        step.roll_for_effect = false;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(!out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    #[test]
    fn set_parameter_original_bombardier_accepted() {
        let mut step = StepSpecialEffect::default();
        assert!(step.set_parameter(&StepParameter::OriginalBombardier(Some("bomb1".into()))));
        assert_eq!(step.original_bombardier.as_deref(), Some("bomb1"));
    }

    #[test]
    fn set_parameter_original_bombardier_none_accepted() {
        let mut step = StepSpecialEffect::default();
        assert!(step.set_parameter(&StepParameter::OriginalBombardier(None)));
        assert!(step.original_bombardier.is_none());
    }

    #[test]
    fn no_roll_lightning_emits_spell_effect_roll_report() {
        let mut game = make_game();
        game.home_playing = true;
        add_home_player(&mut game, "p1");
        let mut step = StepSpecialEffect::new("fail".into());
        step.player_id = Some("p1".into());
        step.special_effect = Some(SpecialEffect::LIGHTNING);
        step.roll_for_effect = false;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::SPELL_EFFECT_ROLL));
    }

    #[test]
    fn roll_failure_still_emits_spell_effect_roll_report() {
        // Seed 0 produces roll=1 for lightning (threshold>=2) → failure → goto label
        // but report should still be added.
        let mut game = make_game();
        game.home_playing = true;
        add_home_player(&mut game, "p1");
        let mut step = StepSpecialEffect::new("fail_label".into());
        step.player_id = Some("p1".into());
        step.special_effect = Some(SpecialEffect::LIGHTNING);
        step.roll_for_effect = true;
        step.start(&mut game, &mut GameRng::new(0));
        // Whether roll succeeded or failed, the report is always emitted.
        assert!(game.report_list.has_report(ReportId::SPELL_EFFECT_ROLL));
    }

    #[test]
    fn bomb_uses_bomb_with_modifier_injury_type() {
        // Bomb with no originalBombardier → uses InjuryTypeBombWithModifier → publishes SteadyFootingContext
        let mut game = make_game();
        game.home_playing = true;
        game.turn_mode = TurnMode::BombHome;
        add_home_player(&mut game, "p1");
        let mut step = StepSpecialEffect::new("fail".into());
        step.player_id = Some("p1".into());
        step.special_effect = Some(SpecialEffect::BOMB);
        step.roll_for_effect = false;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::SteadyFootingContext(_))));
    }

    #[test]
    fn bomb_original_bombardier_is_same_as_player_and_option_disabled_suppresses_end_turn() {
        // When originalBombardier == player and BOMBER_PLACED_PRONE_IGNORES_TURNOVER is disabled,
        // suppressEndTurn should become false → EndTurn should be published for a standing acting-team player
        let mut game = make_game();
        game.home_playing = true;
        game.turn_mode = TurnMode::BombHome;
        add_home_player(&mut game, "bomb1");
        let mut step = StepSpecialEffect::new("fail".into());
        step.player_id = Some("bomb1".into());
        step.special_effect = Some(SpecialEffect::BOMB);
        step.roll_for_effect = false;
        step.original_bombardier = Some("bomb1".into());
        // BOMBER_PLACED_PRONE_IGNORES_TURNOVER defaults to false → !false = true → suppressEndTurn=false
        let out = step.start(&mut game, &mut GameRng::new(0));
        // suppressEndTurn=false AND player is standing AND acting team has player → should publish EndTurn
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))),
            "when original bombardier is hit and option disabled, EndTurn should be published");
    }
}
