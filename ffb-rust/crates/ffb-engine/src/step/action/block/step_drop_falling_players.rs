/// 1:1 translation of com.fumbbl.ffb.server.step.action.block.StepDropFallingPlayers (COMMON)
/// and its BB2016/BB2020 hook com.fumbbl.ffb.server.skillbehaviour.bb2016.PilingOnBehaviour.
///
/// Drops any FALLING players (attacker and/or defender) and rolls armor+injury for them.
/// Also handles the deprecated PilingOn skill prompt (BB2016/BB2020 only).
///
/// Random agent always declines PilingOn, so the simplified path is always taken:
///   1. If defender FALLING: place PRONE, roll armor+injury → publish INJURY_RESULT
///   2. If attacker FALLING: place PRONE, roll armor+injury → publish INJURY_RESULT, END_TURN
///
/// PilingOn full logic (team re-roll, dialog, etc.) is stubbed — conditions for showing the
/// PilingOn dialog involve NamedProperties.canPileOnOpponent which is not yet implemented.
///
/// Expects OLD_DEFENDER_STATE parameter from a preceding step.
use ffb_model::dialog::dialog_id::DialogId;
use ffb_model::enums::{ApothecaryMode, PS_FALLING, PS_PRONE};
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use ffb_mechanics::mechanics::armor_broken;
use crate::action::Action;
use crate::injury::{InjuryContext, InjuryResult};
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::util::util_server_dialog::UtilServerDialog;

pub struct StepDropFallingPlayers {
    /// Java: state.injuryResultDefender — populated after defender is dropped.
    pub injury_result_defender: Option<Box<InjuryResult>>,
    /// Java: state.usingPilingOn — None = not asked, Some = decided.
    pub using_piling_on: Option<bool>,
    /// Java: state.oldDefenderState — defender state before the block result was applied.
    pub old_defender_state: Option<ffb_model::enums::PlayerState>,
}

impl StepDropFallingPlayers {
    pub fn new() -> Self {
        Self {
            injury_result_defender: None,
            using_piling_on: None,
            old_defender_state: None,
        }
    }
}

impl Default for StepDropFallingPlayers {
    fn default() -> Self { Self::new() }
}

