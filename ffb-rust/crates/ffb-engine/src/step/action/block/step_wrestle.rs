/// 1:1 translation of com.fumbbl.ffb.server.step.action.block.StepWrestle (COMMON rules)
/// and its BB2016/BB2020/BB2025 hooks com.fumbbl.ffb.server.skillbehaviour.*.WrestleBehaviour.
///
/// Wrestle lets the attacker or defender convert a Both-Down result into both players going prone.
/// Two sequential dialogs: attacker first, then defender (if attacker declined).
/// Random agent always declines → neither uses Wrestle → NEXT_STEP.
///
/// When either player uses it: both attacker and defender are placed prone (simplified drop stub).
///
/// Expects OLD_DEFENDER_STATE parameter from a preceding step.
use ffb_model::enums::{PlayerAction, SkillId, PS_PRONE};
use ffb_model::enums::PlayerState;
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

pub struct StepWrestle {
    /// Java: state.usingWrestleAttacker — None = not asked, Some = answered.
    pub using_wrestle_attacker: Option<bool>,
    /// Java: state.usingWrestleDefender — None = not asked, Some = answered.
    pub using_wrestle_defender: Option<bool>,
    /// Java: state.oldDefenderState — defender state before the block result was applied.
    pub old_defender_state: Option<PlayerState>,
}

impl StepWrestle {
    pub fn new() -> Self {
        Self {
            using_wrestle_attacker: None,
            using_wrestle_defender: None,
            old_defender_state: None,
        }
    }
}

impl Default for StepWrestle {
    fn default() -> Self { Self::new() }
}

impl Step for StepWrestle {
    fn id(&self) -> StepId { StepId::Wrestle }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: handleCommandHook — attacker answered first, then defender.
        if let Action::UseSkill { use_skill, .. } = action {
            if self.using_wrestle_attacker.is_none() {
                self.using_wrestle_attacker = Some(*use_skill);
            } else {
                self.using_wrestle_defender = Some(*use_skill);
            }
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::OldDefenderState(v) => { self.old_defender_state = Some(*v); true }
            _ => false,
        }
    }
}

impl StepWrestle {
    /// Java: WrestleBehaviour.handleExecuteStepHook (all editions identical logic).
    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        let player_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        // Java: askAttackerForWrestleUse: if attacker has Wrestle and is not Rooted → CONTINUE.
        // Stub: random agent always declines → treat None as false.
        if self.using_wrestle_attacker.is_none() {
            let attacker_has_wrestle = game.player(&player_id)
                .map(|p| p.has_skill(SkillId::Wrestle))
                .unwrap_or(false);
            if attacker_has_wrestle {
                // Show dialog in production — random agent never responds → stub as declined.
                self.using_wrestle_attacker = Some(false);
            } else {
                self.using_wrestle_attacker = Some(false);
            }
        }

        // Java: askDefenderForWrestleUse: if defender has Wrestle, not Rooted, not blitz-cancelled.
        if self.using_wrestle_defender.is_none() {
            let defender_id = game.defender_id.clone();
            let defender_has_wrestle = defender_id.as_deref()
                .and_then(|id| game.player(id))
                .map(|p| p.has_skill(SkillId::Wrestle))
                .unwrap_or(false);
            let is_blitz = game.acting_player.player_action == Some(PlayerAction::Blitz);
            // Java: cancelsSkill(attacker, Wrestle) — Juggernaut or similar. Stub: always false.
            let attacker_cancels_wrestle = false;
            let attacker_declined = self.using_wrestle_attacker == Some(false);
            if attacker_declined && defender_has_wrestle && !(is_blitz && attacker_cancels_wrestle) {
                // Show dialog for defender — random agent declines.
                self.using_wrestle_defender = Some(false);
            } else {
                self.using_wrestle_defender = Some(false);
            }
        }

