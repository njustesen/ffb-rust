use ffb_model::enums::{BlockResult, PS_FALLING, PlayerState, SkillId};
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::util::util_server_pushback::UtilServerPushback;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2020.block.StepBlockChoice.
/// Routes block die result to dodge/juggernaut/pushback sequence labels.
pub struct StepBlockChoice {
    pub goto_label_on_dodge: String,
    pub goto_label_on_juggernaut: String,
    pub goto_label_on_pushback: String,
    pub nr_of_dice: i32,
    pub block_roll: Vec<i32>,
    pub dice_index: usize,
    pub block_result: Option<BlockResult>,
    pub old_defender_state: Option<PlayerState>,
    pub suppress_extra_effect_handling: bool,
    pub show_name_in_report: bool,
    pub block_roll_id: i32,
}

impl StepBlockChoice {
    pub fn new(goto_label_on_dodge: String, goto_label_on_juggernaut: String, goto_label_on_pushback: String) -> Self {
        Self {
            goto_label_on_dodge,
            goto_label_on_juggernaut,
            goto_label_on_pushback,
            nr_of_dice: 0,
            block_roll: Vec::new(),
            dice_index: 0,
            block_result: None,
            old_defender_state: None,
            suppress_extra_effect_handling: false,
            show_name_in_report: false,
            block_roll_id: 0,
        }
    }
}

impl Step for StepBlockChoice {
    fn id(&self) -> StepId { StepId::BlockChoice }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if let Action::BlockChoice { die_index, .. } = action {
            self.dice_index = *die_index;
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::BlockResult(v) => { self.block_result = Some(*v); true }
            StepParameter::BlockRoll(v) => { self.block_roll = v.clone(); true }
            StepParameter::DiceIndex(v) => { self.dice_index = *v; true }
            StepParameter::NrOfDice(v) => { self.nr_of_dice = *v; true }
            StepParameter::OldDefenderState(v) => { self.old_defender_state = Some(*v); true }
            _ => false,
        }
    }
}

