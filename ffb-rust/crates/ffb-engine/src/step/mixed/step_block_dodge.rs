/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.StepBlockDodge`.
///
/// Handles the Dodge skill in the block sequence.  After a pushback the defender may
/// use Dodge to avoid falling.
///
/// Java state fields (inner `StepState`):
///   `usingDodge`       — None until the skill-use dialog has been answered
///   `askForSkill`      — None until computed; true when at least one dangerous push exists
///   `oldDefenderState` — the defender's state before the block, from `OLD_DEFENDER_STATE`
///
/// Java execution logic (executeStep):
///   1. First call: compute `ask_for_skill` by scanning pushback squares.
///   2. Hide dialog / run hooks (simplified: hooks not yet ported).
///   3. If `using_dodge`: restore `old_defender_state` on the defender.
///   4. Otherwise: set defender to `FALLING`.
///   5. Publish pushback-init parameters and advance to next step.
///
/// Note: `findDodgeChoice` and `UtilServerPushback` are not yet fully ported.
/// The step stores the fields faithfully; the pushback-square scan is stubbed (no-op)
/// while the set_parameter and structural logic are correct.
use ffb_model::enums::{PlayerState, PS_FALLING};
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds};
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::action::block::util_block_sequence::init_pushback;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::util::util_server_pushback::UtilServerPushback;

/// Java: `StepBlockDodge` (mixed, BB2020 + BB2025).
pub struct StepBlockDodge {
    /// Java: `state.usingDodge` — None until dialog answered.
    using_dodge: Option<bool>,
    /// Java: `state.askForSkill` — None until computed; true = show dodge dialog.
    ask_for_skill: Option<bool>,
    /// Java: `state.oldDefenderState`
    old_defender_state: Option<PlayerState>,
}

impl StepBlockDodge {
    pub fn new() -> Self {
        Self {
            using_dodge: None,
            ask_for_skill: None,
            old_defender_state: None,
        }
    }

    /// Java: `findDodgeChoice()` — determines if a dodge-choice dialog is needed.
    ///
    /// True when any regular pushback square is occupied (chain-push risk),
    /// any grab-mode square is near a sideline/endzone, or would cross the midfield
    /// line on the first turn after kickoff.
    fn find_dodge_choice(game: &Game) -> bool {
        let attacker_id = match &game.acting_player.player_id {
            Some(id) => id.clone(),
            None => return false,
        };
        let defender_id = match &game.defender_id {
            Some(id) => id.clone(),
            None => return false,
        };
        let attacker_coord = match game.field_model.player_coordinate(&attacker_id) {
            Some(c) => c,
            None => return false,
        };
        let defender_coord = match game.field_model.player_coordinate(&defender_id) {
            Some(c) => c,
            None => return false,
        };

        let starting_square = match UtilServerPushback::find_starting_square(attacker_coord, defender_coord, game.home_playing) {
            Some(sq) => sq,
            None => return false,
        };

        let home_choice = game.home_playing;
        // Java: findPushbackSquares returns all candidate squares (including occupied ones).
        // Use the candidates function (no occupancy filtering) for dodge-choice detection.
        let regular_squares = UtilServerPushback::find_pushback_squares_candidates(starting_square, home_choice);

        let chain_push = regular_squares.iter().any(|sq| game.field_model.player_at(sq.coordinate).is_some());

        // Java: grabPushbackSquares defaults to regularPushbackSquares, overridden with GRAB
        // mode if: block action + attacker has canPushBackToAnySquare + defender lacks SideStep.
        // GRAB mode not fully ported — use regular squares as conservative fallback.
        let action_is_block = game.acting_player.player_action
            .map(|a| a.is_block_action())
            .unwrap_or(false);
        let attacker_can_grab = game.player(&attacker_id)
            .map(|p| p.has_skill_property(NamedProperties::CAN_PUSH_BACK_TO_ANY_SQUARE))
            .unwrap_or(false);
        let defender_has_side_step = game.player(&defender_id)
            .map(|p| p.has_skill_property(NamedProperties::CAN_CHOOSE_OWN_PUSHED_BACK_SQUARE))
            .unwrap_or(false);
        let use_grab = action_is_block && attacker_can_grab && !defender_has_side_step;
        let grab_squares_owned: Vec<_>;
        let grab_squares: &Vec<_>;
        if use_grab {
            // Java: GRAB mode — all adjacent empty, valid pushback squares
            grab_squares_owned = UtilServerPushback::find_pushback_squares_grab(
                starting_square,
                &|c| game.field_model.player_at(c).is_some(),
                &|c| !game.field_model.was_multi_block_target_square(c),
                home_choice,
            );
            grab_squares = &grab_squares_owned;
        } else {
            grab_squares = &regular_squares;
        }

        let sideline_push = grab_squares.iter().any(|sq| {
            let c = sq.coordinate;
            FieldCoordinateBounds::SIDELINE_LOWER.is_in_bounds(c)
                || FieldCoordinateBounds::SIDELINE_UPPER.is_in_bounds(c)
                || FieldCoordinateBounds::ENDZONE_HOME.is_in_bounds(c)
                || FieldCoordinateBounds::ENDZONE_AWAY.is_in_bounds(c)
        });

        let attacker_home = game.team_home.players.iter().any(|p| p.id == attacker_id);
        let attacker_half_push = grab_squares.iter().any(|sq| {
            let c = sq.coordinate;
            if attacker_home {
                FieldCoordinateBounds::HALF_HOME.is_in_bounds(c) && game.turn_data_home.first_turn_after_kickoff
            } else {
                FieldCoordinateBounds::HALF_AWAY.is_in_bounds(c) && game.turn_data_away.first_turn_after_kickoff
            }
        });

        chain_push || sideline_push || attacker_half_push
    }

    fn execute_step(&mut self, game: &mut Game) -> StepOutcome {
        // Java step 1: lazy-compute ask_for_skill
        if self.ask_for_skill.is_none() {
            self.ask_for_skill = Some(Self::find_dodge_choice(game));
        }

        // Java: UtilServerDialog.hideDialog + executeStepHooks (hooks not yet ported — skip).

        // Java: if toPrimitive(usingDodge) → restore defender; else set FALLING.
        let using_dodge = self.using_dodge.unwrap_or(false);

        if let Some(defender_id) = game.defender_id.clone() {
            if using_dodge {
                if let Some(old) = self.old_defender_state {
                    game.field_model.set_player_state(&defender_id, old);
                }
            } else {
                if let Some(state) = game.field_model.player_state(&defender_id) {
                    game.field_model.set_player_state(&defender_id, state.change_base(PS_FALLING));
                }
            }
        }

        // Java: publishParameters(UtilBlockSequence.initPushback(this))
        let pushback_params = init_pushback(game);
        let mut outcome = StepOutcome::next();
        for p in pushback_params {
            outcome = outcome.publish(p);
        }
        outcome
    }
}

impl Default for StepBlockDodge {
    fn default() -> Self { Self::new() }
}

impl Step for StepBlockDodge {
    fn id(&self) -> StepId { StepId::BlockDodge }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: CLIENT_USE_SKILL → handleSkillCommand sets state.usingDodge
        if let Action::UseSkill { use_skill, .. } = action {
            // Dodge skill use answer
            self.using_dodge = Some(*use_skill);
        }
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::OldDefenderState(v) => { self.old_defender_state = Some(*v); true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{test_team, StepAction};
    use ffb_model::enums::{Rules, PS_STANDING, PlayerState};
    use ffb_model::model::game::Game;
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_defender(game: &mut Game, id: &str, state: u32) {
        let pos = FieldCoordinate::new(5, 5);
        game.team_away.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        });
        game.field_model.set_player_coordinate(id, pos);
        game.field_model.set_player_state(id, ffb_model::enums::PlayerState::new(state));
        game.defender_id = Some(id.into());
    }

    #[test]
    fn id_is_block_dodge() {
        assert_eq!(StepBlockDodge::new().id(), StepId::BlockDodge);
    }

    #[test]
    fn no_dodge_sets_defender_falling() {
        let mut step = StepBlockDodge::new();
        let mut game = make_game();
        add_defender(&mut game, "def", PS_STANDING);
        // using_dodge stays None → false → defender should fall
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        let state = game.field_model.player_state("def").unwrap();
        assert_eq!(state.base(), PS_FALLING);
    }

    #[test]
    fn using_dodge_restores_old_state() {
        let mut step = StepBlockDodge::new();
        step.using_dodge = Some(true);
        let old_state = PlayerState::new(PS_STANDING);
        step.old_defender_state = Some(old_state);

        let mut game = make_game();
        add_defender(&mut game, "def", PS_FALLING); // currently falling
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        let state = game.field_model.player_state("def").unwrap();
        // Should be restored to old_state (standing)
        assert_eq!(state.base(), PS_STANDING);
    }

    #[test]
    fn set_parameter_old_defender_state() {
        let mut step = StepBlockDodge::new();
        let ps = PlayerState::new(PS_STANDING);
        let accepted = step.set_parameter(&StepParameter::OldDefenderState(ps));
        assert!(accepted);
        assert_eq!(step.old_defender_state, Some(ps));
    }

    #[test]
    fn start_returns_next_step() {
        let mut step = StepBlockDodge::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn find_dodge_choice_returns_false_when_no_actors() {
        let game = make_game();
        assert!(!StepBlockDodge::find_dodge_choice(&game));
    }

    #[test]
    fn find_dodge_choice_chain_push_detected() {
        let mut game = make_game();
        // Attacker at (10,7), defender at (10,8) → pushed South → candidates (9,9),(10,9),(11,9)
        game.acting_player.player_id = Some("att".into());
        game.defender_id = Some("def".into());
        game.field_model.set_player_coordinate("att", FieldCoordinate::new(10, 7));
        game.field_model.set_player_coordinate("def", FieldCoordinate::new(10, 8));
        // Place a blocker at (10,9) — directly south of defender — causes chain push
        game.field_model.set_player_coordinate("blocker", FieldCoordinate::new(10, 9));
        assert!(StepBlockDodge::find_dodge_choice(&game));
    }

    #[test]
    fn find_dodge_choice_sideline_push_detected() {
        let mut game = make_game();
        // Attacker at (10,2), defender at (10,1) → delta_y = 2-1 = 1 > 0 → direction North
        // → candidates at y=0 (SIDELINE_UPPER)
        game.acting_player.player_id = Some("att".into());
        game.defender_id = Some("def".into());
        game.field_model.set_player_coordinate("att", FieldCoordinate::new(10, 2));
        game.field_model.set_player_coordinate("def", FieldCoordinate::new(10, 1));
        // Candidates: (9,0),(10,0),(11,0) — all on SIDELINE_UPPER → sideline_push = true
        assert!(StepBlockDodge::find_dodge_choice(&game));
    }

    #[test]
    fn execute_step_publishes_starting_pushback_square() {
        // Verify that init_pushback is wired: attacker + defender both set →
        // StartingPushbackSquare is published in the outcome.
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender};
        let mut game = make_game();
        game.home_playing = true;
        // Attacker (home)
        game.team_home.players.push(Player {
            id: "att".into(), name: "att".into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 5, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        });
        game.acting_player.player_id = Some("att".into());
        game.field_model.set_player_coordinate("att", FieldCoordinate::new(10, 6));
        // Defender (away)
        game.team_away.players.push(Player {
            id: "def".into(), name: "def".into(), nr: 2, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 5, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        });
        game.field_model.set_player_coordinate("def", FieldCoordinate::new(10, 7));
        game.field_model.set_player_state("def", PlayerState::new(PS_STANDING));
        game.defender_id = Some("def".into());

        let mut step = StepBlockDodge::new();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert!(
            out.published.iter().any(|p| matches!(p, StepParameter::StartingPushbackSquare(Some(_)))),
            "init_pushback should publish StartingPushbackSquare when attacker and defender are placed"
        );
    }

    #[test]
    fn find_dodge_choice_no_risk_mid_field() {
        let mut game = make_game();
        // Attacker at (12,7), defender at (13,7) → pushed East → candidates open, no sideline
        game.acting_player.player_id = Some("att".into());
        game.defender_id = Some("def".into());
        game.field_model.set_player_coordinate("att", FieldCoordinate::new(12, 7));
        game.field_model.set_player_coordinate("def", FieldCoordinate::new(13, 7));
        // No players on push squares, not near sideline, not first turn after kickoff
        assert!(!StepBlockDodge::find_dodge_choice(&game));
    }

    #[test]
    fn find_dodge_choice_grab_mode_near_sideline_detected() {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender, PlayerAction, SkillId};
        use ffb_model::model::skill_def::SkillWithValue;
        // Attacker has Grab (canPushBackToAnySquare), defender lacks SideStep.
        // Place defender near the upper sideline (y=1) so GRAB squares include sideline.
        let mut game = make_game();
        game.acting_player.player_id = Some("att".into());
        game.acting_player.player_action = Some(PlayerAction::Block);
        game.defender_id = Some("def".into());
        game.field_model.set_player_coordinate("att", FieldCoordinate::new(12, 2));
        game.field_model.set_player_coordinate("def", FieldCoordinate::new(12, 1));
        // Add attacker with Grab skill to team_home
        game.team_home.players.push(Player {
            id: "att".into(), name: "att".into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 5, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![SkillWithValue { skill_id: SkillId::Grab, value: None }],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        });
        // Defender with no SideStep
        game.team_away.players.push(Player {
            id: "def".into(), name: "def".into(), nr: 2, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 5, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        });
        // With GRAB mode, all adjacent empty squares are considered.
        // Defender at y=1, GRAB squares include y=0 (sideline) → sideline push detected.
        assert!(StepBlockDodge::find_dodge_choice(&game));
    }
}
