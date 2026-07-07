/// 1:1 translation of com.fumbbl.ffb.server.step.action.block.StepStab (COMMON rules)
/// and its BB2016/BB2020 hook com.fumbbl.ffb.server.skillbehaviour.bb2016.StabBehaviour.
///
/// Handles the Stab special action: the attacker rolls armor against the defender instead of
/// blocking normally.  Only runs when USING_STAB = true was set by a preceding step.
///
/// On stab:
///   - Roll 2d6 armor vs defender.armour
///   - If armor broken: place defender PRONE, roll 2d6 injury, populate InjuryResult
///   - Publish INJURY_RESULT, GOTO_LABEL_ON_SUCCESS
///
/// Needs GOTO_LABEL_ON_SUCCESS init parameter.
/// Expects USING_STAB parameter from a preceding step.
use ffb_model::enums::{ApothecaryMode, SkillId, PS_PRONE};
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_mechanics::mechanics::armor_broken;
use crate::action::Action;
use crate::injury::{InjuryContext, InjuryResult};
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

pub struct StepStab {
    /// Java: state.goToLabelOnSuccess — GOTO_LABEL_ON_SUCCESS init parameter.
    pub goto_label_on_success: String,
    /// Java: state.usingStab — set by USING_STAB parameter from a preceding step.
    pub using_stab: Option<bool>,
}

impl StepStab {
    pub fn new() -> Self {
        Self {
            goto_label_on_success: String::new(),
            using_stab: None,
        }
    }
}

impl Default for StepStab {
    fn default() -> Self { Self::new() }
}

impl Step for StepStab {
    fn id(&self) -> StepId { StepId::Stab }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnSuccess(v) => { self.goto_label_on_success = v.clone(); true }
            StepParameter::UsingStab(v)          => { self.using_stab = Some(*v); true }
            _ => false,
        }
    }
}

impl StepStab {
    /// Java: StabBehaviour(bb2016).handleExecuteStepHook
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let player_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        let has_stab = game.player(&player_id)
            .map(|p| p.has_skill(SkillId::Stab))
            .unwrap_or(false);

        // Java: if (hasSkill(actingPlayer, Stab) && usingStab != null && usingStab)
        if !has_stab || self.using_stab != Some(true) {
            return StepOutcome::next();
        }

        let defender_id = match game.defender_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        let defender_armour = game.player(&defender_id).map(|p| p.armour).unwrap_or(8);

        // Java: UtilServerInjury.handleInjury → InjuryTypeStab → armor roll + optional injury roll.
        let a1 = rng.d6();
        let a2 = rng.d6();
        let broke = armor_broken(defender_armour, [a1, a2], &[]);

        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        ctx.armor_roll = Some([a1, a2]);
        ctx.armor_broken = broke;

        if broke {
            // Java: UtilServerInjury.dropPlayer → place PRONE.
            let defender_state = game.field_model.player_state(&defender_id).unwrap_or_default();
            game.field_model.set_player_state(&defender_id, defender_state.change_base(PS_PRONE).change_active(false));

            let i1 = rng.d6();
            let i2 = rng.d6();
            ctx.injury_roll = Some([i1, i2]);
        }

        let injury_result = InjuryResult { injury_context: ctx, knocked_out: false, rip: false, already_reported: false, pre_regeneration: true };