impl StepBlockChoice {
    fn execute_step(&self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        let block_result = match self.block_result {
            Some(r) => r,
            None => return StepOutcome::cont(),
        };
        let old_defender_state = self.old_defender_state.unwrap_or_default();
        let dodge_label = self.goto_label_on_dodge.clone();
        let juggernaut_label = self.goto_label_on_juggernaut.clone();
        let pushback_label = self.goto_label_on_pushback.clone();

        let outcome = match block_result {
            BlockResult::Skull => {
                // Restore defender to old state first; set attacker to FALLING
                if let Some(defender_id) = game.defender_id.clone() {
                    game.field_model.set_player_state(&defender_id, old_defender_state);
                }
                if let Some(attacker_id) = game.acting_player.player_id.clone() {
                    let attacker_state = game.field_model.player_state(&attacker_id).unwrap_or_default();
                    game.field_model.set_player_state(&attacker_id, attacker_state.change_base(PS_FALLING));
                }
                StepOutcome::next()
            }
            BlockResult::BothDown => {
                StepOutcome::goto(&juggernaut_label)
            }
            BlockResult::PowPushback => {
                // Java: check if defender has Dodge (ignoreDefenderStumblesResult).
                // If Tackle on attacker cancels it, fall + pushback. Otherwise goto dodge.
                let defender_id = game.defender_id.clone();
                let acting_player_id = game.acting_player.player_id.clone();
                let defender_has_dodge = defender_id.as_deref()
                    .and_then(|id| game.player(id))
                    .map(|p| p.has_skill_property(NamedProperties::IGNORE_DEFENDER_STUMBLES_RESULT))
                    .unwrap_or(false);
                if defender_has_dodge {
                    let attacker_has_tackle = acting_player_id.as_deref()
                        .and_then(|id| game.player(id))
                        .map(|p| p.has_skill_property(NamedProperties::CANCELS_IGNORE_DEFENDER_STUMBLES_RESULT))
                        .unwrap_or(false);
                    let attacker_can_block_same_team = acting_player_id.as_deref()
                        .and_then(|id| game.player(id))
                        .map(|p| p.has_skill_property(NamedProperties::CAN_BLOCK_SAME_TEAM_PLAYER))
                        .unwrap_or(false);
                    let same_team = acting_player_id.as_deref().zip(defender_id.as_deref())
                        .map(|(a, d)| game.player_team_id(a) == game.player_team_id(d))
                        .unwrap_or(false);
                    let tackle_applies = attacker_has_tackle && (!attacker_can_block_same_team || !same_team);
                    if tackle_applies {
                        let right_stuff_cancels_tackle = game.options.get("rightStuffCancelsTackle") == Some("true");
                        let defender_has_right_stuff = defender_id.as_deref()
                            .and_then(|id| game.player(id))
                            .map(|p| p.has_skill_property(NamedProperties::IGNORE_TACKLE_WHEN_BLOCKED))
                            .unwrap_or(false);
                        if right_stuff_cancels_tackle && defender_has_right_stuff {
                            // Java: getResult().addReport(new ReportSkillUse(game.getDefenderId(), ignoreTackleSkill, true, SkillUse.CANCEL_TACKLE))
                            if let Some(ref did) = defender_id {
                                use ffb_model::model::skill_use::SkillUse;
                                use ffb_model::report::report_skill_use::ReportSkillUse;
                                game.report_list.add(ReportSkillUse::new(
                                    Some(did.clone()), SkillId::RightStuff, true, SkillUse::CANCEL_TACKLE,
                                ));
                                game.report_list.add(ReportSkillUse::new(
                                    Some(did.clone()), SkillId::Dodge, true, SkillUse::AVOID_FALLING,
                                ));
                            }
                            let mut out = StepOutcome::goto(&dodge_label);
                            if let Some(ref did) = defender_id {
                                out = out.with_event(GameEvent::SkillUse { player_id: did.clone(), skill_id: SkillId::Dodge as u16, used: true });
                            }
                            out
                        } else {
                            // Tackle cancels Dodge → defender falls + pushback
                            // Java: getResult().addReport(new ReportSkillUse(actingPlayer.getPlayerId(), attackerCanCancelDodgeSkill, true, SkillUse.CANCEL_DODGE))
                            if let Some(ref aid) = acting_player_id {
                                use ffb_model::model::skill_use::SkillUse;
                                use ffb_model::report::report_skill_use::ReportSkillUse;
                                game.report_list.add(ReportSkillUse::new(
                                    Some(aid.clone()), SkillId::Tackle, true, SkillUse::CANCEL_DODGE,
                                ));
                            }
                            if let Some(ref did) = defender_id {
                                let defender_state = game.field_model.player_state(did).unwrap_or_default();
                                game.field_model.set_player_state(did, defender_state.change_base(PS_FALLING));
                            }
                            let (sq, _) = self.init_pushback(game);
                            let mut out = StepOutcome::goto(&pushback_label);
                            if let Some(ref aid) = acting_player_id {
                                out = out.with_event(GameEvent::SkillUse { player_id: aid.clone(), skill_id: SkillId::Tackle as u16, used: true });
                            }
                            if let Some(ref did) = defender_id {
                                out = out.with_event(GameEvent::SkillUse { player_id: did.clone(), skill_id: SkillId::Dodge as u16, used: false });
                            }
                            out = out.publish(StepParameter::StartingPushbackSquare(sq));
                            out
                        }
                    } else {
                        // No Tackle → Dodge works
                        let mut out = StepOutcome::goto(&dodge_label);
                        if let Some(ref did) = defender_id {
                            out = out.with_event(GameEvent::SkillUse { player_id: did.clone(), skill_id: SkillId::Dodge as u16, used: true });
                        }
                        out
                    }
                } else {
                    // No Dodge → defender falls + pushback
                    if let Some(ref did) = defender_id {
                        let defender_state = game.field_model.player_state(did).unwrap_or_default();
                        game.field_model.set_player_state(did, defender_state.change_base(PS_FALLING));
                    }
                    let (sq, _) = self.init_pushback(game);
                    let mut out = StepOutcome::goto(&pushback_label);
                    out = out.publish(StepParameter::StartingPushbackSquare(sq));
                    out
                }
            }
            BlockResult::Pow => {
                if let Some(defender_id) = game.defender_id.clone() {
                    let defender_state = game.field_model.player_state(&defender_id).unwrap_or_default();
                    game.field_model.set_player_state(&defender_id, defender_state.change_base(PS_FALLING));
                }
                let (sq, _) = self.init_pushback(game);
                let mut out = StepOutcome::goto(&pushback_label);
                out = out.publish(StepParameter::StartingPushbackSquare(sq));
                out
            }
            BlockResult::Pushback => {
                // Restore old state (defender didn't fall)
                if let Some(defender_id) = game.defender_id.clone() {
                    game.field_model.set_player_state(&defender_id, old_defender_state);
                }
                let (sq, _) = self.init_pushback(game);
                let mut out = StepOutcome::goto(&pushback_label);
                out = out.publish(StepParameter::StartingPushbackSquare(sq));
                out
            }
            _ => StepOutcome::next(),
        };

        // Java: getResult().addReport(new ReportBlockChoice(fNrOfDice, fBlockRoll, fDiceIndex,
        //         fBlockResult, game.getDefenderId(), suppressExtraEffectHandling, showNameInReport, blockRollId))
        {
            use ffb_model::report::report_block_choice::ReportBlockChoice;
            game.report_list.add(ReportBlockChoice::new(
                self.nr_of_dice,
                self.block_roll.clone(),
                self.dice_index as i32,
                block_result.name().to_string(),
                game.defender_id.clone().unwrap_or_default(),
                self.suppress_extra_effect_handling,
                self.show_name_in_report,
                self.block_roll_id,
            ));
        }

        outcome
    }

