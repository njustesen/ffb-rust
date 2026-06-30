use ffb_model::enums::{ApothecaryMode, PlayerState, PS_FALLING, PS_HIT_ON_GROUND};
use ffb_model::model::game::Game;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::drop_player_context::{DropPlayerContext, SteadyFootingContext};
use crate::injury::InjuryResult;
use crate::step::framework::{Step, StepOutcome, StepParameter};
use crate::step::framework::StepId;
use crate::step::util_server_injury::handle_injury_by_name;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.shared.StepDropFallingPlayers.
///
/// Drops players in FALLING state after a block, performing an injury roll for each:
///
/// Defender path:
///   - If `HIT_ON_GROUND` → upgrade to `FALLING`.
///   - If `FALLING && isRooted` → clear rooted.
///   - If `FALLING` → `handleInjury` → publish `STEADY_FOOTING_CONTEXT(DropPlayerContext)`.
///   - If saboteur triggered defender → publish `DROP_PLAYER_CONTEXT` directly (bypass Steady Footing).
///   - If defender is own team and not already prone → publish `END_TURN(true)`.
///
/// Attacker path:
///   - If `FALLING && isRooted` → clear rooted.
///   - If `FALLING` → publish `END_TURN(true)` + `handleInjury` →
///     publish `STEADY_FOOTING_CONTEXT(InjuryResult)` (InjuryResult variant, not DropPlayer variant).
///   - If `fell_from_rush` → injury type `InjuryTypeDropGFI`; else `InjuryTypeBlock`.
///   - If saboteur triggered attacker → publish `DROP_PLAYER_CONTEXT` directly.
///
/// Saboteur paths: not yet ported (always `false`).
/// `DeferredCommands` in attacker path: not yet ported (always empty).
pub struct StepDropFallingPlayers {
    /// Java: state.oldDefenderState
    pub old_defender_state: Option<PlayerState>,
    /// Java: state.injuryResultDefender — cached across sub-steps
    pub injury_result_defender: Option<Box<InjuryResult>>,
    /// Java: state.saboteurTriggeredAttacker
    pub saboteur_triggered_attacker: bool,
    /// Java: state.usingSaboteurAttacker (Boolean tristate)
    pub using_saboteur_attacker: Option<bool>,
    /// Java: state.saboteurTriggeredDefender
    pub saboteur_triggered_defender: bool,
    /// Java: state.usingSaboteurDefender (Boolean tristate)
    pub using_saboteur_defender: Option<bool>,
}

impl StepDropFallingPlayers {
    pub fn new() -> Self {
        Self {
            old_defender_state: None,
            injury_result_defender: None,
            saboteur_triggered_attacker: false,
            using_saboteur_attacker: None,
            saboteur_triggered_defender: false,
            using_saboteur_defender: None,
        }
    }
}

impl Default for StepDropFallingPlayers {
    fn default() -> Self { Self::new() }
}

impl Step for StepDropFallingPlayers {
    fn id(&self) -> StepId { StepId::DropFallingPlayers }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: CLIENT_USE_SKILL → handleSkillCommand → EXECUTE_STEP
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::OldDefenderState(v) => {
                self.old_defender_state = Some(*v);
                true
            }
            _ => false,
        }
    }
}

impl StepDropFallingPlayers {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: if (getGameState().executeStepHooks(this, state)) return;
        // TODO(StepHooks): Saboteur dialog not yet ported.

        let attacker_id = game.acting_player.player_id.clone().unwrap_or_default();
        let defender_id = game.defender_id.clone().unwrap_or_default();

        // ── Defender fall ──────────────────────────────────────────────────────

        let mut defender_state = game.field_model.player_state(&defender_id);
        let defender_coord = game.field_model.player_coordinate(&defender_id)
            .unwrap_or(FieldCoordinate::new(0, 0));

        // Java: if (FALLING && isRooted) changeRooted(false)
        if let Some(s) = defender_state {
            if s.base() == PS_FALLING && s.is_rooted() {
                let new_s = s.change_rooted(false);
                game.field_model.set_player_state(&defender_id, new_s);
                defender_state = Some(new_s);
            }
        }
        // Java: if (HIT_ON_GROUND) changeBase(FALLING)
        if let Some(s) = defender_state {
            if s.base() == PS_HIT_ON_GROUND {
                let new_s = s.change_base(PS_FALLING);
                game.field_model.set_player_state(&defender_id, new_s);
                defender_state = Some(new_s);
            }
        }

        let defender_is_falling = defender_state.map(|s| s.base() == PS_FALLING).unwrap_or(false);

