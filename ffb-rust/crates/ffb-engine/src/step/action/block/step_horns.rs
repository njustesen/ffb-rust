/// 1:1 translation of com.fumbbl.ffb.server.step.action.block.StepHorns (COMMON rules)
/// and its COMMON hook com.fumbbl.ffb.server.skillbehaviour.common.HornsBehaviour.
///
/// Horns gives the attacker +1 ST during a Blitz action.  The actual ST bonus is applied
/// in the block-dice calculation (ServerUtilBlock#getAttackerStrength); this step just marks
/// the skill used and emits the event so the UI can show it.
///
/// If the attacker has Horns and is performing a Blitz: mark used, emit SkillUse(true).
/// Otherwise: no event.  Always NEXT_STEP.
use ffb_model::enums::{PlayerAction, SkillId};
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

pub struct StepHorns {
    /// Java: state.usingHorns — set during executeStep, internal only (not published).
    pub using_horns: bool,
}

impl StepHorns {
    pub fn new() -> Self { Self { using_horns: false } }
}

impl Default for StepHorns {
    fn default() -> Self { Self::new() }
}

impl Step for StepHorns {
    fn id(&self) -> StepId { StepId::Horns }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

impl StepHorns {
    /// Java: HornsBehaviour.handleExecuteStepHook
    fn execute_step(&mut self, game: &mut Game) -> StepOutcome {
        let player_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        let has_horns = game.player(&player_id)
            .map(|p| p.has_skill(SkillId::Horns))
            .unwrap_or(false);
        let is_blitz = game.acting_player.player_action == Some(PlayerAction::Blitz);

        // Java: state.usingHorns = hasSkill(actingPlayer, Horns) && BLITZ == playerAction
        self.using_horns = has_horns && is_blitz;

        if self.using_horns {
            // Java: actingPlayer.markSkillUsed(skill) → add to used_skills
            let is_home = game.team_home.player(&player_id).is_some();
            if is_home {
                if let Some(p) = game.team_home.player_mut(&player_id) {
                    p.used_skills.insert(SkillId::Horns);
                }
            } else if let Some(p) = game.team_away.player_mut(&player_id) {
                p.used_skills.insert(SkillId::Horns);
            }

            let event = GameEvent::SkillUse {
                player_id: player_id.clone(),
                skill_id: SkillId::Horns as u16,
                used: true,
            };
            StepOutcome::next().with_event(event)
        } else {
            StepOutcome::next()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::Rules;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::types::FieldCoordinate;
    use ffb_model::enums::PlayerState;
    use ffb_model::enums::PS_STANDING;

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

    fn make_game(skills: Vec<SkillId>, action: PlayerAction) -> (Game, String) {
        let pid = "att".to_string();
        let mut home = test_team("home", 0);
        add_player(&mut home, &pid, 1, skills);
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.home_playing = true;
        game.acting_player.player_id = Some(pid.clone());
        game.acting_player.player_action = Some(action);
        game.field_model.set_player_coordinate(&pid, FieldCoordinate::new(5, 5));
        game.field_model.set_player_state(&pid, PlayerState::new(PS_STANDING).change_active(true));
        (game, pid)
    }

    #[test]
    fn no_acting_player_returns_next() {
        let mut home = test_team("home", 0);
        add_player(&mut home, "att", 1, vec![SkillId::Horns]);
        let mut game = Game::new(home, test_team("away", 0), Rules::Bb2025);
        game.acting_player.player_id = None;
        let outcome = StepHorns::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.events.is_empty());
    }

    #[test]
    fn no_horns_skill_returns_next_no_event() {
        let (mut game, _) = make_game(vec![], PlayerAction::Blitz);
        let outcome = StepHorns::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.events.is_empty());
    }

    #[test]
    fn horns_with_block_action_skips() {
        let (mut game, _) = make_game(vec![SkillId::Horns], PlayerAction::Block);
        let outcome = StepHorns::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.events.is_empty());
    }

    #[test]
    fn horns_with_blitz_emits_skill_used_event() {
        let (mut game, _) = make_game(vec![SkillId::Horns], PlayerAction::Blitz);
        let outcome = StepHorns::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.events.iter().any(|e| matches!(e, GameEvent::SkillUse { used: true, .. })));
    }

    #[test]
    fn horns_with_blitz_marks_skill_used() {
        let (mut game, pid) = make_game(vec![SkillId::Horns], PlayerAction::Blitz);
        StepHorns::new().start(&mut game, &mut GameRng::new(0));
        assert!(game.team_home.player(&pid).unwrap().used_skills.contains(&SkillId::Horns));
    }

    #[test]
    fn horns_with_blitz_sets_using_horns_true() {
        let (mut game, _) = make_game(vec![SkillId::Horns], PlayerAction::Blitz);
        let mut step = StepHorns::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(step.using_horns);
    }
}