impl Step for StepDropFallingPlayers {
    fn id(&self) -> StepId { StepId::DropFallingPlayers }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: handleCommandHook — CLIENT_USE_SKILL sets usingPilingOn.
        if let Action::UseSkill { use_skill, .. } = action {
            self.using_piling_on = Some(*use_skill);
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

impl StepDropFallingPlayers {
    /// Java: PilingOnBehaviour.handleExecuteStepHook (simplified: PilingOn always declined).
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let player_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };
        let defender_id = game.defender_id.clone();

        // Java: unroot FALLING players before processing.
        // (Rooted FALLING → just unroot; handled as normal prone below.)

        let defender_state = defender_id.as_deref()
            .and_then(|id| game.field_model.player_state(id));
        let defender_is_falling = defender_state.map(|s| s.base() == PS_FALLING).unwrap_or(false);

        let defender_coord = defender_id.as_deref()
            .and_then(|id| game.field_model.player_coordinate(id));

        let mut outcome = StepOutcome::next();

        // Java: if (usingPilingOn != null) { re-roll with PilingOn } else if (defenderFalling) { drop }
        if self.using_piling_on.is_some() {
            // PilingOn used or declined after dialog.
            if let Some(ir) = &self.injury_result_defender {
                outcome = outcome.publish(StepParameter::InjuryResult(ir.clone()));
            }
            if let Some(v) = self.using_piling_on {
                outcome = outcome.publish(StepParameter::UsingPilingOn(v));
            }
        } else if defender_is_falling && defender_coord.is_some() {
            // Java: dropPlayer(defender) + handleInjury
            let did = defender_id.as_deref().unwrap();
            let defender_state_full = game.field_model.player_state(did).unwrap_or_default();
            game.field_model.set_player_state(did, defender_state_full.change_base(PS_PRONE).change_active(false));

            let av = game.player(did).map(|p| p.armour).unwrap_or(8);
            let a1 = rng.d6();
            let a2 = rng.d6();
            let broke = armor_broken(av, [a1, a2], &[]);

            let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
            ctx.armor_roll = Some([a1, a2]);
            ctx.armor_broken = broke;
            if broke {
                let i1 = rng.d6();
                let i2 = rng.d6();
                ctx.injury_roll = Some([i1, i2]);
            }

            let ir = Box::new(InjuryResult { injury_context: ctx, knocked_out: false, rip: false, already_reported: false, pre_regeneration: true });
            self.injury_result_defender = Some(ir.clone());

            // Java: PilingOnBehaviour.handleExecuteStepHook — check if attacker has canPileOnOpponent.
            let piling_on_eligible = game.acting_player.player_id.as_deref()
                .and_then(|id| game.player(id))
                .map(|p| p.has_skill_property(NamedProperties::CAN_PILE_ON_OPPONENT))
                .unwrap_or(false);

            if piling_on_eligible && self.using_piling_on.is_none() {
                // Attacker has PilingOn and hasn't decided yet → show dialog.
                // Java: UtilServerDialog.showDialog(gameState, new DialogPilingOnParameter(...), false)
                UtilServerDialog::show_dialog(game, DialogId::PILING_ON, false);
                // Continue waiting for Action::UseSkill { use_skill } response.
                return StepOutcome::cont();
            }
            // Not eligible or player already decided → publish result.
            outcome = outcome.publish(StepParameter::InjuryResult(ir));
        }

        // Java: if (defenderFalling && defender is own team && !oldDefenderState.isProne) → END_TURN
        let defender_is_own_team = {
            let did = defender_id.as_deref().unwrap_or("");
            game.team_home.player(&player_id).is_some() && game.team_home.player(did).is_some()
                || game.team_away.player(&player_id).is_some() && game.team_away.player(did).is_some()
        };
        if defender_is_falling
            && defender_is_own_team
            && self.old_defender_state
                .map(|s| !s.is_prone_or_stunned())
                .unwrap_or(false)
        {
            outcome = outcome.publish(StepParameter::EndTurn(true));
        }

        // Java: handle attacker FALLING.
        let attacker_coord = game.field_model.player_coordinate(&player_id);
        let attacker_state = game.field_model.player_state(&player_id);
        let attacker_is_falling = attacker_state.map(|s| s.base() == PS_FALLING).unwrap_or(false);

        if attacker_is_falling && attacker_coord.is_some() {
            // Java: dropPlayer(attacker) + handleInjury
            let attacker_state_full = game.field_model.player_state(&player_id).unwrap_or_default();
            game.field_model.set_player_state(&player_id, attacker_state_full.change_base(PS_PRONE).change_active(false));

            let av = game.player(&player_id).map(|p| p.armour).unwrap_or(8);
            let a1 = rng.d6();
            let a2 = rng.d6();
            let broke = armor_broken(av, [a1, a2], &[]);

            let mut ctx = InjuryContext::new(ApothecaryMode::Attacker);
            ctx.armor_roll = Some([a1, a2]);
            ctx.armor_broken = broke;
            if broke {
                let i1 = rng.d6();
                let i2 = rng.d6();
                ctx.injury_roll = Some([i1, i2]);
            }
            let ir = Box::new(InjuryResult { injury_context: ctx, knocked_out: false, rip: false, already_reported: false, pre_regeneration: true });
            outcome = outcome
                .publish(StepParameter::EndTurn(true))
                .publish(StepParameter::InjuryResult(ir));
        }

        outcome
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{ApothecaryMode, Rules, PS_STANDING};
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::enums::{PlayerState, SkillId};
    use ffb_model::types::FieldCoordinate;

    fn make_game_with_falling(attacker_falling: bool, defender_falling: bool) -> Game {
        let mut home = test_team("home", 0);
        let mut away = test_team("away", 0);
        let make_player = |id: &str, nr: i32| ffb_model::model::player::Player {
            id: id.into(), name: id.into(), nr, position_id: "pos".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![],
            extra_skills: vec![], temporary_skills: vec![], used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };
        home.players.push(make_player("att", 1));
        away.players.push(make_player("def", 2));
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.home_playing = true;
        game.acting_player.player_id = Some("att".into());
        game.defender_id = Some("def".into());
        game.field_model.set_player_coordinate("att", FieldCoordinate::new(5, 5));
        game.field_model.set_player_coordinate("def", FieldCoordinate::new(6, 5));
        let att_ps = if attacker_falling { PlayerState::new(PS_FALLING).change_active(true) }
                     else { PlayerState::new(PS_STANDING).change_active(true) };
        let def_ps = if defender_falling { PlayerState::new(PS_FALLING) }
                     else { PlayerState::new(PS_STANDING) };
        game.field_model.set_player_state("att", att_ps);
        game.field_model.set_player_state("def", def_ps);
        game
    }

    #[test]
    fn neither_falling_returns_next_no_pubs() {
        let mut game = make_game_with_falling(false, false);
        let outcome = StepDropFallingPlayers::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.published.is_empty());
    }

