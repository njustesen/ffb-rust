use ffb_model::types::FieldCoordinate;
use ffb_model::model::game::Game;
use ffb_model::enums::ReRollSource;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::dice_interpreter::DiceInterpreter;
use crate::drop_player_context::SteadyFootingContext;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::abstract_step_with_re_roll::ReRollState;
use crate::step::abstract_step_with_re_roll::find_skill_reroll_source;
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};
use ffb_mechanics::bb2025::jump_mechanic::JumpMechanic;
use ffb_mechanics::jump_mechanic::JumpMechanic as JumpMechanicTrait;
use ffb_mechanics::modifiers::jump_modifier_factory::JumpModifierFactory;
use ffb_mechanics::modifiers::jump_context::JumpContext;
use crate::util::util_server_player_move::UtilServerPlayerMove;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.move.StepJump.
///
/// Resolves a jump (Leap / VLL): agility roll with tackle-zone modifiers.
/// On success: clears jumping flag, publishes JUMPED(true), NEXT_STEP.
/// On failure: clears jumping flag, publishes COORDINATE_FROM + STEADY_FOOTING_CONTEXT,
///             GoTos failure label.
///
/// If `!actingPlayer.isJumping()` or `!mechanic.canStillJump(game, actingPlayer)` → NEXT_STEP.
///
/// Init params: GOTO_LABEL_ON_FAILURE (mandatory), MOVE_START (optional).
///
/// Re-roll: TRR offered via ReRollOffer prompt on first failure.
/// (Skill re-roll for JUMP not yet in the reroll-property mapping.)
///
/// canStillJump (JumpMechanic) is wired; minimum roll uses player agility (BB2025: ag + modifiers).
/// DiceInterpreter.is_skill_roll_successful used for success check.
///
/// Skill re-roll for JUMP (Leap canRerollJump source) → wired via find_skill_reroll_source.
/// JumpModifierFactory wired: TACKLEZONE (max of from/to zones) and PREHENSILE_TAIL modifiers applied.
/// client-only: DivingTackle dialog (checkDivingTackle, usingDivingTackle) — client-side.
/// client-only: canChooseToIgnoreJumpModifierAfterRoll skill dialog — client-side.
/// client-only: canIgnoreJumpModifiers dialog (useIgnoreModifierSkill) — headless always applies modifiers.
/// handleFailure: COORDINATE_FROM(null if roll==1), updatePlayerAndBallPosition, updateMoveSquares — all implemented.
/// client-only: checkDivingTackle/usingDivingTackle dialog — headless auto-skips diving tackle activation.
/// fSecondGoForIt: handled in StepGoForIt (second_go_for_it field there triggers repeat for jumping players).
pub struct StepJump {
    /// Java: goToLabelOnFailure
    pub goto_label_on_failure: String,
    /// Java: moveStart
    pub move_start: Option<FieldCoordinate>,
    /// Java: roll
    pub roll: i32,
    /// Java: usingDivingTackle (Boolean tristate)
    pub using_diving_tackle: Option<bool>,
    /// Java: alreadyReported
    pub already_reported: bool,
    /// Java: useIgnoreModifierAfterRollSkill (Boolean tristate)
    pub use_ignore_modifier_after_roll_skill: Option<bool>,
    /// Java: useIgnoreModifierSkill
    pub use_ignore_modifier_skill: bool,
    /// Java: dtRerollAsked
    pub dt_reroll_asked: bool,
    /// Java: AbstractStepWithReRoll fields
    pub re_roll_state: ReRollState,
}

impl StepJump {
    pub fn new(goto_label_on_failure: String) -> Self {
        Self {
            goto_label_on_failure,
            move_start: None,
            roll: 0,
            using_diving_tackle: None,
            already_reported: false,
            use_ignore_modifier_after_roll_skill: None,
            use_ignore_modifier_skill: false,
            dt_reroll_asked: false,
            re_roll_state: ReRollState::new(),
        }
    }
}

impl Step for StepJump {
    fn id(&self) -> StepId { StepId::Jump }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if let Action::UseReRoll { use_reroll: false } = action {
            self.re_roll_state.re_roll_source = None;
        }
        // client-only: CLIENT_PLAYER_CHOICE → DIVING_TACKLE mode → usingDivingTackle / setDefenderId
        // client-only: CLIENT_USE_SKILL → canChooseToIgnoreJumpModifierAfterRoll → useIgnoreModifierAfterRollSkill
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(v) => { self.goto_label_on_failure = v.clone(); true }
            StepParameter::MoveStart(v) => { self.move_start = Some(*v); true }
            StepParameter::UsingDivingTackle(v) => { self.using_diving_tackle = Some(*v); true }
            _ => false,
        }
    }
}

