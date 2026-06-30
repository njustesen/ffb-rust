use ffb_model::types::FieldCoordinate;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::enums::ReRollSource;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::drop_player_context::SteadyFootingContext;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::abstract_step_with_re_roll::{ReRollState, find_skill_reroll_source};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};
use ffb_mechanics::modifiers::go_for_it_modifier_factory::GoForItModifierFactory;
use ffb_mechanics::modifiers::go_for_it_context::GoForItContext;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.move.StepGoForIt.
///
/// Resolves a Go-For-It (rush): roll d6, minimum 2 (with modifiers); on failure
/// publishes END_TURN + STEADY_FOOTING_CONTEXT and GoTos failure label.
///
/// Init params: GOTO_LABEL_ON_FAILURE (mandatory), BALL_AND_CHAIN_GFI (optional).
/// Sets: END_TURN, STEADY_FOOTING_CONTEXT (InjuryTypeDropGFI) for all stack steps on failure.
///
/// Re-roll order (mirroring Java AbstractStepWithReRoll):
///   1. Skill re-roll (e.g. Sprint / GoForIt — property canMakeAnExtraGfi) — auto-used
///   2. Team Re-Roll token (TRR) — offered via ReRollOffer prompt
///
/// TODO: modifier-ignoring skill dialog (canChooseToIgnoreRushModifierAfterRoll) not yet ported.
/// TODO: succeedGfi second-GFI-for-jumping push not yet ported (fSecondGoForIt).
/// TODO: fBallandChainGfi → actingPlayer.setFellFromRush(true) not yet ported.
/// TODO: (BLITZ action) blitzUsed flag, currentMove increment not yet ported.
pub struct StepGoForIt {
    /// Java: fGotoLabelOnFailure
    pub goto_label_on_failure: String,
    /// Java: fBallandChainGfi
    pub ball_and_chain_gfi: bool,
    /// Java: fSecondGoForIt
    pub second_go_for_it: bool,
    /// Java: moveStart (set via setParameter)
    pub move_start: Option<FieldCoordinate>,
    /// Java: usingModifierIgnoringSkill (Boolean tristate)
    pub using_modifier_ignoring_skill: Option<bool>,
    /// Java: roll
    pub roll: i32,
    /// Java: AbstractStepWithReRoll fields (fReRolledAction, fReRollSource, playerIdForSingleUseReRoll)
    pub re_roll_state: ReRollState,
}

impl StepGoForIt {
    pub fn new(goto_label_on_failure: String) -> Self {
        Self {
            goto_label_on_failure,
            ball_and_chain_gfi: false,
            second_go_for_it: false,
            move_start: None,
            using_modifier_ignoring_skill: None,
            roll: 0,
            re_roll_state: ReRollState::new(),
        }
    }
}

impl Step for StepGoForIt {
    fn id(&self) -> StepId { StepId::GoForIt }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::UseReRoll { use_reroll: true } => {
                // Agent accepted re-roll offer — re_roll_source was stored when we issued the prompt
                self.execute_step(game, rng)
            }
            Action::UseReRoll { use_reroll: false } => {
                // Agent declined — clear source so execute_step sees None → failGfi
                self.re_roll_state.re_roll_source = None;
                self.execute_step(game, rng)
            }
            _ => self.execute_step(game, rng),
        }
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(v) => { self.goto_label_on_failure = v.clone(); true }
            StepParameter::BallAndChainGfi(v) => { self.ball_and_chain_gfi = *v; true }
            StepParameter::MoveStart(v) => { self.move_start = Some(*v); true }
            _ => false,
        }
    }
}

