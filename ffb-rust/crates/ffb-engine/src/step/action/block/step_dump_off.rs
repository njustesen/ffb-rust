/// 1:1 translation of com.fumbbl.ffb.server.step.action.block.StepDumpOff (COMMON rules)
/// and its BB2016/BB2020/BB2025 hook com.fumbbl.ffb.server.skillbehaviour.*.DumpOffBehaviour.
///
/// DumpOff lets the defender make a pass when they are about to be blocked.
/// The defender must have the DumpOff skill, be holding the ball, and not be confused/hypnotized.
///
/// Expects DEFENDER_POSITION parameter from a preceding step.
use ffb_model::enums::{PlayerAction, SkillId, TurnMode};
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::sequences::pass_sequence;

pub struct StepDumpOff {
    /// Java: state.usingDumpOff — None = not asked, Some = decided.
    pub using_dump_off: Option<bool>,
    /// Java: state.defenderPosition — set by DEFENDER_POSITION parameter.
    pub defender_position: Option<FieldCoordinate>,
    /// Java: state.oldTurnMode — saved when DumpOff pass is initiated, restored when done.
    pub old_turn_mode: Option<TurnMode>,
}

impl StepDumpOff {
    pub fn new() -> Self {
        Self {
            using_dump_off: None,
            defender_position: None,
            old_turn_mode: None,
        }
    }
}

impl Default for StepDumpOff {
    fn default() -> Self { Self::new() }
}

impl Step for StepDumpOff {
    fn id(&self) -> StepId { StepId::DumpOff }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if let Action::UseSkill { use_skill, .. } = action {
            self.using_dump_off = Some(*use_skill);
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::DefenderPosition(v) => { self.defender_position = Some(*v); true }
            _ => false,
        }
    }
}

impl StepDumpOff {
    /// Java: DumpOffBehaviour.handleExecuteStepHook (logic identical across editions).
    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: if (turnMode == DUMP_OFF) → restore old turn mode and clear thrower.
        if game.turn_mode == TurnMode::DumpOff {
            if let Some(old) = self.old_turn_mode {
                game.turn_mode = old;
            }
            game.thrower_id = None;
            return StepOutcome::next();
        }

        // Java: if usingDumpOff == null → check conditions.
        if self.using_dump_off.is_none() {
            let can_dump_off = self.can_use_dump_off(game);
            if can_dump_off {
                // Java: showDialog → CONTINUE (random agent declines → stub as false).
                self.using_dump_off = Some(false);
            } else {
                self.using_dump_off = Some(false);
            }
        }

        let using = self.using_dump_off.unwrap_or(false);
        let defender_id = game.defender_id.clone();
        let skill_num = SkillId::DumpOff as u16;

