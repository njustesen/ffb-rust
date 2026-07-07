/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.block.StepBlockChoice`.
///
/// Handles the block die choice: branches on `BlockResult` to fall/pushback/juggernaut/dodge.
///
/// Init parameters: GOTO_LABEL_ON_DODGE, GOTO_LABEL_ON_JUGGERNAUT, GOTO_LABEL_ON_PUSHBACK.
/// Expects: DICE_INDEX, BLOCK_RESULT, BLOCK_ROLL, NR_OF_DICE, OLD_DEFENDER_STATE.
///
use ffb_model::enums::{BlockResult, PlayerState, PS_FALLING, SkillId};
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::action::block::util_block_sequence::init_pushback;

/// Java: `StepBlockChoice` (bb2016/block).
pub struct StepBlockChoice {
    /// Java: `fGotoLabelOnDodge` — init param (mandatory).
    goto_label_on_dodge: String,
    /// Java: `fGotoLabelOnJuggernaut` — init param (mandatory).
    goto_label_on_juggernaut: String,
    /// Java: `fGotoLabelOnPushback` — init param (mandatory).
    goto_label_on_pushback: String,

    /// Java: `fNrOfDice`
    nr_of_dice: i32,
    /// Java: `fBlockRoll`
    block_roll: Vec<i32>,
    /// Java: `fDiceIndex`
    dice_index: usize,
    /// Java: `blockRollId`
    block_roll_id: i32,
    /// Java: `fBlockResult`
    block_result: Option<BlockResult>,
    /// Java: `fOldDefenderState`
    old_defender_state: Option<PlayerState>,
    /// Java: `suppressExtraEffectHandling`
    suppress_extra_effect_handling: bool,
    /// Java: `showNameInReport`
    show_name_in_report: bool,
}

impl StepBlockChoice {
    pub fn new() -> Self {
        Self {
            goto_label_on_dodge: String::new(),
            goto_label_on_juggernaut: String::new(),
            goto_label_on_pushback: String::new(),
            nr_of_dice: 0,
            block_roll: Vec::new(),
            dice_index: 0,
            block_roll_id: 0,
            block_result: None,
            old_defender_state: None,
            suppress_extra_effect_handling: false,
            show_name_in_report: false,
        }
    }

    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        let block_result = match self.block_result {
            None => return StepOutcome::next(),
            Some(r) => r,
        };

        let outcome = match block_result {
            BlockResult::Skull => {
                // attacker falls; defender restores old state
                if let Some(attacker_id) = game.acting_player.player_id.clone() {
                    if let Some(state) = game.field_model.player_state(&attacker_id) {
                        game.field_model.set_player_state(&attacker_id, state.change_base(PS_FALLING));
                    }
                }
                if let (Some(defender_id), Some(old)) = (game.defender_id.clone(), self.old_defender_state) {
                    game.field_model.set_player_state(&defender_id, old);
                }
                StepOutcome::next()
            }
            BlockResult::BothDown => {
                StepOutcome::goto(&self.goto_label_on_juggernaut)
            }
            BlockResult::PowPushback => {
                // Java: check if defender has Dodge (ignoreDefenderStumblesResult).
                // If so and attacker doesn't cancel it with Tackle (cancelsDodge), goto dodge label.
                // If attacker has Tackle that's cancelled by defender's Right Stuff (ignoreTackleWhenBlocked)
                // and RIGHT_STUFF_CANCELS_TACKLE option is set, also goto dodge label.
                // Otherwise defender falls + pushback.
                let defender_id = game.defender_id.clone();
                let acting_player_id = game.acting_player.player_id.clone();

                let defender_has_dodge = defender_id.as_deref()
                    .and_then(|id| game.player(id))
                    .map(|p| p.has_skill_property(NamedProperties::IGNORE_DEFENDER_STUMBLES_RESULT))
                    .unwrap_or(false);

                if defender_has_dodge {
                    // Java: attackerCanCancelDodgeSkill = getSkillCancelling(attacker, dodgeSkill)
                    // Rust: attacker has "cancelsIgnoreDefenderStumblesResult" (= Tackle)
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
                    let tackle_applies = attacker_has_tackle
                        && (!attacker_can_block_same_team || !same_team);

                    if tackle_applies {
                        // Java: check RIGHT_STUFF_CANCELS_TACKLE option + defender.ignoreTackleWhenBlocked
                        let right_stuff_cancels_tackle = game.options.get("rightStuffCancelsTackle") == Some("true");
                        let defender_has_right_stuff = defender_id.as_deref()
                            .and_then(|id| game.player(id))
                            .map(|p| p.has_skill_property(NamedProperties::IGNORE_TACKLE_WHEN_BLOCKED))
                            .unwrap_or(false);
                        if right_stuff_cancels_tackle && defender_has_right_stuff {
                            // Right Stuff cancels Tackle → Dodge still works → goto dodge
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
                                let _ = did;
                            }
                            let mut outcome = StepOutcome::goto(&self.goto_label_on_dodge);
                            if let Some(ref did) = defender_id {
                                outcome = outcome.with_event(GameEvent::SkillUse { player_id: did.clone(), skill_id: SkillId::Dodge as u16, used: true });
                            }
                            outcome
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
                                if let Some(state) = game.field_model.player_state(did) {
                                    game.field_model.set_player_state(did, state.change_base(PS_FALLING));
                                }
                            }
                            let pushback = init_pushback(game);
                            let mut outcome = StepOutcome::goto(&self.goto_label_on_pushback);
                            if let Some(ref aid) = acting_player_id {
                                outcome = outcome.with_event(GameEvent::SkillUse { player_id: aid.clone(), skill_id: SkillId::Tackle as u16, used: true });
                            }
                            if let Some(ref did) = defender_id {
                                outcome = outcome.with_event(GameEvent::SkillUse { player_id: did.clone(), skill_id: SkillId::Dodge as u16, used: false });
                            }
                            outcome.published.extend(pushback);
                            outcome
                        }
                    } else {
                        // No Tackle (or same-team block) → Dodge works → goto dodge
                        let mut outcome = StepOutcome::goto(&self.goto_label_on_dodge);
                        if let Some(ref did) = defender_id {
                            outcome = outcome.with_event(GameEvent::SkillUse { player_id: did.clone(), skill_id: SkillId::Dodge as u16, used: true });
                        }
                        outcome
                    }
                } else {
                    // No Dodge → defender falls + pushback
                    if let Some(ref did) = defender_id {
                        if let Some(state) = game.field_model.player_state(did) {
                            game.field_model.set_player_state(did, state.change_base(PS_FALLING));
                        }
                    }
                    let pushback = init_pushback(game);
                    let mut outcome = StepOutcome::goto(&self.goto_label_on_pushback);
                    outcome.published.extend(pushback);
                    outcome
                }
            }
            BlockResult::Pow => {
                if let Some(defender_id) = game.defender_id.clone() {
                    if let Some(state) = game.field_model.player_state(&defender_id) {
                        game.field_model.set_player_state(&defender_id, state.change_base(PS_FALLING));
                    }
                }
                let pushback = init_pushback(game);
                let mut outcome = StepOutcome::goto(&self.goto_label_on_pushback);
                outcome.published.extend(pushback);
                outcome
            }
            BlockResult::Pushback => {
                if let (Some(defender_id), Some(old)) = (game.defender_id.clone(), self.old_defender_state) {
                    game.field_model.set_player_state(&defender_id, old);
                }
                let pushback = init_pushback(game);
                let mut outcome = StepOutcome::goto(&self.goto_label_on_pushback);
                outcome.published.extend(pushback);
                outcome
            }
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
}

impl Default for StepBlockChoice {
    fn default() -> Self { Self::new() }
}

impl Step for StepBlockChoice {
    fn id(&self) -> StepId { StepId::BlockChoice }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnDodge(s)      => { self.goto_label_on_dodge = s.clone(); true }
            StepParameter::GotoLabelOnJuggernaut(s) => { self.goto_label_on_juggernaut = s.clone(); true }
            StepParameter::GotoLabelOnPushback(s)   => { self.goto_label_on_pushback = s.clone(); true }
            StepParameter::DiceIndex(i)             => { self.dice_index = *i; true }
            StepParameter::BlockResult(r)           => { self.block_result = Some(*r); true }
            StepParameter::BlockRoll(r)             => { self.block_roll = r.clone(); true }
            StepParameter::NrOfDice(n)              => { self.nr_of_dice = *n; true }
            StepParameter::OldDefenderState(s)      => { self.old_defender_state = Some(*s); true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{Rules, PS_STANDING, PS_PRONE};
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    fn add_player(game: &mut Game, id: &str, state_base: u32) {
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
                    ..Default::default()
});
        game.field_model.set_player_coordinate(id, FieldCoordinate::new(5, 5));
        game.field_model.set_player_state(id, PlayerState::new(state_base));
    }

    #[test]
    fn id_is_block_choice() {
        assert_eq!(StepBlockChoice::new().id(), StepId::BlockChoice);
    }

    #[test]
    fn skull_attacker_falls_defender_restores() {
        let mut step = StepBlockChoice::new();
        step.set_parameter(&StepParameter::BlockResult(BlockResult::Skull));
        step.set_parameter(&StepParameter::OldDefenderState(PlayerState::new(PS_PRONE)));
        let mut game = make_game();
        add_player(&mut game, "att", PS_STANDING);
        add_player(&mut game, "def", PS_STANDING);
        game.acting_player.player_id = Some("att".into());
        game.defender_id = Some("def".into());
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert!(matches!(outcome.action, StepAction::NextStep));
        assert_eq!(game.field_model.player_state("att").unwrap().base(), PS_FALLING);
        assert_eq!(game.field_model.player_state("def").unwrap().base(), PS_PRONE);
    }

    #[test]
    fn both_down_goes_to_juggernaut_label() {
        let mut step = StepBlockChoice::new();
        step.set_parameter(&StepParameter::GotoLabelOnJuggernaut("jug".into()));
        step.set_parameter(&StepParameter::BlockResult(BlockResult::BothDown));
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert!(matches!(outcome.action, StepAction::GotoLabel));
        assert_eq!(outcome.goto_label.as_deref(), Some("jug"));
    }

    #[test]
    fn pow_defender_falls_goto_pushback() {
        let mut step = StepBlockChoice::new();
        step.set_parameter(&StepParameter::GotoLabelOnPushback("push".into()));
        step.set_parameter(&StepParameter::BlockResult(BlockResult::Pow));
        let mut game = make_game();
        add_player(&mut game, "def", PS_STANDING);
        game.defender_id = Some("def".into());
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert!(matches!(outcome.action, StepAction::GotoLabel));
        assert_eq!(outcome.goto_label.as_deref(), Some("push"));
        assert_eq!(game.field_model.player_state("def").unwrap().base(), PS_FALLING);
    }

    #[test]
    fn pushback_defender_restores_goto_pushback() {
        let old_state = PlayerState::new(PS_PRONE);
        let mut step = StepBlockChoice::new();
        step.set_parameter(&StepParameter::GotoLabelOnPushback("push".into()));
        step.set_parameter(&StepParameter::BlockResult(BlockResult::Pushback));
        step.set_parameter(&StepParameter::OldDefenderState(old_state));
        let mut game = make_game();
        add_player(&mut game, "def", PS_STANDING);
        game.defender_id = Some("def".into());
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert!(matches!(outcome.action, StepAction::GotoLabel));
        // defender restored to old state
        assert_eq!(game.field_model.player_state("def").unwrap().base(), PS_PRONE);
    }

    #[test]
    fn no_block_result_returns_next() {
        let mut step = StepBlockChoice::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert!(matches!(outcome.action, StepAction::NextStep));
    }

    #[test]
    fn set_parameter_labels_stored() {
        let mut step = StepBlockChoice::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnDodge("d".into())));
        assert!(step.set_parameter(&StepParameter::GotoLabelOnJuggernaut("j".into())));
        assert!(step.set_parameter(&StepParameter::GotoLabelOnPushback("p".into())));
        assert_eq!(step.goto_label_on_dodge, "d");
        assert_eq!(step.goto_label_on_juggernaut, "j");
        assert_eq!(step.goto_label_on_pushback, "p");
    }

    #[test]
    fn pow_result_emits_report_block_choice() {
        use ffb_model::report::report_id::ReportId;
        let mut step = StepBlockChoice::new();
        step.set_parameter(&StepParameter::GotoLabelOnPushback("push".into()));
        step.set_parameter(&StepParameter::BlockResult(BlockResult::Pow));
        step.set_parameter(&StepParameter::BlockRoll(vec![6]));
        step.set_parameter(&StepParameter::NrOfDice(1));
        let mut game = make_game();
        add_player(&mut game, "def", PS_STANDING);
        game.defender_id = Some("def".into());
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::BLOCK_CHOICE));
    }

    #[test]
    fn skull_result_emits_report_block_choice() {
        use ffb_model::report::report_id::ReportId;
        let mut step = StepBlockChoice::new();
        step.set_parameter(&StepParameter::BlockResult(BlockResult::Skull));
        step.set_parameter(&StepParameter::BlockRoll(vec![1]));
        step.set_parameter(&StepParameter::NrOfDice(1));
        let mut game = make_game();
        add_player(&mut game, "att", PS_STANDING);
        game.acting_player.player_id = Some("att".into());
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::BLOCK_CHOICE));
    }

    #[test]
    fn tackle_cancels_dodge_emits_cancel_dodge_skill_use() {
        use ffb_model::model::skill_def::SkillWithValue;
        use ffb_model::enums::SkillId;
        use ffb_model::report::report_id::ReportId;
        let mut step = StepBlockChoice::new();
        step.set_parameter(&StepParameter::GotoLabelOnPushback("push".into()));
        step.set_parameter(&StepParameter::GotoLabelOnDodge("dodge".into()));
        step.set_parameter(&StepParameter::BlockResult(BlockResult::PowPushback));
        let mut game = make_game();
        // attacker: has Tackle
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
        game.field_model.set_player_coordinate("att", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("att", PlayerState::new(PS_STANDING));
        // defender: has Dodge
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
        game.field_model.set_player_coordinate("def", FieldCoordinate::new(6, 5));
        game.field_model.set_player_state("def", PlayerState::new(PS_STANDING));
        game.acting_player.player_id = Some("att".into());
        game.defender_id = Some("def".into());
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::SKILL_USE), "Tackle cancel dodge must emit ReportSkillUse");
        assert!(game.report_list.has_report(ReportId::BLOCK_CHOICE));
    }

    fn add_away_player(game: &mut Game, id: &str, state_base: u32) {
        game.team_away.players.push(Player {
            id: id.into(), name: id.into(), nr: 2, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
                    ..Default::default()
});
        game.field_model.set_player_coordinate(id, FieldCoordinate::new(6, 5));
        game.field_model.set_player_state(id, PlayerState::new(state_base));
    }

    #[test]
    fn pow_pushback_defender_no_dodge_falls_and_goto_pushback() {
        let mut step = StepBlockChoice::new();
        step.set_parameter(&StepParameter::GotoLabelOnPushback("push".into()));
        step.set_parameter(&StepParameter::GotoLabelOnDodge("dodge".into()));
        step.set_parameter(&StepParameter::BlockResult(BlockResult::PowPushback));
        let mut game = make_game();
        add_player(&mut game, "att", PS_STANDING);
        add_away_player(&mut game, "def", PS_STANDING);
        game.acting_player.player_id = Some("att".into());
        game.defender_id = Some("def".into());
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert_eq!(outcome.goto_label.as_deref(), Some("push"));
        assert_eq!(game.field_model.player_state("def").unwrap().base(), PS_FALLING);
    }

    #[test]
    fn pow_pushback_defender_has_dodge_and_no_attacker_tackle_goes_to_dodge_label() {
        use ffb_model::model::skill_def::SkillWithValue;
        use ffb_model::enums::SkillId;
        let mut step = StepBlockChoice::new();
        step.set_parameter(&StepParameter::GotoLabelOnPushback("push".into()));
        step.set_parameter(&StepParameter::GotoLabelOnDodge("dodge".into()));
        step.set_parameter(&StepParameter::BlockResult(BlockResult::PowPushback));
        let mut game = make_game();
        // attacker: no Tackle
        add_player(&mut game, "att", PS_STANDING);
        // defender: has Dodge (ignoreDefenderStumblesResult)
        game.team_away.players.push(Player {
            id: "def".into(), name: "def".into(), nr: 2, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 4, passing: 4, armour: 9,
            starting_skills: vec![SkillWithValue::new(SkillId::Dodge)],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
                    ..Default::default()
});
        game.field_model.set_player_coordinate("def", FieldCoordinate::new(6, 5));
        game.field_model.set_player_state("def", PlayerState::new(PS_STANDING));
        game.acting_player.player_id = Some("att".into());
        game.defender_id = Some("def".into());
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert_eq!(outcome.goto_label.as_deref(), Some("dodge"));
        assert_eq!(game.field_model.player_state("def").unwrap().base(), PS_STANDING, "Dodge should prevent falling");
    }

    #[test]
    fn pow_pushback_defender_has_dodge_attacker_has_tackle_goes_to_pushback() {
        use ffb_model::model::skill_def::SkillWithValue;
        use ffb_model::enums::SkillId;
        let mut step = StepBlockChoice::new();
        step.set_parameter(&StepParameter::GotoLabelOnPushback("push".into()));
        step.set_parameter(&StepParameter::GotoLabelOnDodge("dodge".into()));
        step.set_parameter(&StepParameter::BlockResult(BlockResult::PowPushback));
        let mut game = make_game();
        // attacker: has Tackle
        game.team_home.players.push(Player {
            id: "att".into(), name: "att".into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![SkillWithValue::new(SkillId::Tackle)],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
                    ..Default::default()
});
        game.field_model.set_player_coordinate("att", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("att", PlayerState::new(PS_STANDING));
        // defender: has Dodge
        game.team_away.players.push(Player {
            id: "def".into(), name: "def".into(), nr: 2, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 4, passing: 4, armour: 9,
            starting_skills: vec![SkillWithValue::new(SkillId::Dodge)],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
                    ..Default::default()
});
        game.field_model.set_player_coordinate("def", FieldCoordinate::new(6, 5));
        game.field_model.set_player_state("def", PlayerState::new(PS_STANDING));
        game.acting_player.player_id = Some("att".into());
        game.defender_id = Some("def".into());
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert_eq!(outcome.goto_label.as_deref(), Some("push"), "Tackle should cancel Dodge");
        assert_eq!(game.field_model.player_state("def").unwrap().base(), PS_FALLING);
    }
}