        StepOutcome::goto(&self.goto_label_on_success)
            .publish(StepParameter::InjuryResult(Box::new(injury_result)))
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
        armour: i32,
    ) {
        team.players.push(ffb_model::model::player::Player {
            id: id.into(),
            name: id.into(),
            nr,
            position_id: "pos".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4,
            armour,
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
            is_big_guy: false,
            ..Default::default()
        });
    }

    fn make_stab_game(attacker_skills: Vec<SkillId>, defender_armour: i32) -> Game {
        let mut home = test_team("home", 0);
        let mut away = test_team("away", 0);
        add_player(&mut home, "att", 1, attacker_skills, 8);
        add_player(&mut away, "def", 2, vec![], defender_armour);
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

    fn seed_for_2d6(target_sum: i32) -> u64 {
        for s in 0u64..100_000 {
            let mut rng = GameRng::new(s);
            if rng.d6() + rng.d6() == target_sum { return s; }
        }
        panic!("no seed for 2d6={}", target_sum);
    }

    #[test]
    fn not_using_stab_returns_next() {
        let mut game = make_stab_game(vec![SkillId::Stab], 8);
        let mut step = StepStab::new();
        step.goto_label_on_success = "STAB".into();
        // using_stab = None (not set) → NEXT_STEP
        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
    }

    #[test]
    fn using_stab_false_returns_next() {
        let mut game = make_stab_game(vec![SkillId::Stab], 8);
        let mut step = StepStab::new();
        step.goto_label_on_success = "STAB".into();
        step.using_stab = Some(false);
        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
    }

    #[test]
    fn no_stab_skill_returns_next() {
        let mut game = make_stab_game(vec![], 8);
        let mut step = StepStab::new();
        step.goto_label_on_success = "STAB".into();
        step.using_stab = Some(true);
        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
    }

    #[test]
    fn no_acting_player_returns_next() {
        let mut game = make_stab_game(vec![SkillId::Stab], 8);
        game.acting_player.player_id = None;
        let mut step = StepStab::new();
        step.using_stab = Some(true);
        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
    }

    #[test]
    fn armor_not_broken_goes_to_label_without_defender_prone() {
        // defender armour = 10, need sum > 10 → need 11 or 12. Use sum=8 which doesn't break.
        let seed = seed_for_2d6(8); // 8 <= 10 → not broken
        let mut game = make_stab_game(vec![SkillId::Stab], 10);
        let mut step = StepStab::new();
        step.goto_label_on_success = "STAB_OK".into();
        step.using_stab = Some(true);
        let outcome = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(outcome.action, StepAction::GotoLabel);
        assert_eq!(outcome.goto_label.as_deref(), Some("STAB_OK"));
        // Defender not prone (armor not broken)
        assert_ne!(game.field_model.player_state("def").unwrap().base(), PS_PRONE);
        // InjuryResult published with armor not broken, no injury roll
        let has_injury_result = outcome.published.iter().any(|p| {
            if let StepParameter::InjuryResult(r) = p {
                !r.injury_context().is_armor_broken() && r.injury_context().get_injury_roll().is_none()
            } else { false }
        });
        assert!(has_injury_result);
    }

    #[test]
    fn armor_broken_places_defender_prone_and_rolls_injury() {
        // defender armour = 2, need sum > 2 → almost any roll. Use sum=12.
        let seed = seed_for_2d6(12); // 12 > 2 → breaks
        let mut game = make_stab_game(vec![SkillId::Stab], 2);
        let mut step = StepStab::new();
        step.goto_label_on_success = "STAB_HURT".into();
        step.using_stab = Some(true);
        let outcome = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(outcome.action, StepAction::GotoLabel);
        // Defender should be PRONE
        assert_eq!(game.field_model.player_state("def").unwrap().base(), PS_PRONE);
        // InjuryResult published with armor broken + injury roll present
        let injury_ok = outcome.published.iter().any(|p| {
            if let StepParameter::InjuryResult(r) = p {
                r.injury_context().is_armor_broken() && r.injury_context().get_injury_roll().is_some()
            } else { false }
        });
        assert!(injury_ok);
    }

    #[test]
    fn set_parameter_stores_fields() {
        let mut step = StepStab::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnSuccess("G".into())));
        assert!(step.set_parameter(&StepParameter::UsingStab(true)));
        assert_eq!(step.goto_label_on_success, "G");
        assert_eq!(step.using_stab, Some(true));
        assert!(!step.set_parameter(&StepParameter::EndPlayerAction(true)));
    }
}
