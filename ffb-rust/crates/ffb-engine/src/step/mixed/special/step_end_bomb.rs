/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.special.StepEndBomb`.
///
/// Final step of the bomb sequence (mixed, BB2020 + BB2025).
///
/// More complex than the BB2016 version: handles PassState, the
/// `throwTwoBombs` / `canUseThrowBombActionTwice` skill, `allowMoveAfterBomb`,
/// and the AllYouCanEat step.
///
/// Parameters consumed:
///   - CATCHER_ID
///   - END_TURN
///   - BOMB_EXPLODED
///
/// Java: uses `SequenceGeneratorFactory` to select edition-specific Pass/EndPlayerAction/Move.
/// Rust: uses BB2025 generators (the primary target for mixed).
use ffb_model::enums::{PlayerAction, TurnMode};
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepId, StepOutcome, StepParameter};
use crate::step::generator::bb2025::{EndPlayerAction, Pass};
use crate::step::generator::bb2025::end_player_action::EndPlayerActionParams;
use crate::step::generator::bb2025::pass::PassParams;
use crate::step::util_server_steps::{change_player_action, check_touchdown};

/// Java: `StepEndBomb` (mixed/special, BB2020 + BB2025).
pub struct StepEndBomb {
    /// Java: fCatcherId
    pub catcher_id: Option<String>,
    /// Java: fEndTurn
    pub end_turn: bool,
    /// Java: fBombExploded
    pub bomb_exploded: bool,
}

impl StepEndBomb {
    pub fn new() -> Self {
        Self { catcher_id: None, end_turn: false, bomb_exploded: false }
    }

    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.end_turn |= check_touchdown(game);

        // Java: boolean removePassCoordinate = true;
        let mut remove_pass_coordinate = true;

        if self.end_turn || self.catcher_id.is_none() || self.bomb_exploded {
            // Restore home_playing from bomb turn mode
            if game.turn_mode.is_bomb_turn() {
                game.home_playing = matches!(
                    game.turn_mode,
                    TurnMode::BombHome | TurnMode::BombHomeBlitz
                );
                // Restore turn mode: blitz → BLITZ, other bomb → REGULAR
                game.turn_mode =
                    if matches!(game.turn_mode, TurnMode::BombHomeBlitz | TurnMode::BombAwayBlitz) {
                        TurnMode::Blitz
                    } else {
                        TurnMode::Regular
                    };
            }

            // Java: PassState state = getGameState().getPassState();
            // PassState is a stub in Rust; we read from game fields directly.
            // Java reads: state.getOriginalBombardier(), state.getThrowTwoBombs(),
            //             state.isAllowMoveAfterBomb(), state.getThrowTwoBombs()
            //
            // Since PassState is a stub (all None), the complex branching collapses:
            //   originalBomber == actingPlayer (state.getOriginalBombardier() is None)
            //   skill = null (no canUseThrowBombActionTwice property)
            //   threwOnlyFirstBomb = toPrimitive(null) → false
            //   state.getThrowTwoBombs() == null → else: push EndPlayerAction
            //
            // We mirror the final else-branch which is the common path.
            //
            // Note: Java's `if (!fEndTurn && threwOnlyFirstBomb && skill != null && ...)`
            // branch (StepEndBomb.java lines 114-118) is the only call site of
            // `actingPlayer.setMustCompleteAction(true)` in the whole Java codebase. It
            // requires `skill != null` (the `canUseThrowBombActionTwice` property, i.e.
            // Ninja's second bomb throw), which is unreachable here because `skill` is
            // always `null` given the PassState stub above — so there is currently no
            // reachable call site to wire `ActingPlayer::set_must_complete_action(true)`
            // to in this translation (see `acting_player.rs` for the now-real field/methods).
            // Java: `if (originalBomber != actingPlayer.getPlayer()) { changePlayerAction(
            // originalBomber, THROW_BOMB); if (prone/stunned) changeActive(false); }` — the
            // acting-player slot is handed BACK to the original bomber before EndPlayerAction, so
            // the activation being ended is the BOMBER's, not the catcher's. Without this, a
            // catcher who re-threw the bomb was retired STANDING+INACTIVE by the end of the
            // activation and could never activate for the rest of the game half (goblin bb2016
            // seed 4: home_10 caught+re-threw at turn 4; Java's harness picked him at step 56,
            // Rust's engine rejected him as inactive and the activation streams forked).
            // (`game.original_bombardier` is the Rust home of Java's PassState.originalBombardier;
            // the older comment claiming PassState is an all-None stub predates it.)
            if let Some(orig) = game.original_bombardier.clone() {
                if game.acting_player.player_id.as_deref() != Some(orig.as_str()) {
                    // Java captures the bomber's PlayerState BEFORE changePlayerAction — the
                    // acting-player switch stamps the new acting player MOVING (UtilActingPlayer's
                    // "show acting player as moving"), and Java's prone/stunned write puts the
                    // CAPTURED state back with active=false. Reading the state after the switch
                    // saw MOVING, skipped the write, and a bomber stunned by his own fumbled bomb
                    // stood back up when the activation retired (goblin bb2025 seed 65 step 19).
                    let pre_state = game.field_model.player_state(&orig);
                    change_player_action(game, &orig, PlayerAction::ThrowBomb, false);
                    if let Some(ps) = pre_state {
                        if ps.is_prone_or_stunned() {
                            game.field_model.set_player_state(&orig, ps.change_active(false));
                        } else if game.rules == ffb_model::enums::Rules::Bb2016 && !ps.is_standing() {
                            // bb2016 ONLY: its apothecary applies the injury via the AcceptInjury
                            // answer BEFORE this step runs, so change_player_action's MOVING stamp
                            // erased a fresh KO (goblin bb2016 seed 29 step 34 — the bomber's own
                            // bomb KO'd him and he retired STANDING in the box). bb2020/bb2025's
                            // ordering never has an applied KO here and measured GREEN without the
                            // write-back (adding it unconditionally regressed both).
                            game.field_model.set_player_state(&orig, ps);
                        }
                    }
                }
            }
            let seq = EndPlayerAction::build_sequence(&EndPlayerActionParams {
                feeding_allowed: false,
                end_player_action: true,
                end_turn: self.end_turn,
                check_forgo: false,
                            rules: game.rules,
            });
            game.pass_coordinate = None;
            game.thrower_id = None;
            game.thrower_action = None;
            return StepOutcome::next().push_seq(seq);
        }

        // Catcher caught the bomb and is re-throwing it
        // Java: game.setPassCoordinate(null) (done explicitly before changePlayerAction)
        game.pass_coordinate = None;
        let catcher_id = self.catcher_id.as_ref().unwrap().clone();
        game.home_playing = game.team_home.players.iter().any(|p| p.id == catcher_id);
        change_player_action(game, &catcher_id, PlayerAction::ThrowBomb, false);

        // Java pushes the re-throw sequence via the game's per-ruleset SequenceGeneratorFactory,
        // so a bb2016 game gets the BB2016 Pass sequence. The step ORDER differs materially:
        // bb2016 places MissedPass AFTER ResolvePass (an inaccurate goto skips ResolvePass
        // entirely), while bb2025 places it BEFORE — so building the bb2025 sequence for a bb2016
        // re-throw let ResolvePass run after the scatter and drag the bomb back to the original
        // target square (goblin bb2016 seed 3 step 6: both engines scattered the inaccurate
        // re-throw to the empty (18,6), then Rust's ResolvePass reset bomb_coordinate to (21,6)
        // and rolled a catch for the player standing there).
        let seq = if game.rules == ffb_model::enums::Rules::Bb2016 {
            crate::step::generator::bb2016::pass::Pass::build_sequence(
                &crate::step::generator::bb2016::pass::PassParams { target_coordinate: None },
            )
        } else {
            Pass::build_sequence(&PassParams { target_coordinate: None, rules: game.rules })
        };

        // Java: removePassCoordinate stays true so passCoordinate is cleared again here
        if remove_pass_coordinate {
            game.pass_coordinate = None;
        }
        game.thrower_id = None;
        game.thrower_action = None;
        StepOutcome::next().push_seq(seq)
    }
}