    #[test]
    fn defender_falling_placed_prone_and_injury_result_published() {
        let mut game = make_game_with_falling(false, true);
        let outcome = StepDropFallingPlayers::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert_eq!(game.field_model.player_state("def").unwrap().base(), PS_PRONE);
        assert!(outcome.published.iter().any(|p| matches!(p, StepParameter::InjuryResult(_))));
    }

    #[test]
    fn attacker_falling_placed_prone_end_turn_published() {
        let mut game = make_game_with_falling(true, false);
        let outcome = StepDropFallingPlayers::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert_eq!(game.field_model.player_state("att").unwrap().base(), PS_PRONE);
        assert!(outcome.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
        assert!(outcome.published.iter().any(|p| matches!(p, StepParameter::InjuryResult(_))));
    }

    #[test]
    fn injury_result_has_correct_apothecary_mode_for_defender() {
        let mut game = make_game_with_falling(false, true);
        let outcome = StepDropFallingPlayers::new().start(&mut game, &mut GameRng::new(0));
        let defender_result = outcome.published.iter().find_map(|p| {
            if let StepParameter::InjuryResult(r) = p {
                if r.injury_context().get_apothecary_mode() == ApothecaryMode::Defender {
                    return Some(r.clone());
                }
            }
            None
        });
        assert!(defender_result.is_some(), "should have a Defender InjuryResult");
    }

    #[test]
    fn set_parameter_stores_old_defender_state() {
        let mut step = StepDropFallingPlayers::new();
        assert!(step.set_parameter(&StepParameter::OldDefenderState(PlayerState::new(PS_STANDING))));
        assert!(step.old_defender_state.is_some());
        assert!(!step.set_parameter(&StepParameter::EndPlayerAction(true)));
    }

    fn add_piling_on_skill(game: &mut Game, player_id: &str) {
        // Add PilingOn skill (canPileOnOpponent property) to the attacker.
        if let Some(p) = game.team_home.player_mut(player_id) {
            p.extra_skills.push(SkillWithValue::new(SkillId::PilingOn));
        }
    }

    #[test]
    fn defender_falling_piling_on_eligible_returns_continue() {
        // When attacker has PilingOn and defender is falling, step returns Continue.
        let mut game = make_game_with_falling(false, true);
        add_piling_on_skill(&mut game, "att");
        let mut step = StepDropFallingPlayers::new();
        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::Continue);
        // InjuryResult not yet published — waiting for dialog response.
        assert!(!outcome.published.iter().any(|p| matches!(p, StepParameter::InjuryResult(_))));
    }

    #[test]
    fn piling_on_declined_publishes_injury_result() {
        // When player declines PilingOn (UseSkill false), InjuryResult is published.
        let mut game = make_game_with_falling(false, true);
        add_piling_on_skill(&mut game, "att");
        let mut step = StepDropFallingPlayers::new();
        // First call: returns Continue (dialog shown).
        step.start(&mut game, &mut GameRng::new(0));
        // Player declines PilingOn.
        let outcome = step.handle_command(
            &Action::UseSkill { skill_id: SkillId::PilingOn, use_skill: false },
            &mut game, &mut GameRng::new(0),
        );
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.published.iter().any(|p| matches!(p, StepParameter::InjuryResult(_))));
    }

    #[test]
    fn piling_on_accepted_publishes_using_piling_on_true() {
        // When player accepts PilingOn, UsingPilingOn(true) is published.
        let mut game = make_game_with_falling(false, true);
        add_piling_on_skill(&mut game, "att");
        let mut step = StepDropFallingPlayers::new();
        step.start(&mut game, &mut GameRng::new(0));
        let outcome = step.handle_command(
            &Action::UseSkill { skill_id: SkillId::PilingOn, use_skill: true },
            &mut game, &mut GameRng::new(0),
        );
        assert!(outcome.published.iter().any(|p| matches!(p, StepParameter::UsingPilingOn(true))));
    }

    #[test]
    fn no_piling_on_skill_publishes_injury_result_immediately() {
        // Without PilingOn skill, no dialog — InjuryResult published on first call.
        let mut game = make_game_with_falling(false, true);
        // attacker has no PilingOn skill (default test_team players)
        let outcome = StepDropFallingPlayers::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.published.iter().any(|p| matches!(p, StepParameter::InjuryResult(_))));
    }
}