impl StepJump {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: doLeap = actingPlayer.isJumping() && mechanic.canStillJump(game, actingPlayer)
        let mechanic = JumpMechanic::new();
        let do_leap = game.acting_player.jumping
            && mechanic.can_still_jump(game, &game.acting_player.clone());

        if !do_leap {
            return StepOutcome::next();
        }

        let already_rerolled = self.re_roll_state.re_rolled_action
            .as_ref().map(|a| a.name == "JUMP").unwrap_or(false);

        // Java: if (JUMP == reRolledAction && !useIgnoreModifierAfterRollSkill && usingDivingTackle==null)
        //         if (!dtRerollAsked && (source==null || !useReRoll(...))) { handleFailure; doLeap=false }
        if already_rerolled {
            let pid = game.acting_player.player_id.as_deref().unwrap_or("").to_owned();
            let source_opt = self.re_roll_state.re_roll_source.clone();
            let consumed = source_opt
                .as_ref()
                .map(|s| use_reroll(game, s, &pid))
                .unwrap_or(false);
            if !consumed {
                return self.handle_failure(game);
            }
            // Roll was reset to 0 when the re-roll offer was issued; fresh d6 below
        }

        if self.roll == 0 {
            self.roll = rng.d6();
        }

        let player_id = game.acting_player.player_id.clone();
        let agility = player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.agility_with_modifiers())
            .unwrap_or(3);

        // Java: JumpModifierFactory.findModifiers(JumpContext(game, player, moveStart, to)).
        // from = move_start (origin of leap); to = current player coordinate.
        let (modifier_total, modifier_names) = if let (Some(pid), Some(from)) = (player_id.as_deref(), self.move_start) {
            if let Some(player) = game.player(pid) {
                let to = game.field_model.player_coordinate(pid).unwrap_or(from);
                let ctx = JumpContext::new(game, player, from, to);
                let factory = JumpModifierFactory::for_rules(game.rules);
                let mods = factory.find_applicable(&ctx);
                let total: i32 = mods.iter().map(|m| m.get_modifier()).sum();
                let names: Vec<String> = mods.iter().map(|m| m.get_report_string().to_owned()).collect();
                (total, names)
            } else { (0, vec![]) }
        } else { (0, vec![]) };
        let minimum_roll = (agility + modifier_total).max(2);
        let successful = DiceInterpreter::is_skill_roll_successful(self.roll, minimum_roll);

        // Java: getResult().addReport(new ReportJumpRoll(playerId, successful, roll, minRoll, reRolled, modifiers))
        {
            use ffb_model::report::report_jump_roll::ReportJumpRoll;
            game.report_list.add(ReportJumpRoll::new(
                player_id.clone(),
                successful,
                self.roll,
                minimum_roll,
                already_rerolled,
                modifier_names,
            ));
        }

        if successful {
            game.acting_player.jumping = false;
            StepOutcome::next()
                .publish(StepParameter::Jumped(true))
        } else {
            // Try re-roll on first failure
            if !already_rerolled {
                use ffb_model::model::re_rolled_action::ReRolledAction;
                self.re_roll_state.re_rolled_action = Some(ReRolledAction::new("JUMP"));

                // Java: skillReRollSource = UtilCards.getUnusedRerollSource(actingPlayer, JUMP)
                let skill_source = find_skill_reroll_source(game, "JUMP");
                if let Some(source) = skill_source {
                    let pid = player_id.as_deref().unwrap_or("").to_owned();
                    use_reroll(game, &source, &pid);
                    self.re_roll_state.re_roll_source = Some(source);
                    self.roll = 0;
                    return self.execute_step(game, rng);
                }
                // TRR offer
                if let Some(prompt) = ask_for_reroll_if_available(game, "JUMP", minimum_roll, false) {
                    self.re_roll_state.re_roll_source = Some(ReRollSource::new("TRR"));
                    self.roll = 0; // reset so the re-roll gets a fresh d6
                    return StepOutcome::cont().with_prompt(prompt);
                }
            }

            self.handle_failure(game)
        }
    }

    fn handle_failure(&mut self, game: &mut Game) -> StepOutcome {
        // Java: actingPlayer.setJumping(false)
        game.acting_player.jumping = false;
        // Java: publishParameter(STEADY_FOOTING_CONTEXT, new SteadyFootingContext(new InjuryTypeDropJump(defender)))
        let ctx = SteadyFootingContext::from_injury_type_name("InjuryTypeDropJump".into());
        let label = self.goto_label_on_failure.clone();
        let coord_from = if self.roll > 1 {
            self.move_start
        } else {
            // Java: game.getFieldModel().updatePlayerAndBallPosition(actingPlayer.getPlayer(), moveStart)
            if let (Some(pid), Some(start)) = (
                game.acting_player.player_id.clone(),
                self.move_start,
            ) {
                let old_pos = game.field_model.player_coordinate(&pid);
                if !game.field_model.ball_moving {
                    if let (Some(old), Some(ball)) = (old_pos, game.field_model.ball_coordinate) {
                        if old == ball {
                            game.field_model.ball_coordinate = Some(start);
                        }
                    }
                }
                game.field_model.set_player_coordinate(&pid, start);
            }
            // Java: UtilServerPlayerMove.updateMoveSquares(getGameState(), actingPlayer.isJumping())
            UtilServerPlayerMove::update_move_squares(game, false);
            None
        };
        let mut out = StepOutcome::goto(&label)
            .publish(StepParameter::SteadyFootingContext(Box::new(ctx)));
        if let Some(c) = coord_from {
            out = out.publish(StepParameter::CoordinateFrom(c));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::{Rules, TurnMode};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::enums::SkillId;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    fn make_game_with_leaper() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2025);

        let mut player = Player::default();
        player.id = "p1".into();
        player.agility = 3;
        player.starting_skills.push(SkillWithValue::new(SkillId::Leap));
        game.team_home.players.push(player);
        game.field_model.set_player_coordinate("p1", ffb_model::types::FieldCoordinate::new(5, 5));
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.jumping = true;
        game
    }

    #[test]
    fn not_jumping_returns_next_step() {
        let mut game = make_game();
        game.acting_player.jumping = false;
        let mut step = StepJump::new("fail".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn jumping_but_cannot_still_jump_returns_next_step() {
        let mut game = make_game();
        game.acting_player.jumping = true;
        let mut step = StepJump::new("fail".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn success_clears_jumping_flag() {
        let mut game = make_game_with_leaper();
        let mut step = StepJump::new("fail".into());
        step.roll = 4; // ag=3, min_roll=3, 4 >= 3 → success
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.acting_player.jumping);
    }

    #[test]
    fn success_publishes_jumped_true() {
        let mut game = make_game_with_leaper();
        let mut step = StepJump::new("fail".into());
        step.roll = 4;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::Jumped(true))));
    }

    #[test]
    fn failure_goes_to_failure_label() {
        let mut game = make_game_with_leaper();
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        let mut step = StepJump::new("fail".into());
        step.roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
    }

    #[test]
    fn failure_roll_2_below_ag3_goes_to_label() {
        let mut game = make_game_with_leaper();
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        let mut step = StepJump::new("fail".into());
        step.roll = 2;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn failure_roll_gt_1_publishes_coordinate_from() {
        let mut game = make_game_with_leaper();
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        let start = FieldCoordinate::new(4, 4);
        let mut step = StepJump::new("fail".into());
        step.roll = 2;
        step.move_start = Some(start);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::CoordinateFrom(_))));
    }

    #[test]
    fn failure_roll_1_no_coordinate_from() {
        let mut game = make_game_with_leaper();
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        let mut step = StepJump::new("fail".into());
        step.roll = 1;
        step.move_start = Some(FieldCoordinate::new(4, 4));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(!out.published.iter().any(|p| matches!(p, StepParameter::CoordinateFrom(_))));
    }

    #[test]
    fn failure_with_trr_offers_reroll_prompt() {
        let mut game = make_game_with_leaper();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        let mut step = StepJump::new("fail".into());
        step.roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
        assert!(out.prompt.is_some());
    }

    #[test]
    fn decline_reroll_goes_to_failure_label() {
        let mut game = make_game_with_leaper();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        let mut step = StepJump::new("fail".into());
        step.roll = 1;
        let _offer = step.start(&mut game, &mut GameRng::new(0));
        let out = step.handle_command(&Action::UseReRoll { use_reroll: false }, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn accept_reroll_with_success_returns_next_step() {
        let mut game = make_game_with_leaper();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        let mut step = StepJump::new("fail".into());
        step.roll = 1; // first roll fails
        let _offer = step.start(&mut game, &mut GameRng::new(0));
        step.roll = 5; // success on re-roll (ag=3, min=3)
        let out = step.handle_command(&Action::UseReRoll { use_reroll: true }, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_move_start_accepted() {
        let mut step = StepJump::new("fail".into());
        let coord = FieldCoordinate::new(4, 4);
        assert!(step.set_parameter(&StepParameter::MoveStart(coord)));
        assert_eq!(step.move_start, Some(coord));
    }

    #[test]
    fn set_parameter_goto_label_on_failure_accepted() {
        let mut step = StepJump::new("old".into());
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("new".into())));
        assert_eq!(step.goto_label_on_failure, "new");
    }

    #[test]
    fn failure_publishes_steady_footing_context_drop_jump() {
        let mut game = make_game_with_leaper();
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        let mut step = StepJump::new("fail".into());
        step.roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::SteadyFootingContext(_))));
    }

    #[test]
    fn high_agility_lowers_minimum_roll() {
        let mut game = make_game();
        let mut player = Player::default();
        player.id = "p2".into();
        player.agility = 2; // 2+ target
        player.starting_skills.push(SkillWithValue::new(SkillId::Leap));
        game.team_home.players.push(player);
        game.field_model.set_player_coordinate("p2", ffb_model::types::FieldCoordinate::new(5, 5));
        game.acting_player.player_id = Some("p2".into());
        game.acting_player.jumping = true;

        let mut step = StepJump::new("fail".into());
        step.roll = 2;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep, "AG2 player should succeed on roll 2");
    }

    #[test]
    fn tackle_zone_at_destination_increases_minimum_roll() {
        // AG3 player jumping TO a square adjacent to an opponent with TZ.
        // Without TZ: min_roll = 3. With 1 TZ: min_roll = 4.
        // Roll of 3 should fail when there's a TZ opponent at the landing square.
        use ffb_model::enums::PS_STANDING;
        use ffb_model::types::FieldCoordinate;
        let mut game = make_game();

        let mut leaper = Player::default();
        leaper.id = "leaper".into();
        leaper.agility = 3;
        leaper.starting_skills.push(SkillWithValue::new(SkillId::Leap));
        game.team_home.players.push(leaper);

        // Place leaper at (5,5) — this is the "to" (current) coordinate.
        game.field_model.set_player_coordinate("leaper", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("leaper", ffb_model::enums::PlayerState::new(PS_STANDING));
        game.acting_player.player_id = Some("leaper".into());
        game.acting_player.jumping = true;

        // Opponent adjacent to (5,5) at (5,6) — creates a tackle zone at the "to" square.
        let mut opp = Player::default();
        opp.id = "opp".into();
        opp.agility = 3;
        game.team_away.players.push(opp);
        game.field_model.set_player_coordinate("opp", FieldCoordinate::new(5, 6));
        game.field_model.set_player_state("opp", ffb_model::enums::PlayerState::new(PS_STANDING));

        // move_start = (3,3) — the "from" coordinate (no adjacent opponents there).
        let mut step = StepJump::new("fail".into());
        step.move_start = Some(FieldCoordinate::new(3, 3));
        step.roll = 3; // Exactly meets ag=3 if no modifiers; fails with +1 from 1 TZ.
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;

        let out = step.start(&mut game, &mut GameRng::new(0));
        // With 1 tackle zone at "to": min_roll = 3 + 1 = 4, so roll 3 fails.
        assert_eq!(out.action, StepAction::GotoLabel,
            "roll=3 should fail when 1 TZ at landing square (min_roll=4)");
    }

    #[test]
    fn no_tackle_zones_roll_equals_agility_succeeds() {
        // AG3 player with no adjacent opponents: min_roll = 3, roll=3 → success.
        use ffb_model::types::FieldCoordinate;
        let mut game = make_game_with_leaper(); // p1 at (5,5), ag=3, no opponents

        let mut step = StepJump::new("fail".into());
        step.move_start = Some(FieldCoordinate::new(3, 3));
        step.roll = 3;

        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep,
            "roll=3 should succeed when no TZ (min_roll=3)");
    }
}