impl Default for StepEndBomb {
    fn default() -> Self { Self::new() }
}

impl Step for StepEndBomb {
    fn id(&self) -> StepId { StepId::EndBomb }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::CatcherId(v) => { self.catcher_id = v.clone(); true }
            StepParameter::EndTurn(v) => { self.end_turn = *v; true }
            StepParameter::BombExploded(v) => { self.bomb_exploded = *v; true }
            _ => false,
        }
    }

    // Java: setParameter consume()s these keys.
    fn consumes_parameter(&self, param: &StepParameter) -> bool {
        matches!(param,
            StepParameter::CatcherId(_) | StepParameter::EndTurn(_)
            | StepParameter::BombExploded(_))
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, StepId, test_team};
    use ffb_model::enums::{Rules, PS_STANDING, PlayerState, PlayerType, PlayerGender};
    use ffb_model::model::player::Player;
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_player(game: &mut Game, team: &str, id: &str) {
        let p = Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };
        if team == "home" {
            game.team_home.players.push(p);
        } else {
            game.team_away.players.push(p);
        }
        game.field_model.set_player_coordinate(id, FieldCoordinate::new(5, 5));
        game.field_model.set_player_state(id, PlayerState::new(PS_STANDING));
    }

    #[test]
    fn no_catcher_pushes_end_player_action() {
        let mut game = make_game();
        game.turn_mode = TurnMode::BombHome;
        let mut step = StepEndBomb::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 1);
        // BB2025 EndPlayerAction sequence starts with RemoveTargetSelectionState
        assert_eq!(out.pushes[0][0].step_id, StepId::RemoveTargetSelectionState);
    }

    #[test]
    fn end_turn_flag_pushes_end_player_action() {
        let mut game = make_game();
        game.turn_mode = TurnMode::BombHome;
        let mut step = StepEndBomb::new();
        step.end_turn = true;
        step.catcher_id = Some("p1".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::RemoveTargetSelectionState);
    }

    #[test]
    fn bomb_exploded_pushes_end_player_action() {
        let mut game = make_game();
        game.turn_mode = TurnMode::BombHome;
        let mut step = StepEndBomb::new();
        step.bomb_exploded = true;
        step.catcher_id = Some("p1".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::RemoveTargetSelectionState);
    }

    #[test]
    fn bomb_home_sets_home_playing_true() {
        let mut game = make_game();
        game.turn_mode = TurnMode::BombHome;
        game.home_playing = false;
        let mut step = StepEndBomb::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.home_playing);
    }

    #[test]
    fn bomb_away_sets_home_playing_false() {
        let mut game = make_game();
        game.turn_mode = TurnMode::BombAway;
        game.home_playing = true;
        let mut step = StepEndBomb::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.home_playing);
    }

    #[test]
    fn bomb_home_blitz_restores_blitz_mode() {
        let mut game = make_game();
        game.turn_mode = TurnMode::BombHomeBlitz;
        let mut step = StepEndBomb::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.turn_mode, TurnMode::Blitz);
    }

    #[test]
    fn bomb_away_non_blitz_restores_regular_mode() {
        let mut game = make_game();
        game.turn_mode = TurnMode::BombAway;
        let mut step = StepEndBomb::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.turn_mode, TurnMode::Regular);
    }

    #[test]
    fn catcher_present_pushes_pass_sequence() {
        let mut game = make_game();
        game.turn_mode = TurnMode::BombHome;
        add_player(&mut game, "away", "catcher1");
        let mut step = StepEndBomb::new();
        step.catcher_id = Some("catcher1".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::InitPassing);
    }

    #[test]
    fn clears_pass_coordinate_and_thrower() {
        let mut game = make_game();
        game.turn_mode = TurnMode::BombHome;
        game.pass_coordinate = Some(FieldCoordinate::new(3, 4));
        game.thrower_id = Some("t1".into());
        game.thrower_action = Some(PlayerAction::ThrowBomb);
        let mut step = StepEndBomb::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.pass_coordinate.is_none());
        assert!(game.thrower_id.is_none());
        assert!(game.thrower_action.is_none());
    }

    #[test]
    fn set_parameter_catcher_id_accepted() {
        let mut step = StepEndBomb::new();
        assert!(step.set_parameter(&StepParameter::CatcherId(Some("p1".into()))));
        assert_eq!(step.catcher_id, Some("p1".into()));
    }

    #[test]
    fn set_parameter_end_turn_accepted() {
        let mut step = StepEndBomb::new();
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.end_turn);
    }

    #[test]
    fn set_parameter_bomb_exploded_accepted() {
        let mut step = StepEndBomb::new();
        assert!(step.set_parameter(&StepParameter::BombExploded(true)));
        assert!(step.bomb_exploded);
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        let mut step = StepEndBomb::new();
        assert!(!step.set_parameter(&StepParameter::HomeTeam(true)));
    }
}
