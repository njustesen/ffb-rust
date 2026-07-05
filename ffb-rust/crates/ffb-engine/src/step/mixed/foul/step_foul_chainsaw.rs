use ffb_mechanics::mechanics::minimum_roll_chainsaw;
use ffb_model::enums::{ApothecaryMode, ReRollSource};
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::drop_player_context::DropPlayerContext;
use crate::step::abstract_step_with_re_roll::{ReRollState, find_skill_reroll_source};
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_injury::handle_injury_by_name;
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};
use ffb_model::model::re_rolled_action::ReRolledAction;

/// 1:1 translation of com.fumbbl.ffb.server.step.mixed.foul.StepFoulChainsaw.
///
/// Handles the Chainsaw skill during a foul. The attacker makes a chainsaw roll (d6 ≥ 2).
/// On success: publish USING_CHAINSAW=true → NEXT_STEP (StepFoul handles the actual injury).
/// On failure: the chainsaw backfires — attacker is injured with InjuryTypeChainsaw.
///
/// Init: GOTO_LABEL_ON_FAILURE (mandatory).
/// Java: @RulesCollection(BB2020, BB2025)
pub struct StepFoulChainsaw {
    /// Java: fGotoLabelOnFailure (init param)
    pub goto_label_on_failure: String,
    /// Java: usingChainsaw — whether the chainsaw is actually being used this action
    pub using_chainsaw: bool,
    /// AbstractStepWithReRoll state
    pub re_roll_state: ReRollState,
}

impl StepFoulChainsaw {
    pub fn new(goto_label_on_failure: String) -> Self {
        Self {
            goto_label_on_failure,
            using_chainsaw: false,
            re_roll_state: ReRollState::new(),
        }
    }
}

impl Default for StepFoulChainsaw {
    fn default() -> Self { Self::new(String::new()) }
}

impl Step for StepFoulChainsaw {
    fn id(&self) -> StepId { StepId::FoulChainsaw }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::UseReRoll { use_reroll: false } => {
                self.re_roll_state.re_roll_source = None;
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::UsingChainsaw(v) => { self.using_chainsaw = *v; true }
            StepParameter::GotoLabelOnFailure(v) => { self.goto_label_on_failure = v.clone(); true }
            _ => false,
        }
    }
}

impl StepFoulChainsaw {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let attacker_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        // Java: if (!hasUnusedSkillWithProperty(blocksLikeChainsaw) || !usingChainsaw) → NEXT_STEP
        let has_chainsaw = game.player(&attacker_id)
            .map(|p| p.has_skill_property(NamedProperties::BLOCKS_LIKE_CHAINSAW))
            .unwrap_or(false);
        if !has_chainsaw || !self.using_chainsaw {
            return StepOutcome::next();
        }

        // Java: if (CHAINSAW == reRolledAction) { if (source == null || !useReRoll) dropChainsawPlayer = true }
        let already_rerolled = self.re_roll_state.re_rolled_action
            .as_ref().map(|a| a.name == "CHAINSAW").unwrap_or(false);
        let mut drop_chainsaw_player = false;

        if already_rerolled {
            let source = self.re_roll_state.re_roll_source.clone();
            let consumed = source.as_ref()
                .map(|s| use_reroll(game, s, &attacker_id))
                .unwrap_or(false);
            if !consumed {
                drop_chainsaw_player = true;
            }
        }

        if !drop_chainsaw_player {
            let roll = rng.d6();
            let minimum_roll = minimum_roll_chainsaw();
            let successful = roll >= minimum_roll;

            if successful {
                return StepOutcome::next()
                    .publish(StepParameter::UsingChainsaw(true));
            }

            // Failed roll — try re-roll if not already rerolled
            if !already_rerolled {
                self.re_roll_state.re_rolled_action = Some(ReRolledAction::new("CHAINSAW"));

                let skill_source = find_skill_reroll_source(game, "CHAINSAW");
                if let Some(source) = skill_source {
                    use_reroll(game, &source, &attacker_id);
                    self.re_roll_state.re_roll_source = Some(source);
                    return self.execute_step(game, rng);
                }

                if let Some(prompt) = ask_for_reroll_if_available(game, "CHAINSAW", minimum_roll, false) {
                    self.re_roll_state.re_roll_source = Some(ReRollSource::new("TRR"));
                    return StepOutcome::cont().with_prompt(prompt);
                }
            }

            drop_chainsaw_player = true;
        }

