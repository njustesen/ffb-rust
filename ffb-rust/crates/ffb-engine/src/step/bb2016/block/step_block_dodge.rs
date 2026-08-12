/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.block.StepBlockDodge`.
///
/// Step in block sequence to handle skill DODGE.
///
/// Expects stepParameter OLD_DEFENDER_STATE to be set by a preceding step.
///
/// Java: executeStep() calls executeStepHooks(this, state) which dispatches to
/// DodgeBehaviour (BB2016). The hook logic is inlined here for headless translation.
use ffb_model::enums::{PlayerState, SkillId, PS_FALLING};
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::prompts::agent_prompt::AgentPrompt;
use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds};
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::action::block::util_block_sequence::init_pushback;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::util::util_server_pushback::UtilServerPushback;

/// Java: `StepBlockDodge.StepState` — inner class holding hook-visible state.
#[derive(Debug, Clone, Default)]
pub struct StepState {
    /// Java: `usingDodge` (Boolean — tristate: null/true/false)
    pub using_dodge: Option<bool>,
    /// Java: `oldDefenderState`
    pub old_defender_state: Option<PlayerState>,
}

/// Java: `StepBlockDodge` (bb2016/block).
pub struct StepBlockDodge {
    pub state: StepState,
}

impl StepBlockDodge {
    pub fn new() -> Self {
        Self {
            state: StepState::default(),
        }
    }

    /// Java: `DodgeBehaviour.findDodgeChoice()` — decides `state.usingDodge` when it is still null.
    ///
    /// Returns the tri-state that Java assigns to `usingDodge`:
    /// * `Some(false)` when the starting pushback square cannot be computed
    ///   (Java: `startingSquare == null` → `usingDodge = false`, "should not happen").
    /// * `Some(true)` when there is no chain-push / sideline / attacker-half risk
    ///   (Java: `usingDodge = true` — auto-use dodge, defender stays standing).
    /// * `None` when a risk exists → Java leaves `usingDodge` null and shows the
    ///   `DialogSkillUseParameter` dialog (headless: emit `AgentPrompt::SkillUse`).
    fn find_dodge_choice(game: &Game) -> Option<bool> {
        let attacker_id = match &game.acting_player.player_id {
            Some(id) => id.clone(),
            None => return Some(false),
        };
        let defender_id = match &game.defender_id {
            Some(id) => id.clone(),
            None => return Some(false),
        };
        let attacker_coord = match game.field_model.player_coordinate(&attacker_id) {
            Some(c) => c,
            None => return Some(false),
        };
        let defender_coord = match game.field_model.player_coordinate(&defender_id) {
            Some(c) => c,
            None => return Some(false),
        };

        let starting_square = match UtilServerPushback::find_starting_square(attacker_coord, defender_coord, game.home_playing) {
            Some(sq) => sq,
            None => return Some(false),
        };

        let home_choice = game.home_playing;
        let regular_squares = UtilServerPushback::find_pushback_squares_candidates(starting_square, home_choice);

        let chain_push = regular_squares.iter().any(|sq| game.field_model.player_at(sq.coordinate).is_some());

        // Java: grabPushbackSquares defaults to regularPushbackSquares, overridden with GRAB mode if:
        // block action + attacker has canPushBackToAnySquare + defender lacks SideStep.
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

        // Java: `usingDodge = true` when no risk; otherwise leave null → dialog.
        if !chain_push && !sideline_push && !attacker_half_push {
            Some(true)
        } else {
            None
        }
    }

    fn execute_step(&mut self, game: &mut Game) -> StepOutcome {
        // Java: DodgeBehaviour.handleExecuteStepHook → findDodgeChoice(step, state).
        // Only decides when usingDodge is still null (a command may already have set it).
        if self.state.using_dodge.is_none() {
            self.state.using_dodge = Self::find_dodge_choice(game);
        }

        // Java: if usingDodge == null → UtilServerDialog.showDialog(DialogSkillUseParameter(defenderId, Dodge)).
        // Headless: emit AgentPrompt::SkillUse so the agent answers (ParityRunner always USES Dodge;
        // random_agent's SkillUse default is use_skill:true) → matches Java's dialog answer.
        let using_dodge = match self.state.using_dodge {
            Some(v) => v,
            None => {
                let defender_id = match game.defender_id.clone() {
                    Some(id) => id,
                    None => return StepOutcome::next(),
                };
                return StepOutcome::cont().with_prompt(AgentPrompt::SkillUse {
                    player_id: defender_id,
                    skill_id: SkillId::Dodge as u16,
                    skill_name: format!("{:?}", SkillId::Dodge),
                });
            }
        };

        // Java: addReport(ReportSkillUse(defenderId, Dodge, usingDodge, AVOID_FALLING))
        if let Some(ref defender_id) = game.defender_id {
            use ffb_model::model::skill_use::SkillUse;
            use ffb_model::report::report_skill_use::ReportSkillUse;
            game.report_list.add(ReportSkillUse::new(
                Some(defender_id.clone()), SkillId::Dodge, using_dodge, SkillUse::AVOID_FALLING,
            ));
        }

        if let Some(defender_id) = game.defender_id.clone() {
            if using_dodge {
                // Java: fieldModel.setPlayerState(defender, oldDefenderState)
                if let Some(old) = self.state.old_defender_state {
                    game.field_model.set_player_state(&defender_id, old);
                }
            } else {
                // Java: fieldModel.setPlayerState(defender, defenderState.changeBase(FALLING))
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
        match action {
            Action::UseSkill { use_skill, .. } => {
                self.state.using_dodge = Some(*use_skill);
            }
            _ => {}
        }
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::OldDefenderState(s) => { self.state.old_defender_state = Some(*s); true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{Rules, PS_PRONE, PS_STANDING};
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    fn add_player(game: &mut Game, team_home: bool, id: &str, coord: FieldCoordinate, state_base: u32) {
        let player = Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };
        if team_home { game.team_home.players.push(player); }
        else { game.team_away.players.push(player); }
        game.field_model.set_player_coordinate(id, coord);
        game.field_model.set_player_state(id, PlayerState::new(state_base));
    }

    #[test]
    fn no_risk_auto_uses_dodge_restores_old_state() {
        // No chain push, no sideline risk → auto-dodge = true → defender restored to old state
        let mut step = StepBlockDodge::new();
        let old_state = PlayerState::new(PS_STANDING);
        step.state.old_defender_state = Some(old_state);

        let mut game = make_game();
        game.home_playing = true;
        // Attacker at (10,5), defender at (10,6) — push south, open squares
        add_player(&mut game, true, "att", FieldCoordinate::new(10, 5), PS_STANDING);
        add_player(&mut game, false, "def", FieldCoordinate::new(10, 6), PS_PRONE);
        game.acting_player.player_id = Some("att".into());
        game.defender_id = Some("def".into());

        step.start(&mut game, &mut GameRng::new(0));
        let state = game.field_model.player_state("def").unwrap();
        assert_eq!(state.base(), PS_STANDING, "No-risk auto-dodge should restore old state");
    }

    #[test]
    fn explicit_no_dodge_sets_defender_falling() {
        let mut step = StepBlockDodge::new();
        step.state.using_dodge = Some(false);

        let mut game = make_game();
        add_player(&mut game, false, "def", FieldCoordinate::new(5, 5), PS_STANDING);
        game.defender_id = Some("def".into());

        step.start(&mut game, &mut GameRng::new(0));
        let state = game.field_model.player_state("def").unwrap();
        assert_eq!(state.base(), PS_FALLING);
    }

    #[test]
    fn explicit_use_dodge_restores_old_state() {
        let mut step = StepBlockDodge::new();
        step.state.using_dodge = Some(true);
        let old = PlayerState::new(PS_STANDING);
        step.state.old_defender_state = Some(old);

        let mut game = make_game();
        add_player(&mut game, false, "def", FieldCoordinate::new(5, 5), PS_PRONE);
        game.defender_id = Some("def".into());

        step.start(&mut game, &mut GameRng::new(0));
        let state = game.field_model.player_state("def").unwrap();
        assert_eq!(state.base(), PS_STANDING);
    }

    #[test]
    fn set_parameter_old_defender_state() {
        let mut step = StepBlockDodge::new();
        let ps = PlayerState::new(PS_PRONE);
        assert!(step.set_parameter(&StepParameter::OldDefenderState(ps)));
        assert_eq!(step.state.old_defender_state.unwrap().base(), PS_PRONE);
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        let mut step = StepBlockDodge::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(false)));
    }

    #[test]
    fn use_skill_command_sets_using_dodge() {
        use ffb_mechanics::skills::SkillId;
        let mut step = StepBlockDodge::new();
        let mut game = make_game();
        step.handle_command(
            &Action::UseSkill { skill_id: SkillId::Dodge, use_skill: true },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(step.state.using_dodge, Some(true));
    }

    #[test]
    fn sideline_push_risk_emits_skill_use_prompt() {
        // Defender near sideline → Java leaves usingDodge null → showDialog.
        // Headless: emit AgentPrompt::SkillUse (agent answers use=true) — NOT auto-decide no-dodge.
        let mut step = StepBlockDodge::new();

        let mut game = make_game();
        game.home_playing = true;
        // Attacker at (10,2), defender at (10,1) → candidates include y=0 (sideline)
        add_player(&mut game, true, "att", FieldCoordinate::new(10, 2), PS_STANDING);
        add_player(&mut game, false, "def", FieldCoordinate::new(10, 1), PS_STANDING);
        game.acting_player.player_id = Some("att".into());
        game.defender_id = Some("def".into());

        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(
            matches!(out.prompt, Some(AgentPrompt::SkillUse { ref skill_name, .. }) if skill_name == "Dodge"),
            "sideline-push risk should emit a Dodge SkillUse prompt, got {:?}", out.prompt
        );
        // usingDodge must stay undecided until the agent answers.
        assert_eq!(step.state.using_dodge, None);
    }

    #[test]
    fn chain_push_risk_emits_skill_use_prompt() {
        let mut step = StepBlockDodge::new();
        let mut game = make_game();
        game.home_playing = true;
        // Attacker at (10,7), defender at (10,8), blocker at (10,9)
        add_player(&mut game, true, "att", FieldCoordinate::new(10, 7), PS_STANDING);
        add_player(&mut game, false, "def", FieldCoordinate::new(10, 8), PS_STANDING);
        add_player(&mut game, false, "blocker", FieldCoordinate::new(10, 9), PS_STANDING);
        game.acting_player.player_id = Some("att".into());
        game.defender_id = Some("def".into());

        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(
            matches!(out.prompt, Some(AgentPrompt::SkillUse { ref skill_name, .. }) if skill_name == "Dodge"),
            "chain-push risk should emit a Dodge SkillUse prompt, got {:?}", out.prompt
        );
        assert_eq!(step.state.using_dodge, None);
    }

    #[test]
    fn chain_push_risk_then_agent_uses_dodge_restores_standing() {
        // Full risk path: prompt emitted, agent answers use_skill=true (as ParityRunner does)
        // → defender restored to its old (standing) state, not knocked down.
        use ffb_mechanics::skills::SkillId as MechSkillId;
        let mut step = StepBlockDodge::new();
        step.state.old_defender_state = Some(PlayerState::new(PS_STANDING));
        let mut game = make_game();
        game.home_playing = true;
        add_player(&mut game, true, "att", FieldCoordinate::new(10, 7), PS_STANDING);
        add_player(&mut game, false, "def", FieldCoordinate::new(10, 8), PS_STANDING);
        add_player(&mut game, false, "blocker", FieldCoordinate::new(10, 9), PS_STANDING);
        game.acting_player.player_id = Some("att".into());
        game.defender_id = Some("def".into());

        // Defender starts knocked-down (block result set FALLING before the dodge check).
        game.field_model.set_player_state("def", PlayerState::new(PS_FALLING));

        step.start(&mut game, &mut GameRng::new(0));
        step.handle_command(
            &Action::UseSkill { skill_id: MechSkillId::Dodge, use_skill: true },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(
            game.field_model.player_state("def").unwrap().base(),
            PS_STANDING,
            "agent-used Dodge on a risk push should restore the defender to standing"
        );
    }

    #[test]
    fn start_publishes_starting_pushback_square_when_actors_set() {
        let mut step = StepBlockDodge::new();
        let mut game = make_game();
        game.home_playing = true;
        add_player(&mut game, true, "att", FieldCoordinate::new(10, 6), PS_STANDING);
        add_player(&mut game, false, "def", FieldCoordinate::new(10, 7), PS_STANDING);
        game.acting_player.player_id = Some("att".into());
        game.defender_id = Some("def".into());
        game.field_model.set_player_state("def", PlayerState::new(PS_STANDING));

        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert!(
            out.published.iter().any(|p| matches!(p, StepParameter::StartingPushbackSquare(Some(_)))),
            "init_pushback should publish StartingPushbackSquare"
        );
    }
}
