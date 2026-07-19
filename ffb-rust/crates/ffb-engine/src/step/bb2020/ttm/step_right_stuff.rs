/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2020.ttm.StepRightStuff`.
///
/// Step in TTM sequence to handle skill RIGHT_STUFF (landing roll).
/// - If player state is FALLING (thrown out of bounds) or on a box coordinate:
///   skip landing roll, publish END_TURN + THROWN_PLAYER_COORDINATE(null).
/// - Restore player state to `old_player_state` (BB2020 change) and sync ball.
/// - If FUMBLE + kickedPlayer → fumbled-KTM injury (no landing roll).
/// - If drop_thrown_player == false: roll landing (minimumRollRightStuff + modifiers).
///   - Success + has ball → touchdown check.
///   - Success without ball on ball square → SCATTER_BALL.
///   - Failure → re-roll if available.
/// - If roll fails or drop_thrown_player: TTMLanding / FumbledKtm injury.
///
/// BB2020 differences vs BB2016:
///  - Restores player state via `old_player_state` (OLD_DEFENDER_STATE), not thrownPlayerState.
///  - Publishes THROWN_PLAYER_STATE (oldPlayerState) after restoring.
///  - Adds `passResult` and `kickedPlayer` fields.
///  - Adds `goToOnSuccess` label (GOTO_LABEL_ON_SUCCESS).
///  - On fumbled KTM → InjuryTypeFumbledKtm instead of InjuryTypeTTMLanding.
///  - Uses `playerCoordinate.isBoxCoordinate()` guard (trapdoor).
///
/// RightStuffModifierFactory + AgilityMechanic.minimumRollRightStuff → wired.
/// UtilServerInjury.handleInjury wired for both TTMLanding and FumbledKtm paths.
/// SppMechanic.addCompletion wired for accurate TTM pass (non-kicked player).
/// `isBoxCoordinate()` guard wired — skips landing roll when player is in the dugout.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::enums::{PlayerState, PassOutcome as ModelPassResult, PS_FALLING, ApothecaryMode, ReRollSource};
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::report::report_right_stuff_roll::ReportRightStuffRoll;
use ffb_model::report::report_skill_use::ReportSkillUse;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter, CatchScatterThrowInMode};
use ffb_mechanics::modifiers::right_stuff_modifier_factory::RightStuffModifierFactory;
use ffb_mechanics::modifiers::right_stuff_context::RightStuffContext;
use ffb_mechanics::pass_result::PassResult as MechanicPassResult;
use crate::dice_interpreter::DiceInterpreter;
use crate::injury::injuryType::injury_type_ttm_landing::InjuryTypeTTMLanding;
use crate::injury::injuryType::injury_type_fumbled_ktm::InjuryTypeFumbledKtm;
use crate::step::util_server_injury;
use crate::step::util_server_steps::check_touchdown;
use crate::step::abstract_step_with_re_roll::{ReRollState, find_skill_reroll_source};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

/// Java: `StepRightStuff` (bb2020/ttm).
pub struct StepRightStuff {
    /// Java: `fThrownPlayerHasBall`
    thrown_player_has_ball: Option<bool>,
    /// Java: `fThrownPlayerId`
    thrown_player_id: Option<String>,
    /// Java: `fDropThrownPlayer`
    drop_thrown_player: bool,
    /// Java: `passResult` — BB2020 addition.
    pass_result: Option<ModelPassResult>,
    /// Java: `kickedPlayer` — BB2020 addition.
    kicked_player: bool,
    /// Java: `goToOnSuccess` — BB2020 addition.
    goto_on_success: Option<String>,
    /// Java: `oldPlayerState` (OLD_DEFENDER_STATE) — BB2020 addition.
    old_player_state: Option<PlayerState>,
    /// Java: AbstractStepWithReRoll fields
    re_roll_state: ReRollState,
    /// Cached roll value (0 = not yet rolled).
    roll: i32,
}

impl StepRightStuff {
    pub fn new() -> Self {
        Self {
            thrown_player_has_ball: None,
            thrown_player_id: None,
            drop_thrown_player: false,
            pass_result: None,
            kicked_player: false,
            goto_on_success: None,
            old_player_state: None,
            re_roll_state: ReRollState::new(),
            roll: 0,
        }
    }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let player_id = match &self.thrown_player_id {
            Some(id) => id.clone(),
            None     => return StepOutcome::next(),
        };
        let has_ball = self.thrown_player_has_ball.unwrap_or(false);
        let player_coord = game.field_model.player_coordinate(&player_id);

        // BB2020: skip landing roll when FALLING or on a box coordinate.
        let is_falling = game.field_model.player_state(&player_id)
            .map(|s| s.base() == PS_FALLING)
            .unwrap_or(false);
        let is_box_coord = player_coord.map(|c| c.is_box_coordinate()).unwrap_or(false);

        if is_falling || is_box_coord {
            return StepOutcome::next()
                .publish(StepParameter::EndTurn(has_ball))
                .publish(StepParameter::ThrownPlayerCoordinate(None));
        }

        // BB2020: restore player state to old_player_state before the roll.
        if let Some(old) = self.old_player_state {
            game.field_model.set_player_state(&player_id, old);
        }
        // Publish restored state so downstream steps see the right value.
        let out_state = self.old_player_state.unwrap_or_default();

        // Sync ball to player coordinate when holding ball.
        if has_ball {
            if let Some(coord) = player_coord {
                game.field_model.ball_coordinate = Some(coord);
            }
        }

        // BB2020: fumbled KTM path.
        let fumbled_ktm = self.pass_result == Some(ModelPassResult::Fumble) && self.kicked_player;

        let already_rerolled = self.re_roll_state.re_rolled_action
            .as_ref().map(|a| a.name == "RIGHT_STUFF").unwrap_or(false);
        let mut do_roll = !self.drop_thrown_player && !fumbled_ktm;

        // Java: if (reRolledAction == RIGHT_STUFF) { if (source == null || !useReRoll) doRoll = false; }
        if do_roll && already_rerolled {
            let source_opt = self.re_roll_state.re_roll_source.clone();
            let consumed = source_opt.as_ref().map(|s| use_reroll(game, s, &player_id)).unwrap_or(false);
            if !consumed {
                do_roll = false;
            }
        }

        if do_roll {
            let minimum_roll = if let Some(player) = game.player(&player_id) {
                let factory = RightStuffModifierFactory::for_rules(game.rules);
                let mechanic_pass_result = self.pass_result.map(|r| match r {
                    ModelPassResult::Fumble | ModelPassResult::MissedCatch => MechanicPassResult::FUMBLE,
                    ModelPassResult::Inaccurate => MechanicPassResult::INACCURATE,
                    ModelPassResult::WildlyInaccurate => MechanicPassResult::WILDLY_INACCURATE,
                    _ => MechanicPassResult::ACCURATE,
                });
                let ctx = RightStuffContext::new_full(game, player, mechanic_pass_result, None);
                let mods = factory.find_applicable(&ctx);
                RightStuffModifierFactory::minimum_roll(player.agility_with_modifiers(), &mods)
            } else {
                4
            };
            if self.roll == 0 {
                self.roll = rng.d6();
            }
            let mut successful = DiceInterpreter::is_skill_roll_successful(self.roll, minimum_roll);
            let re_rolled = already_rerolled && self.re_roll_state.re_roll_source.is_some();

            // Java: if (PassOutcome.FUMBLE == passResult && thrower.hasSkillProperty(fumbledPlayerLandsSafely))
            //         { successful = true; addReport(ReportSkillUse(..., FUMBLED_PLAYER_LANDS_SAFELY)); }
            //       else { addReport(new ReportRightStuffRoll(...)); }
            // NOTE: Java's `hasSkillProperty`/`getSkillWithProperty` ignore the skill's used
            // state entirely (unlike ActingPlayer's getUnusedSkillWithProperty). Using the
            // "unused" variant here would incorrectly skip the safe-landing effect if the
            // thrower's skill happened to be marked used elsewhere this turn.
            let fumble_lands_safely_skill = if self.pass_result == Some(ModelPassResult::Fumble) {
                game.thrower_id.as_deref()
                    .and_then(|id| game.player(id))
                    .and_then(|p| p.skill_id_with_property(NamedProperties::FUMBLED_PLAYER_LANDS_SAFELY))
            } else {
                None
            };
            if let Some(skill_id) = fumble_lands_safely_skill {
                successful = true;
                let thrower_id = game.thrower_id.clone();
                game.report_list.add(ReportSkillUse::new(
                    thrower_id,
                    skill_id,
                    true,
                    SkillUse::FUMBLED_PLAYER_LANDS_SAFELY,
                ));
            } else {
                game.report_list.add(ReportRightStuffRoll::new(
                    Some(player_id.clone()),
                    successful,
                    self.roll,
                    minimum_roll,
                    re_rolled,
                    vec![],
                ));
            }

            if successful {
                // Java: if (passResult == ACCURATE && !kickedPlayer && thrower != null)
                //   spp.addCompletion(additionalCompletionSppTeams, playerResult(thrower))
                if self.pass_result == Some(ModelPassResult::Complete) && !self.kicked_player {
                    if let Some(ref thrower_id) = game.thrower_id.clone() {
                        let is_home = game.team_home.has_player(thrower_id);
                        let team_result = if is_home {
                            &mut game.game_result.home
                        } else {
                            &mut game.game_result.away
                        };
                        team_result.player_result_mut(thrower_id).completions += 1;
                    }
                }
                let success_label = self.goto_on_success.as_deref().unwrap_or("");
                let mut out = StepOutcome::goto(success_label)
                    .publish(StepParameter::ThrownPlayerState(out_state))
                    .publish(StepParameter::ThrownPlayerCoordinate(None));
                if has_ball {
                    if check_touchdown(game) {
                        out = out.publish(StepParameter::EndTurn(true));
                    }
                } else {
                    let ball_coord = game.field_model.ball_coordinate;
                    if player_coord.is_some() && player_coord == ball_coord {
                        game.field_model.ball_moving = true;
                        out = out.publish(StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ScatterBall));
                    }
                }
                return out;
            }

            // Failure: offer re-roll if not yet re-rolled
            if !already_rerolled {
                use ffb_model::model::re_rolled_action::ReRolledAction;
                self.re_roll_state.re_rolled_action = Some(ReRolledAction::new("RIGHT_STUFF"));

                let skill_source = find_skill_reroll_source(game, "RIGHT_STUFF");
                if let Some(source) = skill_source {
                    use_reroll(game, &source, &player_id);
                    self.re_roll_state.re_roll_source = Some(source);
                    self.roll = 0;
                    return self.execute_step(game, rng);
                }

                if let Some(prompt) = ask_for_reroll_if_available(game, "RIGHT_STUFF", minimum_roll, false) {
                    self.re_roll_state.re_roll_source = Some(ReRollSource::new("TRR"));
                    self.roll = 0;
                    return StepOutcome::cont().with_prompt(prompt);
                }
            }
            // Failed roll, no re-roll available: fall through to drop path.
        }

        // Drop path (drop_thrown_player == true OR fumbled_ktm OR failed roll).
        // Java: UtilServerInjury.handleInjury(fumbledKtm ? FumbledKtm : TTMLanding,
        //         game.getActingPlayer().getPlayer(), thrownPlayer, playerCoordinate, null, null,
        //         ApothecaryMode.THROWN_PLAYER);
        // NOTE: the ApothecaryMode is always THROWN_PLAYER regardless of fumbledKtm — only the
        // injury type varies. The attacker is the acting player (thrower), not null.
        let coord = game.field_model.player_coordinate(&player_id)
            .unwrap_or(ffb_model::types::FieldCoordinate::new(0, 0));
        let attacker_id = game.acting_player.player_id.clone();
        let ir = if fumbled_ktm {
            let mut injury_type = InjuryTypeFumbledKtm::new();
            util_server_injury::handle_injury(
                game, rng, &mut injury_type,
                attacker_id.as_deref(), &player_id, coord, None, None,
                ApothecaryMode::ThrownPlayer,
            )
        } else {
            let mut injury_type = InjuryTypeTTMLanding::new();
            util_server_injury::handle_injury(
                game, rng, &mut injury_type,
                attacker_id.as_deref(), &player_id, coord, None, None,
                ApothecaryMode::ThrownPlayer,
            )
        };
        ir.apply_to(game);
        // Java: publishParameter(INJURY_RESULT, injuryResultThrownPlayer)
        let mut out = StepOutcome::next()
            .publish(StepParameter::ThrownPlayerState(out_state))
            .publish(StepParameter::InjuryResult(Box::new(ir)))
            .publish(StepParameter::ThrownPlayerCoordinate(None));

        // Java: StepParameterSet params = UtilServerInjury.dropPlayer(this, thrownPlayer, ApothecaryMode.THROWN_PLAYER);
        //       if (!fThrownPlayerHasBall) params.remove(END_TURN);
        //       publishParameters(params);
        //       if (fThrownPlayerHasBall) publishParameter(END_TURN, true);
        let mut drop_params = util_server_injury::drop_player_no_sph(game, &player_id);
        if !has_ball {
            drop_params.retain(|p| !matches!(p, StepParameter::EndTurn(_)));
        }
        for p in drop_params {
            out = out.publish(p);
        }
        if has_ball {
            out = out.publish(StepParameter::EndTurn(true));
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

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::UseReRoll { use_reroll: true } => {
                self.execute_step(game, rng)
            }
            Action::UseReRoll { use_reroll: false } => {
                self.re_roll_state.re_roll_source = None;
                self.execute_step(game, rng)
            }
            _ => self.execute_step(game, rng),
        }
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::ThrownPlayerHasBall(v) => { self.thrown_player_has_ball = Some(*v); true }
            StepParameter::ThrownPlayerId(v)      => { self.thrown_player_id = v.clone(); true }
            StepParameter::DropThrownPlayer(v)    => { self.drop_thrown_player = *v; true }
            StepParameter::PassResultParam(v)     => { self.pass_result = Some(*v); true }
            StepParameter::IsKickedPlayer(v)      => { self.kicked_player = *v; true }
            StepParameter::GotoLabelOnSuccess(s)  => { self.goto_on_success = Some(s.clone()); true }
            StepParameter::OldDefenderState(v)    => { self.old_player_state = Some(*v); true }
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
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
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
    fn set_parameter_pass_result() {
        let mut step = StepRightStuff::new();
        assert!(step.set_parameter(&StepParameter::PassResultParam(ModelPassResult::Fumble)));
        assert_eq!(step.pass_result, Some(ModelPassResult::Fumble));
    }

    #[test]
    fn set_parameter_old_defender_state() {
        use ffb_model::enums::{PlayerState, PS_STANDING};
        let mut step = StepRightStuff::new();
        let state = PlayerState::new(PS_STANDING);
        assert!(step.set_parameter(&StepParameter::OldDefenderState(state)));
        assert_eq!(step.old_player_state, Some(state));
    }

    #[test]
    fn fumbled_ktm_drops_player_without_roll() {
        let mut game = make_game();
        let mut step = StepRightStuff::new();
        step.thrown_player_id = Some("p1".into());
        step.pass_result = Some(ModelPassResult::Fumble);
        step.kicked_player = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Fumbled KTM → no landing roll → drop path → NEXT_STEP.
        assert!(matches!(out.action, StepAction::NextStep));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::ThrownPlayerCoordinate(None))));
    }

    // Java: `!doRoll` branch always calls
    //   `publishParameter(INJURY_RESULT, ...)` and
    //   `UtilServerInjury.dropPlayer(this, thrownPlayer, ApothecaryMode.THROWN_PLAYER)`,
    // always with `ApothecaryMode.THROWN_PLAYER` (fumbledKtm only changes the injury *type*,
    // not the apothecary mode) and the acting player as attacker (never null). The Rust code
    // previously omitted the INJURY_RESULT publish and the dropPlayer() call entirely, and
    // used `ApothecaryMode::Defender` + no attacker for the fumbled-KTM branch.
    #[test]
    fn drop_path_publishes_injury_result_with_thrown_player_apothecary_mode_and_attacker() {
        use std::collections::HashSet;
        use ffb_model::enums::{PlayerGender, PlayerType};
        use ffb_model::model::player::Player;

        let mut game = make_game();
        let thrower = Player {
            id: "thrower".into(), name: "thrower".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };
        game.team_home.players.push(thrower);
        game.acting_player.player_id = Some("thrower".into());

        let p1 = Player {
            id: "p1".into(), name: "p1".into(), nr: 2, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };
        game.team_home.players.push(p1);
        game.field_model.set_player_coordinate("p1", ffb_model::types::FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("p1", ffb_model::enums::PlayerState::new(ffb_model::enums::PS_STANDING));

        let mut step = StepRightStuff::new();
        step.thrown_player_id = Some("p1".into());
        step.pass_result = Some(ModelPassResult::Fumble);
        step.kicked_player = true; // fumbled_ktm branch
        let out = step.start(&mut game, &mut GameRng::new(0));

        let injury_result = out.published.iter().find_map(|p| match p {
            StepParameter::InjuryResult(ir) => Some(ir.clone()),
            _ => None,
        });
        let ir = injury_result.expect("Java always publishes INJURY_RESULT in the !doRoll branch");
        assert_eq!(ir.injury_context().apothecary_mode, ApothecaryMode::ThrownPlayer,
            "ApothecaryMode is always THROWN_PLAYER regardless of fumbledKtm");
        assert_eq!(ir.injury_context().attacker_id.as_deref(), Some("thrower"),
            "attacker is the acting player, not null");
    }

    #[test]
    fn report_right_stuff_roll_added_on_normal_roll() {
        // Java: StepRightStuff (BB2020) adds ReportRightStuffRoll in the normal (non-fumble) path.
        use std::collections::HashSet;
        use ffb_model::enums::{PlayerGender, PlayerType, SkillId};
        use ffb_model::model::player::Player;
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        let p = Player {
            id: "p1".into(), name: "p1".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 6, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };
        game.team_home.players.push(p);
        let mut step = StepRightStuff::new();
        step.thrown_player_id = Some("p1".into());
        // Not a fumble, so normal roll path → ReportRightStuffRoll
        let _out = step.start(&mut game, &mut GameRng::new(5));
        assert!(game.report_list.has_report(ReportId::RIGHT_STUFF_ROLL),
            "BB2020 normal roll path must add ReportRightStuffRoll");
    }

    #[test]
    fn report_right_stuff_roll_added_on_failure_roll() {
        // Java: ReportRightStuffRoll is also added on failure rolls (else branch).
        use std::collections::HashSet;
        use ffb_model::enums::{PlayerGender, PlayerType, SkillId};
        use ffb_model::model::player::Player;
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        let p = Player {
            id: "p1".into(), name: "p1".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 1, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };
        game.team_home.players.push(p);
        let mut step = StepRightStuff::new();
        step.thrown_player_id = Some("p1".into());
        step.roll = 1; // Force failure: agility 1 needs 6+, roll 1 fails
        let _out = step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::RIGHT_STUFF_ROLL),
            "BB2020 failure roll path must also add ReportRightStuffRoll");
    }

    #[test]
    fn minimum_roll_uses_agility_with_modifiers_not_raw_agility() {
        // Java: AgilityMechanic.minimumRollRightStuff(Player, modifiers) uses
        // player.getAgilityWithModifiers(), not the raw AG stat. A temporary AG
        // penalty (e.g. from a spell/skill effect) must lower the effective
        // agility used for the Right Stuff minimum-roll calculation.
        use std::collections::HashSet;
        use ffb_model::enums::{PlayerGender, PlayerType};
        use ffb_model::model::player::{Player, STAT_AG};
        use crate::step::framework::StepAction;
        let mut game = make_game();
        let mut p = Player {
            id: "p1".into(), name: "p1".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 6, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };
        // Raw agility 6 → minimum roll 6 (fails on a roll of 5).
        // Effective agility (with -2 penalty) is 4 → minimum roll 4 (succeeds on a roll of 5).
        p.add_temporary_stat_mod("test", STAT_AG, -2);
        game.team_home.players.push(p);
        let mut step = StepRightStuff::new();
        step.thrown_player_id = Some("p1".into());
        step.roll = 5;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel,
            "roll of 5 must succeed against the agility-with-modifiers threshold of 4, \
             not fail against the raw-agility threshold of 6");
    }

}