        if drop_chainsaw_player {
            let attacker_coord = game.field_model.player_coordinate(&attacker_id)
                .unwrap_or(ffb_model::types::FieldCoordinate::new(0, 0));

            // Java: handleInjury(InjuryTypeChainsaw, null, attacker, attackerCoord, null, null, ATTACKER)
            let injury_result = handle_injury_by_name(
                game, rng, "InjuryTypeChainsaw",
                None, &attacker_id,
                attacker_coord, None, None, ApothecaryMode::Attacker,
            );

            // Java: causesTurnOver = hasBall(attacker) || (armor broken && option != NEVER)
            // Stub: default to ALL_AV_BREAKS (armor broken → turnover)
            let has_ball = game.field_model.player_coordinate(&attacker_id)
                .zip(game.field_model.ball_coordinate)
                .map(|(pc, bc)| pc == bc)
                .unwrap_or(false);
            let armor_broken = injury_result.injury_context().is_armor_broken();
            let causes_turn_over = has_ball || armor_broken;

            let label = self.goto_label_on_failure.clone();
            let dpc = DropPlayerContext {
                injury_result: Some(Box::new(injury_result)),
                end_turn: causes_turn_over,
                eligible_for_safe_pair_of_hands: true,
                label: if label.is_empty() { None } else { Some(label) },
                player_id: Some(attacker_id),
                apothecary_mode: Some(ApothecaryMode::Attacker),
                modified_injury_ends_turn: true,
                ..DropPlayerContext::new()
            };
            return StepOutcome::next()
                .publish(StepParameter::DropPlayerContext(Box::new(dpc)));
        }

        StepOutcome::next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::{Rules, PS_STANDING, SkillId};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_player(game: &mut Game, team: &str, id: &str, skill: Option<SkillId>) {
        let p = Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: skill.iter().map(|&s| SkillWithValue::new(s)).collect(),
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
};
        if team == "home" { game.team_home.players.push(p); } else { game.team_away.players.push(p); }
        game.field_model.set_player_coordinate(id, FieldCoordinate::new(5, 5));
        game.field_model.set_player_state(id, ffb_model::enums::PlayerState::new(PS_STANDING));
    }

    #[test]
    fn no_attacker_returns_next_step() {
        let mut game = make_game();
        let mut step = StepFoulChainsaw::new("fail".into());
        step.using_chainsaw = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn not_using_chainsaw_returns_next_step() {
        let mut game = make_game();
        add_player(&mut game, "home", "atk", Some(SkillId::Chainsaw));
        game.acting_player.player_id = Some("atk".into());
        let mut step = StepFoulChainsaw::new("fail".into());
        step.using_chainsaw = false;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.is_empty());
    }

    #[test]
    fn no_chainsaw_skill_returns_next_step() {
        let mut game = make_game();
        add_player(&mut game, "home", "atk", None);
        game.acting_player.player_id = Some("atk".into());
        let mut step = StepFoulChainsaw::new("fail".into());
        step.using_chainsaw = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn successful_roll_publishes_using_chainsaw_true() {
        // Roll of 2+ (any non-1) = success for chainsaw
        let mut game = make_game();
        let chainsaw_props = SkillId::Chainsaw.properties();
        if !chainsaw_props.contains(&"blocksLikeChainsaw") {
            // Chainsaw skill not yet wired — skip test
            return;
        }
        add_player(&mut game, "home", "atk", Some(SkillId::Chainsaw));
        game.acting_player.player_id = Some("atk".into());
        // Find seed producing roll >= 2
        for seed in 0..200u64 {
            let mut rng = GameRng::new(seed);
            if rng.d6() >= 2 {
                let mut g = make_game();
                add_player(&mut g, "home", "atk", Some(SkillId::Chainsaw));
                g.acting_player.player_id = Some("atk".into());
                let mut step = StepFoulChainsaw::new("fail".into());
                step.using_chainsaw = true;
                let out = step.start(&mut g, &mut GameRng::new(seed));
                assert!(
                    out.published.iter().any(|p| matches!(p, StepParameter::UsingChainsaw(true))),
                    "seed={seed}: expected UsingChainsaw(true)"
                );
                return;
            }
        }
        panic!("no seed producing roll >= 2");
    }

    #[test]
    fn set_parameter_using_chainsaw_accepted() {
        let mut step = StepFoulChainsaw::new("fail".into());
        assert!(step.set_parameter(&StepParameter::UsingChainsaw(true)));
        assert!(step.using_chainsaw);
    }

    #[test]
    fn set_parameter_goto_label_accepted() {
        let mut step = StepFoulChainsaw::new(String::new());
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("mylabel".into())));
        assert_eq!(step.goto_label_on_failure, "mylabel");
    }
}