    /// Returns (starting_pushback_square, scatter_ball).
    /// Java: UtilBlockSequence.initPushback(step) — clears pushback squares, finds direction.
    fn init_pushback(&self, game: &mut Game) -> (Option<ffb_model::types::PushbackSquare>, bool) {
        game.field_model.pushback_squares.clear();
        let attacker_coord = game.acting_player.player_id.as_deref()
            .and_then(|id| game.field_model.player_coordinate(id));
        let defender_coord = game.defender_id.as_deref()
            .and_then(|id| game.field_model.player_coordinate(id));
        let starting_sq = attacker_coord.zip(defender_coord)
            .map(|(ac, dc)| UtilServerPushback::find_starting_square(ac, dc, game.home_playing))
            .flatten();
        // Java: scatter_ball = attacker.hasSkillProperty(forceOpponentToDropBallOnPushback)
        let scatter_ball = game.acting_player.player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.has_skill_property(NamedProperties::FORCE_OPPONENT_TO_DROP_BALL_ON_PUSHBACK))
            .unwrap_or(false);
        (starting_sq, scatter_ball)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{Rules, BlockResult, PS_STANDING, PS_FALLING, PlayerGender, PlayerType, SkillId};
    use ffb_model::model::player::Player;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2020)
    }

    #[test]
    fn no_block_result_stays_cont() {
        let mut step = StepBlockChoice::new("dodge".into(), "jugger".into(), "push".into());
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn skull_result_sets_attacker_falling_and_next_step() {
        let mut step = StepBlockChoice::new("dodge".into(), "jugger".into(), "push".into());
        step.block_result = Some(BlockResult::Skull);
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn both_down_gotos_juggernaut_label() {
        let mut step = StepBlockChoice::new("dodge".into(), "jugger".into(), "push".into());
        step.block_result = Some(BlockResult::BothDown);
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("jugger"));
    }

    #[test]
    fn pow_gotos_pushback_label() {
        let mut step = StepBlockChoice::new("dodge".into(), "jugger".into(), "push".into());
        step.block_result = Some(BlockResult::Pow);
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("push"));
    }

    #[test]
    fn pushback_gotos_pushback_label() {
        let mut step = StepBlockChoice::new("dodge".into(), "jugger".into(), "push".into());
        step.block_result = Some(BlockResult::Pushback);
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("push"));
    }

    #[test]
    fn set_parameter_block_result_accepted() {
        let mut step = StepBlockChoice::new("d".into(), "j".into(), "p".into());
        step.set_parameter(&StepParameter::BlockResult(BlockResult::Skull));
        assert_eq!(step.block_result, Some(BlockResult::Skull));
    }

    #[test]
    fn set_parameter_old_defender_state_accepted() {
        let mut step = StepBlockChoice::new("d".into(), "j".into(), "p".into());
        let state = PlayerState::new(PS_STANDING);
        step.set_parameter(&StepParameter::OldDefenderState(state));
        assert!(step.old_defender_state.is_some());
    }

    #[test]
    fn pow_result_emits_report_block_choice() {
        use ffb_model::report::report_id::ReportId;
        let mut step = StepBlockChoice::new("dodge".into(), "jugger".into(), "push".into());
        step.block_result = Some(BlockResult::Pow);
        step.block_roll = vec![4];
        let mut game = make_game();
        // add a defender so field_model doesn't crash on pushback
        game.team_away.players.push(make_bare_player("def", 2_i32));
        game.field_model.set_player_coordinate("def", ffb_model::types::FieldCoordinate::new(6, 5));
        game.field_model.set_player_state("def", PlayerState::new(PS_STANDING));
        game.defender_id = Some("def".into());
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::BLOCK_CHOICE));
    }

    #[test]
    fn skull_result_emits_report_block_choice() {
        use ffb_model::report::report_id::ReportId;
        let mut step = StepBlockChoice::new("dodge".into(), "jugger".into(), "push".into());
        step.block_result = Some(BlockResult::Skull);
        step.block_roll = vec![1];
        let mut game = make_game();
        game.team_home.players.push(make_bare_player("att", 1_i32));
        game.field_model.set_player_coordinate("att", ffb_model::types::FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("att", PlayerState::new(PS_STANDING));
        game.acting_player.player_id = Some("att".into());
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::BLOCK_CHOICE));
    }

    #[test]
    fn tackle_cancels_dodge_emits_cancel_dodge_skill_use() {
        use ffb_model::report::report_id::ReportId;
        use ffb_model::model::skill_def::SkillWithValue;
        let mut step = StepBlockChoice::new("dodge".into(), "jugger".into(), "push".into());
        step.block_result = Some(BlockResult::PowPushback);
        let mut game = make_game();
        game.team_home.players.push(Player {
            id: "att".into(), name: "att".into(), nr: 1, position_id: "l".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![SkillWithValue::new(SkillId::Tackle)],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.field_model.set_player_coordinate("att", ffb_model::types::FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("att", PlayerState::new(PS_STANDING));
        game.team_away.players.push(Player {
            id: "def".into(), name: "def".into(), nr: 2, position_id: "l".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 4, passing: 4, armour: 9,
            starting_skills: vec![SkillWithValue::new(SkillId::Dodge)],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.field_model.set_player_coordinate("def", ffb_model::types::FieldCoordinate::new(6, 5));
        game.field_model.set_player_state("def", PlayerState::new(PS_STANDING));
        game.acting_player.player_id = Some("att".into());
        game.defender_id = Some("def".into());
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::SKILL_USE), "Tackle cancel dodge must emit ReportSkillUse");
        assert!(game.report_list.has_report(ReportId::BLOCK_CHOICE));
    }

    fn make_bare_player(id: &str, nr: i32) -> Player {
        Player {
            id: id.into(), name: id.into(), nr, position_id: "l".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        }
    }
}