        if defender_is_falling && self.injury_result_defender.is_none() {
            // Java: pick injuryType based on oldDefenderState
            // Ball and Chain + Violent Innovator grantsSpp check — stub false (skill props not yet wired)
            let grants_spp = false;
            let injury_type_name = match self.old_defender_state {
                Some(s) if s.is_stunned() => {
                    if grants_spp { "InjuryTypeBlockStunnedForSpp" } else { "InjuryTypeBlockStunned" }
                }
                Some(s) if s.is_prone_or_stunned() => {
                    if grants_spp { "InjuryTypeBlockProneForSpp" } else { "InjuryTypeBlockProne" }
                }
                _ => "InjuryTypeBlock",
            };
            // Saboteur overrides (not yet ported)
            let injury_type_name = if self.saboteur_triggered_defender { "InjuryTypeSaboteur" }
                else if self.saboteur_triggered_attacker { "InjuryTypeSabotaged" }
                else { injury_type_name };

            let result = handle_injury_by_name(
                game, rng, injury_type_name,
                Some(&attacker_id.clone()), &defender_id,
                defender_coord, None, None, ApothecaryMode::Defender,
            );
            self.injury_result_defender = Some(Box::new(result));
        }

        // Java: droppedOwnTeam = FALLING && defender.team == attacker.team
        //                         && oldDefenderState != null && !oldDefenderState.isProneOrStunned()
        let attacker_team_id = game.player_team_id(&attacker_id).map(|s| s.to_owned());
        let defender_team_id = game.player_team_id(&defender_id).map(|s| s.to_owned());
        let dropped_own_team = defender_is_falling
            && attacker_team_id.is_some()
            && attacker_team_id == defender_team_id
            && self.old_defender_state.is_some()
            && !self.old_defender_state.map(|s| s.is_prone_or_stunned()).unwrap_or(false);

        let mut out = StepOutcome::next();

        if let Some(ref injury_result) = self.injury_result_defender {
            let dpc = DropPlayerContext {
                injury_result: Some(injury_result.clone()),
                end_turn: dropped_own_team,
                eligible_for_safe_pair_of_hands: true,
                label: None,
                player_id: if defender_id.is_empty() { None } else { Some(defender_id.clone()) },
                apothecary_mode: Some(ApothecaryMode::Defender),
                requires_armour_break: false,
                ..DropPlayerContext::new()
            };

            if self.saboteur_triggered_defender {
                // Bypass Steady Footing — publish DROP_PLAYER_CONTEXT + INJURY_RESULT directly
                out = out.publish(StepParameter::DropPlayerContext(Box::new(dpc)));
                out = out.publish(StepParameter::InjuryResult(injury_result.clone()));
            } else {
                let ctx = SteadyFootingContext::from_drop_player(dpc);
                out = out.publish(StepParameter::SteadyFootingContext(Box::new(ctx)));
            }
        } else if dropped_own_team && !self.saboteur_triggered_defender && !self.saboteur_triggered_attacker {
            out = out.publish(StepParameter::EndTurn(true));
        }

        // ── Attacker fall ──────────────────────────────────────────────────────

        let mut attacker_state = game.field_model.player_state(&attacker_id);
        let attacker_coord = game.field_model.player_coordinate(&attacker_id)
            .unwrap_or(FieldCoordinate::new(0, 0));

        // Java: if (FALLING && isRooted) changeRooted(false)
        if let Some(s) = attacker_state {
            if s.base() == PS_FALLING && s.is_rooted() {
                let new_s = s.change_rooted(false);
                game.field_model.set_player_state(&attacker_id, new_s);
                attacker_state = Some(new_s);
            }
        }

        let attacker_is_falling = attacker_state.map(|s| s.base() == PS_FALLING).unwrap_or(false);