        if using {
            // Java: save old turn mode, switch to DumpOff, set thrower, push Pass sequence.
            self.old_turn_mode = Some(game.turn_mode);
            game.turn_mode = TurnMode::DumpOff;
            if let Some(ref did) = defender_id {
                game.thrower_id = Some(did.clone());
            }
            game.acting_player.player_action = Some(PlayerAction::DumpOff);
            game.defender_action = Some(PlayerAction::DumpOff);
            let event = GameEvent::SkillUse {
                player_id: defender_id.unwrap_or_default(),
                skill_id: skill_num,
                used: true,
            };
            StepOutcome::next().with_event(event).push_seq(pass_sequence())
        } else {
            // Java: addReport(SkillUse false if skill present); NEXT_STEP.
            let defender_has = defender_id.as_deref()
                .and_then(|id| game.player(id))
                .map(|p| p.has_skill(SkillId::DumpOff))
                .unwrap_or(false);
            if defender_has {
                let event = GameEvent::SkillUse {
                    player_id: defender_id.unwrap_or_default(),
                    skill_id: skill_num,
                    used: false,
                };
                StepOutcome::next().with_event(event)
            } else {
                StepOutcome::next()
            }
        }
    }

    /// Java: condition check inside the (usingDumpOff == null) branch.
    fn can_use_dump_off(&self, game: &Game) -> bool {
        let defender_id = match &game.defender_id {
            Some(id) => id,
            None => return false,
        };
        let has_skill = game.player(defender_id)
            .map(|p| p.has_skill(SkillId::DumpOff))
            .unwrap_or(false);
        if !has_skill { return false; }
        if self.defender_position.is_none() { return false; }
        // Java: defenderPosition.equals(game.getFieldModel().getBallCoordinate())
        let ball_at_defender = game.field_model.ball_coordinate == self.defender_position;
        if !ball_at_defender { return false; }
        // Java: !game.getFieldModel().isBallMoving()
        if game.field_model.ball_moving { return false; }
        // Java: !defenderState.isConfused() && !defenderState.isHypnotized()
        let state = game.field_model.player_state(defender_id);
        if state.map(|s| s.is_confused() || s.is_hypnotized()).unwrap_or(false) {
            return false;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{Rules, PS_STANDING};
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::types::FieldCoordinate;
    use ffb_model::enums::PlayerState;

    fn add_player(
        team: &mut ffb_model::model::team::Team,
        id: &str,
        nr: i32,
        skills: Vec<SkillId>,
    ) {
        team.players.push(ffb_model::model::player::Player {
            id: id.into(),
            name: id.into(),
            nr,
            position_id: "pos".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: skills
                .into_iter()
                .map(|s| SkillWithValue { skill_id: s, value: None })
                .collect(),
            extra_skills: vec![],
            temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0,
            stat_injuries: vec![],
            current_spps: 0,
            career_spps: 0,
            race: None,
            ..Default::default()
        });
    }

    fn make_game(defender_skills: Vec<SkillId>) -> Game {
        let mut home = test_team("home", 0);
        let mut away = test_team("away", 0);
        add_player(&mut home, "att", 1, vec![]);
        add_player(&mut away, "def", 2, defender_skills);
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.home_playing = true;
        game.acting_player.player_id = Some("att".into());
        game.defender_id = Some("def".into());
        game.field_model.set_player_coordinate("att", FieldCoordinate::new(5, 5));
        game.field_model.set_player_coordinate("def", FieldCoordinate::new(6, 5));
        game.field_model.set_player_state("att", PlayerState::new(PS_STANDING).change_active(true));
        game.field_model.set_player_state("def", PlayerState::new(PS_STANDING));
        game
    }

    #[test]
    fn no_dump_off_skill_returns_next_no_event() {
        let mut game = make_game(vec![]);
        let outcome = StepDumpOff::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.events.is_empty());
    }

    #[test]
    fn defender_has_skill_but_no_ball_returns_next_with_declined_event() {
        let mut game = make_game(vec![SkillId::DumpOff]);
        // Ball not at defender's position
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(3, 3));
        let mut step = StepDumpOff::new();
        step.defender_position = Some(FieldCoordinate::new(6, 5)); // defender is at (6,5), ball at (3,3)
        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        // Skill declined (has skill but conditions not met → using_dump_off = false)
        assert!(outcome.events.iter().any(|e| matches!(e, GameEvent::SkillUse { used: false, .. })));
    }

    #[test]
    fn ball_at_defender_with_skill_declines_by_stub() {
        let mut game = make_game(vec![SkillId::DumpOff]);
        let def_pos = FieldCoordinate::new(6, 5);
        game.field_model.ball_coordinate = Some(def_pos);
        let mut step = StepDumpOff::new();
        step.defender_position = Some(def_pos);
        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        // Stub: random agent declines → emit declined event
        assert!(outcome.events.iter().any(|e| matches!(e, GameEvent::SkillUse { used: false, .. })));
    }

    #[test]
    fn using_dump_off_true_sets_turn_mode_and_thrower() {
        let mut game = make_game(vec![SkillId::DumpOff]);
        let mut step = StepDumpOff::new();
        step.using_dump_off = Some(true);
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.turn_mode, TurnMode::DumpOff);
        assert_eq!(game.thrower_id.as_deref(), Some("def"));
    }

    #[test]
    fn using_dump_off_true_emits_skill_used_event() {
        let mut game = make_game(vec![SkillId::DumpOff]);
        let mut step = StepDumpOff::new();
        step.using_dump_off = Some(true);
        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert!(outcome.events.iter().any(|e| matches!(e, GameEvent::SkillUse { used: true, .. })));
    }

    #[test]
    fn dump_off_turn_mode_restores_old_mode() {
        let mut game = make_game(vec![]);
        game.turn_mode = TurnMode::DumpOff;
        let mut step = StepDumpOff::new();
        step.old_turn_mode = Some(TurnMode::Regular);
        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert_eq!(game.turn_mode, TurnMode::Regular);
        assert!(game.thrower_id.is_none());
    }

    #[test]
    fn set_parameter_stores_defender_position() {
        let mut step = StepDumpOff::new();
        let pos = FieldCoordinate::new(7, 4);
        assert!(step.set_parameter(&StepParameter::DefenderPosition(pos)));
        assert_eq!(step.defender_position, Some(pos));
        assert!(!step.set_parameter(&StepParameter::EndPlayerAction(true)));
    }
}