        // Java: performWrestle
        self.perform_wrestle(game, &player_id)
    }

    fn perform_wrestle(&self, game: &mut Game, player_id: &str) -> StepOutcome {
        let using_attacker = self.using_wrestle_attacker.unwrap_or(false);
        let using_defender = self.using_wrestle_defender.unwrap_or(false);
        let defender_id = game.defender_id.clone();

        let mut events = Vec::new();
        let skill_num = SkillId::Wrestle as u16;

        if using_attacker {
            events.push(GameEvent::SkillUse { player_id: player_id.to_string(), skill_id: skill_num, used: true });
        } else if using_defender {
            if let Some(did) = &defender_id {
                events.push(GameEvent::SkillUse { player_id: did.clone(), skill_id: skill_num, used: true });
            }
        } else {
            // Java: if either player has Wrestle skill, emit declined report.
            let attacker_has = game.player(player_id).map(|p| p.has_skill(SkillId::Wrestle)).unwrap_or(false);
            let defender_has = defender_id.as_deref()
                .and_then(|id| game.player(id))
                .map(|p| p.has_skill(SkillId::Wrestle))
                .unwrap_or(false);
            if attacker_has || defender_has {
                events.push(GameEvent::SkillUse { player_id: player_id.to_string(), skill_id: skill_num, used: false });
            }
        }

        if using_attacker || using_defender {
            // Java: UtilServerInjury.dropPlayer → place both PRONE.
            // Simplified stub: set both to PRONE, deactivate.
            let attacker_state = game.field_model.player_state(player_id)
                .unwrap_or_default();
            game.field_model.set_player_state(player_id, attacker_state.change_base(PS_PRONE).change_active(false));

            if let Some(did) = &defender_id {
                let defender_state = game.field_model.player_state(did).unwrap_or_default();
                game.field_model.set_player_state(did, defender_state.change_base(PS_PRONE).change_active(false));
            }
        }

        let mut outcome = StepOutcome::next();
        for e in events {
            outcome = outcome.with_event(e);
        }
        outcome
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
        });
    }

    fn make_game(attacker_skills: Vec<SkillId>, defender_skills: Vec<SkillId>) -> Game {
        let mut home = test_team("home", 0);
        let mut away = test_team("away", 0);
        add_player(&mut home, "att", 1, attacker_skills);
        add_player(&mut away, "def", 2, defender_skills);
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.home_playing = true;
        game.acting_player.player_id = Some("att".into());
        game.acting_player.player_action = Some(PlayerAction::Block);
        game.defender_id = Some("def".into());
        game.field_model.set_player_coordinate("att", FieldCoordinate::new(5, 5));
        game.field_model.set_player_coordinate("def", FieldCoordinate::new(6, 5));
        game.field_model.set_player_state("att", PlayerState::new(PS_STANDING).change_active(true));
        game.field_model.set_player_state("def", PlayerState::new(PS_STANDING));
        game
    }

    #[test]
    fn no_acting_player_returns_next() {
        let mut game = make_game(vec![], vec![]);
        game.acting_player.player_id = None;
        let outcome = StepWrestle::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
    }

    #[test]
    fn neither_has_wrestle_returns_next_no_events() {
        let mut game = make_game(vec![], vec![]);
        let outcome = StepWrestle::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.events.is_empty());
    }

    #[test]
    fn attacker_has_wrestle_emits_declined_event() {
        let mut game = make_game(vec![SkillId::Wrestle], vec![]);
        let outcome = StepWrestle::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.events.iter().any(|e| matches!(e, GameEvent::SkillUse { used: false, .. })));
    }

    #[test]
    fn defender_has_wrestle_emits_declined_event() {
        let mut game = make_game(vec![], vec![SkillId::Wrestle]);
        let outcome = StepWrestle::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.events.iter().any(|e| matches!(e, GameEvent::SkillUse { used: false, .. })));
    }

    #[test]
    fn attacker_uses_wrestle_drops_both_prone() {
        let mut game = make_game(vec![SkillId::Wrestle], vec![]);
        let mut step = StepWrestle::new();
        step.using_wrestle_attacker = Some(true);
        step.using_wrestle_defender = Some(false);
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.field_model.player_state("att").unwrap().base(), PS_PRONE);
        assert_eq!(game.field_model.player_state("def").unwrap().base(), PS_PRONE);
    }

    #[test]
    fn defender_uses_wrestle_drops_both_prone() {
        let mut game = make_game(vec![], vec![SkillId::Wrestle]);
        let mut step = StepWrestle::new();
        step.using_wrestle_attacker = Some(false);
        step.using_wrestle_defender = Some(true);
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.field_model.player_state("att").unwrap().base(), PS_PRONE);
        assert_eq!(game.field_model.player_state("def").unwrap().base(), PS_PRONE);
    }

    #[test]
    fn attacker_uses_wrestle_emits_skill_used_event() {
        let mut game = make_game(vec![SkillId::Wrestle], vec![]);
        let mut step = StepWrestle::new();
        step.using_wrestle_attacker = Some(true);
        step.using_wrestle_defender = Some(false);
        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert!(outcome.events.iter().any(|e| matches!(e, GameEvent::SkillUse { used: true, .. })));
    }

    #[test]
    fn set_parameter_stores_old_defender_state() {
        let mut step = StepWrestle::new();
        assert!(step.set_parameter(&StepParameter::OldDefenderState(PlayerState::new(PS_STANDING))));
        assert!(step.old_defender_state.is_some());
        assert!(!step.set_parameter(&StepParameter::EndPlayerAction(true)));
    }
}