impl StepGoForIt {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let player_id = game.acting_player.player_id.clone();
        let go_for_it_after_block = player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.has_skill_property(NamedProperties::GO_FOR_IT_AFTER_BLOCK))
            .unwrap_or(false);
        let run_gfi = go_for_it_after_block == self.ball_and_chain_gfi;

        if !run_gfi {
            return StepOutcome::next();
        }

        let going_for_it = game.acting_player.goes_for_it;
        let current_move = game.acting_player.current_move;
        let ma = player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.movement as i32)
            .unwrap_or(4);

        if !going_for_it || current_move <= ma {
            return StepOutcome::next();
        }

        // Java: if (ReRolledActions.RUSH == getReRolledAction() && !usingModifierIgnoringSkill) {
        //         if (getReRollSource() == null || !useReRoll(...)) { failGfi(); return; }
        //       }
        let already_rerolled = self.re_roll_state.re_rolled_action
            .as_ref().map(|a| a.name == "GFI").unwrap_or(false);
        let using_modifier_ignoring = self.using_modifier_ignoring_skill == Some(true);

        if already_rerolled && !using_modifier_ignoring {
            let pid = player_id.as_deref().unwrap_or("");
            let source_opt = self.re_roll_state.re_roll_source.clone();
            let consumed = source_opt
                .as_ref()
                .map(|s| use_reroll(game, s, pid))
                .unwrap_or(false);
            if !consumed {
                return self.fail_gfi(game);
            }
            // Roll was reset to 0 when the re-roll offer was issued; a fresh d6 is rolled in rush()
        }

        self.rush(game, rng)
    }

    fn rush(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Roll only on first call or after a skill re-roll that resets self.roll
        if self.roll == 0 {
            self.roll = rng.d6();
        }

        let player_id = game.acting_player.player_id.clone();
        let minimum_roll = {
            let factory = GoForItModifierFactory::for_rules(game.rules);
            if let Some(pid) = player_id.as_deref() {
                if let Some(player) = game.player(pid) {
                    let ctx = GoForItContext::new(game, player);
                    let mods = factory.find_applicable(&ctx);
                    GoForItModifierFactory::minimum_roll_going_for_it(&mods)
                } else {
                    2
                }
            } else {
                2
            }
        };

        let successful = self.roll >= minimum_roll;

        if successful {
            // Java: succeedGfi → push second GFI if jumping with surplus move, else NEXT_STEP
            // TODO: fSecondGoForIt / jumping second-GFI push not yet ported
            return StepOutcome::next();
        }

        // Failure path — attempt re-roll if this is the first failure
        let already_rerolled = self.re_roll_state.re_rolled_action
            .as_ref().map(|a| a.name == "GFI").unwrap_or(false);

        if !already_rerolled {
            use ffb_model::model::re_rolled_action::ReRolledAction;
            self.re_roll_state.re_rolled_action = Some(ReRolledAction::new("GFI"));

            // Java: findSkillReRollSource(ReRolledActions.RUSH) — auto-use skill re-roll if found
            let skill_source = find_skill_reroll_source(game, "GFI");
            if let Some(source) = skill_source {
                let pid = player_id.as_deref().unwrap_or("").to_owned();
                use_reroll(game, &source, &pid);
                self.re_roll_state.re_roll_source = Some(source);
                self.using_modifier_ignoring_skill = None;
                self.roll = 0; // fresh roll for the re-roll
                return self.rush(game, rng);
            }

            // No skill re-roll — offer TRR
            if let Some(prompt) = ask_for_reroll_if_available(game, "GFI", minimum_roll, false) {
                self.re_roll_state.re_roll_source = Some(ReRollSource::new("TRR"));
                self.roll = 0; // reset so the re-roll gets a fresh d6
                return StepOutcome::cont().with_prompt(prompt);
            }
        }

        self.fail_gfi(game)
    }

    fn fail_gfi(&mut self, game: &mut Game) -> StepOutcome {
        // Java: if actingPlayer.isJumping() → publishParameter(COORDINATE_FROM, null) + updatePos
        // TODO: jumping path not yet ported

        if self.ball_and_chain_gfi {
            game.acting_player.fell_from_rush = true;
        }
        // Java: publishParameter(STEADY_FOOTING_CONTEXT, new SteadyFootingContext(new InjuryTypeDropGFI()))
        // InjuryTypeName variant — no injury roll at this step; SteadyFooting performs the roll
        let ctx = SteadyFootingContext::from_injury_type_name("InjuryTypeDropGFI".into());
        let label = self.goto_label_on_failure.clone();
        StepOutcome::goto(&label)
            .publish(StepParameter::EndTurn(true))
            .publish(StepParameter::SteadyFootingContext(Box::new(ctx)))
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
    use ffb_model::enums::{SkillId, PlayerType, PlayerGender};
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;
    use std::collections::HashSet;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    fn make_gfi_game() -> Game {
        let mut game = make_game();
        game.acting_player.goes_for_it = true;
        game.acting_player.current_move = 10; // exceeds any MA (player_id=None → ma=4)
        game
    }

    fn add_player(game: &mut Game, id: &str) {
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 4, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
        });
        game.field_model.set_player_coordinate(id, FieldCoordinate::new(5, 5));
    }

    fn add_player_with_skill(game: &mut Game, id: &str, skill: SkillId) {
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 4, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![SkillWithValue { skill_id: skill, value: None }],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
        });
        game.field_model.set_player_coordinate(id, FieldCoordinate::new(5, 5));
    }

    #[test]
    fn success_on_roll_two_or_above_returns_next_step() {
        let mut game = make_gfi_game();
        let mut step = StepGoForIt::new("fail".into());
        step.roll = 2;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn failure_on_roll_one_goes_to_failure_label() {
        let mut game = make_gfi_game();
        let mut step = StepGoForIt::new("fail".into());
        step.roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
    }

    #[test]
    fn failure_publishes_end_turn() {
        let mut game = make_gfi_game();
        let mut step = StepGoForIt::new("fail".into());
        step.roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    #[test]
    fn ball_and_chain_gfi_skips_gfi_check() {
        let mut game = make_game();
        let mut step = StepGoForIt::new("fail".into());
        step.ball_and_chain_gfi = true;
        step.roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_goto_label_on_failure_accepted() {
        let mut step = StepGoForIt::new("old".into());
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("new".into())));
        assert_eq!(step.goto_label_on_failure, "new");
    }

    #[test]
    fn blizzard_weather_raises_minimum_roll() {
        use ffb_model::enums::Weather;
        let mut game = make_game();
        game.field_model.weather = Weather::Blizzard;
        game.acting_player.goes_for_it = true;
        game.acting_player.current_move = 10;
        let mut step = StepGoForIt::new("fail".into());
        step.roll = 2;
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Without a real player, modifier lookup falls back → minimum=2, roll=2 → success
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn failure_without_reroll_goes_to_label() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 0; // no TRR
        add_player(&mut game, "p1");
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.goes_for_it = true;
        game.acting_player.current_move = 5; // > MA(4) → GFI path
        let mut step = StepGoForIt::new("fail".into());
        step.roll = 1; // guaranteed fail
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn failure_with_trr_offers_reroll_prompt() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 1; // TRR available
        add_player(&mut game, "p1");
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.goes_for_it = true;
        game.acting_player.current_move = 5; // > MA(4)
        let mut step = StepGoForIt::new("fail".into());
        step.roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Should offer re-roll (Continue + prompt)
        assert_eq!(out.action, StepAction::Continue);
        assert!(out.prompt.is_some());
    }

    #[test]
    fn accept_reroll_then_success_returns_next_step() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        add_player(&mut game, "p1");
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.goes_for_it = true;
        game.acting_player.current_move = 5;
        let mut step = StepGoForIt::new("fail".into());
        step.roll = 1; // first roll fails
        let _offer = step.start(&mut game, &mut GameRng::new(0));
        // Simulate agent accepting, next roll will succeed
        step.roll = 4; // pre-set so rush() uses this on re-roll
        let out = step.handle_command(&Action::UseReRoll { use_reroll: true }, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn decline_reroll_goes_to_failure_label() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        add_player(&mut game, "p1");
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.goes_for_it = true;
        game.acting_player.current_move = 5;
        let mut step = StepGoForIt::new("fail".into());
        step.roll = 1;
        let _offer = step.start(&mut game, &mut GameRng::new(0));
        let out = step.handle_command(&Action::UseReRoll { use_reroll: false }, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
    }
}