        if attacker_is_falling {
            if !(self.saboteur_triggered_defender || self.saboteur_triggered_attacker) {
                out = out.publish(StepParameter::EndTurn(true));
            }

            let fell_from_rush = game.acting_player.fell_from_rush;

            let injury_type_attacker = if fell_from_rush {
                "InjuryTypeDropGFI"
            } else if self.saboteur_triggered_attacker {
                "InjuryTypeSaboteur"
            } else if self.saboteur_triggered_defender {
                "InjuryTypeSabotaged"
            } else {
                "InjuryTypeBlock"
            };

            // Java: defender passed as attacker of the injury (punching back)
            let injury_result_attacker = handle_injury_by_name(
                game, rng, injury_type_attacker,
                Some(&defender_id),
                &attacker_id,
                attacker_coord, None, None, ApothecaryMode::Attacker,
            );

            if self.saboteur_triggered_attacker {
                let dpc = DropPlayerContext {
                    injury_result: Some(Box::new(injury_result_attacker.clone())),
                    end_turn: false,
                    eligible_for_safe_pair_of_hands: false,
                    label: None,
                    player_id: if attacker_id.is_empty() { None } else { Some(attacker_id.clone()) },
                    apothecary_mode: Some(ApothecaryMode::Attacker),
                    requires_armour_break: false,
                    ..DropPlayerContext::new()
                };
                out = out.publish(StepParameter::DropPlayerContext(Box::new(dpc)));
                out = out.publish(StepParameter::InjuryResult(Box::new(injury_result_attacker)));
            } else {
                // Java: new SteadyFootingContext(injuryResultAttacker, deferredCommands)
                // deferredCommands not yet ported — always empty; InjuryResult variant
                let ctx = SteadyFootingContext::from_injury_result(injury_result_attacker);
                out = out.publish(StepParameter::SteadyFootingContext(Box::new(ctx)));
            }
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::{PlayerState, PS_FALLING, PS_HIT_ON_GROUND, PS_STANDING, PS_STUNNED, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    fn add_player(game: &mut Game, team: &str, id: &str, coord: FieldCoordinate, state_base: u32) {
        let p = Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
        };
        if team == "home" { game.team_home.players.push(p); }
        else { game.team_away.players.push(p); }
        game.field_model.set_player_coordinate(id, coord);
        game.field_model.set_player_state(id, PlayerState::new(state_base));
    }

    #[test]
    fn start_no_players_returns_next() {
        let mut game = make_game();
        let mut step = StepDropFallingPlayers::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn old_defender_state_parameter_accepted() {
        let mut step = StepDropFallingPlayers::default();
        let state = PlayerState::new(PS_STUNNED);
        assert!(step.set_parameter(&StepParameter::OldDefenderState(state)));
        assert_eq!(step.old_defender_state, Some(state));
    }

    #[test]
    fn unknown_parameter_rejected() {
        let mut step = StepDropFallingPlayers::default();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }

    #[test]
    fn defender_falling_publishes_steady_footing_context() {
        let mut game = make_game();
        let coord = FieldCoordinate::new(5, 5);
        add_player(&mut game, "home", "atk", coord, PS_STANDING);
        add_player(&mut game, "away", "def", coord, PS_FALLING);
        game.acting_player.player_id = Some("atk".into());
        game.defender_id = Some("def".into());

        let mut step = StepDropFallingPlayers::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::SteadyFootingContext(_))),
            "expected SteadyFootingContext for falling defender");
    }

    #[test]
    fn defender_hit_on_ground_upgraded_to_falling() {
        let mut game = make_game();
        let coord = FieldCoordinate::new(5, 5);
        add_player(&mut game, "home", "atk", coord, PS_STANDING);
        add_player(&mut game, "away", "def", coord, PS_HIT_ON_GROUND);
        game.acting_player.player_id = Some("atk".into());
        game.defender_id = Some("def".into());

        let mut step = StepDropFallingPlayers::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        // HIT_ON_GROUND is upgraded → injury is performed → STEADY_FOOTING_CONTEXT published
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::SteadyFootingContext(_))));
    }

    #[test]
    fn attacker_falling_publishes_steady_footing_context_and_end_turn() {
        let mut game = make_game();
        let coord = FieldCoordinate::new(5, 5);
        add_player(&mut game, "home", "atk", coord, PS_FALLING);
        add_player(&mut game, "away", "def", coord, PS_STANDING);
        game.acting_player.player_id = Some("atk".into());
        game.defender_id = Some("def".into());

        let mut step = StepDropFallingPlayers::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::SteadyFootingContext(_))),
            "expected SteadyFootingContext for falling attacker");
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))),
            "expected END_TURN for falling attacker");
    }

    #[test]
    fn no_players_falling_no_parameters_published() {
        let mut game = make_game();
        let coord = FieldCoordinate::new(5, 5);
        add_player(&mut game, "home", "atk", coord, PS_STANDING);
        add_player(&mut game, "away", "def", coord, PS_STANDING);
        game.acting_player.player_id = Some("atk".into());
        game.defender_id = Some("def".into());

        let mut step = StepDropFallingPlayers::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.is_empty(), "expected no parameters published");
    }

    #[test]
    fn saboteur_defender_bypasses_steady_footing() {
        let mut game = make_game();
        let coord = FieldCoordinate::new(5, 5);
        add_player(&mut game, "home", "atk", coord, PS_STANDING);
        add_player(&mut game, "away", "def", coord, PS_FALLING);
        game.acting_player.player_id = Some("atk".into());
        game.defender_id = Some("def".into());

        let mut step = StepDropFallingPlayers::new();
        step.saboteur_triggered_defender = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Should publish DROP_PLAYER_CONTEXT + INJURY_RESULT, NOT STEADY_FOOTING_CONTEXT
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::DropPlayerContext(_))));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::InjuryResult(_))));
        assert!(!out.published.iter().any(|p| matches!(p, StepParameter::SteadyFootingContext(_))));
    }
}
