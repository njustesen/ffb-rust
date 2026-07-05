/// 1:1 translation of com.fumbbl.ffb.server.step.action.block.StepJuggernaut (COMMON rules)
/// and its BB2016/BB2020/BB2025 hook com.fumbbl.ffb.server.skillbehaviour.*.JuggernautBehaviour.
///
/// Juggernaut lets the attacker convert a Both-Down result into a Pushback during a Blitz.
/// The coach is prompted; random agent always declines (stub: skip dialog → NEXT_STEP).
///
/// On use:
///   - Block result overridden to Pushback
///   - Defender restored to old state
///   - initPushback stub: starting pushback square = defender coordinate
///   - GOTO_LABEL_ON_SUCCESS
///
/// Needs GOTO_LABEL_ON_SUCCESS init parameter.
/// Expects OLD_DEFENDER_STATE parameter from a preceding step.
use ffb_model::enums::{PlayerAction, SkillId};
use ffb_model::enums::BlockResult;
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::util::util_server_pushback::UtilServerPushback;

pub struct StepJuggernaut {
    /// Java: state.goToLabelOnSuccess — GOTO_LABEL_ON_SUCCESS init parameter.
    pub goto_label_on_success: String,
    /// Java: state.usingJuggernaut — None = waiting for coach, Some(true/false) = decided.
    pub using_juggernaut: Option<bool>,
    /// Java: state.oldDefenderState — defender state before the block result was applied.
    pub old_defender_state: Option<ffb_model::enums::PlayerState>,
}

impl StepJuggernaut {
    pub fn new() -> Self {
        Self {
            goto_label_on_success: String::new(),
            using_juggernaut: None,
            old_defender_state: None,
        }
    }
}

impl Default for StepJuggernaut {
    fn default() -> Self { Self::new() }
}

impl Step for StepJuggernaut {
    fn id(&self) -> StepId { StepId::Juggernaut }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: handleCommandHook sets state.usingJuggernaut from CLIENT_USE_SKILL command.
        if let Action::UseSkill { use_skill, .. } = action {
            self.using_juggernaut = Some(*use_skill);
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnSuccess(v) => { self.goto_label_on_success = v.clone(); true }
            StepParameter::OldDefenderState(v)   => { self.old_defender_state = Some(*v); true }
            _ => false,
        }
    }
}

impl StepJuggernaut {
    /// Java: JuggernautBehaviour.handleExecuteStepHook
    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        let player_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        let is_blitz = game.acting_player.player_action == Some(PlayerAction::Blitz);
        let has_juggernaut = game.player(&player_id)
            .map(|p| p.has_skill(SkillId::Juggernaut) && !p.used_skills.contains(&SkillId::Juggernaut))
            .unwrap_or(false);

        if !is_blitz || !has_juggernaut {
            return StepOutcome::next();
        }

        // Java: if usingJuggernaut == null → showDialog (prompt coach).
        // Stub: random agent never answers dialogs → treat as declined (false).
        let using = self.using_juggernaut.unwrap_or(false);

        let skill_num = SkillId::Juggernaut as u16;

        if using {
            // Java: publish BLOCK_RESULT(Pushback), restore defender, initPushback, goto label.
            let old_state = self.old_defender_state.unwrap_or_default();
            if let Some(defender_id) = game.defender_id.clone() {
                game.field_model.set_player_state(&defender_id, old_state);
            }

            game.field_model.pushback_squares.clear();
            let attacker_coord = game.acting_player.player_id.as_deref()
                .and_then(|id| game.field_model.player_coordinate(id));
            let defender_coord = game.defender_id.as_deref()
                .and_then(|id| game.field_model.player_coordinate(id));
            let starting_sq = attacker_coord.zip(defender_coord)
                .map(|(ac, dc)| UtilServerPushback::find_starting_square(ac, dc, game.home_playing))
                .flatten();

            let mut outcome = StepOutcome::goto(&self.goto_label_on_success)
                .with_event(GameEvent::SkillUse { player_id, skill_id: skill_num, used: true })
                .publish(StepParameter::BlockResult(BlockResult::Pushback));
            outcome = outcome.publish(StepParameter::StartingPushbackSquare(starting_sq));
            outcome
        } else {
            // Java: addReport(SkillUse false); NEXT_STEP
            StepOutcome::next()
                .with_event(GameEvent::SkillUse { player_id, skill_id: skill_num, used: false })
        }
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

    fn make_blitz_game(skills: Vec<SkillId>) -> Game {
        let mut home = test_team("home", 0);
        let mut away = test_team("away", 0);
        add_player(&mut home, "att", 1, skills);
        add_player(&mut away, "def", 2, vec![]);
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.home_playing = true;
        game.acting_player.player_id = Some("att".into());
        game.acting_player.player_action = Some(PlayerAction::Blitz);
        game.defender_id = Some("def".into());
        game.field_model.set_player_coordinate("att", FieldCoordinate::new(5, 5));
        game.field_model.set_player_coordinate("def", FieldCoordinate::new(6, 5));
        game.field_model.set_player_state("att", PlayerState::new(PS_STANDING).change_active(true));
        game.field_model.set_player_state("def", PlayerState::new(PS_STANDING));
        game
    }

    #[test]
    fn no_acting_player_returns_next() {
        let mut game = make_blitz_game(vec![SkillId::Juggernaut]);
        game.acting_player.player_id = None;
        let outcome = StepJuggernaut::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.events.is_empty());
    }

    #[test]
    fn no_juggernaut_skill_returns_next() {
        let mut game = make_blitz_game(vec![]);
        let outcome = StepJuggernaut::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.events.is_empty());
    }

    #[test]
    fn non_blitz_action_returns_next() {
        let mut game = make_blitz_game(vec![SkillId::Juggernaut]);
        game.acting_player.player_action = Some(PlayerAction::Block);
        let outcome = StepJuggernaut::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.events.is_empty());
    }

    #[test]
    fn dialog_not_answered_declines_skill() {
        // using_juggernaut == None → treated as declined (false) → NEXT_STEP
        let mut game = make_blitz_game(vec![SkillId::Juggernaut]);
        let mut step = StepJuggernaut::new();
        step.goto_label_on_success = "JUG".into();
        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.events.iter().any(|e| matches!(e, GameEvent::SkillUse { used: false, .. })));
    }

    #[test]
    fn declining_skill_returns_next() {
        let mut game = make_blitz_game(vec![SkillId::Juggernaut]);
        let mut step = StepJuggernaut::new();
        step.goto_label_on_success = "JUG".into();
        step.using_juggernaut = Some(false);
        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.events.iter().any(|e| matches!(e, GameEvent::SkillUse { used: false, .. })));
    }

    #[test]
    fn using_juggernaut_goes_to_label() {
        let mut game = make_blitz_game(vec![SkillId::Juggernaut]);
        let mut step = StepJuggernaut::new();
        step.goto_label_on_success = "PUSHBACK_LABEL".into();
        step.using_juggernaut = Some(true);
        step.old_defender_state = Some(PlayerState::new(PS_STANDING));
        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::GotoLabel);
        assert_eq!(outcome.goto_label.as_deref(), Some("PUSHBACK_LABEL"));
    }

    #[test]
    fn using_juggernaut_publishes_pushback_block_result() {
        let mut game = make_blitz_game(vec![SkillId::Juggernaut]);
        let mut step = StepJuggernaut::new();
        step.goto_label_on_success = "JUG".into();
        step.using_juggernaut = Some(true);
        step.old_defender_state = Some(PlayerState::new(PS_STANDING));
        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert!(outcome.published.iter().any(|p| matches!(p, StepParameter::BlockResult(BlockResult::Pushback))));
    }

    #[test]
    fn using_juggernaut_publishes_starting_pushback_square() {
        let mut game = make_blitz_game(vec![SkillId::Juggernaut]);
        let mut step = StepJuggernaut::new();
        step.goto_label_on_success = "JUG".into();
        step.using_juggernaut = Some(true);
        step.old_defender_state = Some(PlayerState::new(PS_STANDING));
        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert!(outcome.published.iter().any(|p| matches!(p, StepParameter::StartingPushbackSquare(_))));
    }

    #[test]
    fn using_juggernaut_restores_defender_state() {
        let mut game = make_blitz_game(vec![SkillId::Juggernaut]);
        // Put defender in a different state first
        use ffb_model::enums::PS_FALLING;
        game.field_model.set_player_state("def", PlayerState::new(PS_FALLING));
        let mut step = StepJuggernaut::new();
        step.goto_label_on_success = "JUG".into();
        step.using_juggernaut = Some(true);
        step.old_defender_state = Some(PlayerState::new(PS_STANDING));
        step.start(&mut game, &mut GameRng::new(0));
        let state = game.field_model.player_state("def").unwrap();
        assert_eq!(state.base(), PS_STANDING);
    }

    #[test]
    fn set_parameter_stores_goto_and_old_state() {
        let mut step = StepJuggernaut::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnSuccess("X".into())));
        assert!(step.set_parameter(&StepParameter::OldDefenderState(PlayerState::new(PS_STANDING))));
        assert_eq!(step.goto_label_on_success, "X");
        assert!(step.old_defender_state.is_some());
        assert!(!step.set_parameter(&StepParameter::EndPlayerAction(true)));
    }
}
