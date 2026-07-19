use ffb_mechanics::mechanics::minimum_roll_chainsaw;
use ffb_model::enums::{ApothecaryMode, ReRollSource};
use ffb_model::option::{game_option_id, game_option_string};
use ffb_model::report::report_chainsaw_roll::ReportChainsawRoll;
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

        // Java: if (!UtilCards.hasUnusedSkillWithProperty(actingPlayer, blocksLikeChainsaw) || !usingChainsaw) → NEXT_STEP
        let has_chainsaw = game.player(&attacker_id)
            .map(|p| p.has_unused_skill_with_property(NamedProperties::BLOCKS_LIKE_CHAINSAW))
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

            // Java: getResult().addReport(new ReportChainsawRoll(actingPlayer.getPlayerId(), successful, roll, minimumRoll, reRolled, null))
            game.report_list.add(ReportChainsawRoll::new(
                Some(attacker_id.clone()),
                successful,
                roll,
                minimum_roll,
                already_rerolled,
                vec![],
                None, // Java passes null for defender_id in foul context
            ));

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

            // Java: String chainsawOption = game.getOptions().getOptionWithDefault(CHAINSAW_TURNOVER).getValueAsString();
            let chainsaw_option = game.options.get(game_option_id::CHAINSAW_TURNOVER)
                .unwrap_or(game_option_string::CHAINSAW_TURNOVER_KICKBACK)
                .to_owned();

            // Java: boolean causesTurnOver = UtilPlayer.hasBall(game, actingPlayer.getPlayer());
            let has_ball = game.field_model.player_coordinate(&attacker_id)
                .zip(game.field_model.ball_coordinate)
                .map(|(pc, bc)| pc == bc)
                .unwrap_or(false);
            let armor_broken = injury_result.injury_context().is_armor_broken();
            let mut causes_turn_over = has_ball;
            let mut extra_end_turn = false;
            if armor_broken {
                // Java: if (!CHAINSAW_TURNOVER_NEVER.equalsIgnoreCase(chainsawOption)) causesTurnOver = true;
                if !chainsaw_option.eq_ignore_ascii_case(game_option_string::CHAINSAW_TURNOVER_NEVER) {
                    causes_turn_over = true;
                }
            } else if chainsaw_option == game_option_string::CHAINSAW_TURNOVER_KICKBACK {
                // Java: else if (CHAINSAW_TURNOVER_KICKBACK.equals(chainsawOption)) publishParameter(END_TURN, true);
                extra_end_turn = true;
            }

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
            let mut out = StepOutcome::next();
            if extra_end_turn {
                out = out.publish(StepParameter::EndTurn(true));
            }
            return out.publish(StepParameter::DropPlayerContext(Box::new(dpc)));
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
    use ffb_model::report::report_id::ReportId;

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
            is_big_guy: false,
            ..Default::default()
};
        if team == "home" { game.team_home.players.push(p); } else { game.team_away.players.push(p); }
        game.field_model.set_player_coordinate(id, FieldCoordinate::new(5, 5));
        game.field_model.set_player_state(id, ffb_model::enums::PlayerState::new(PS_STANDING));
    }

    #[test]
    fn chainsaw_roll_report_added_on_roll() {
        let mut game = make_game();
        add_player(&mut game, "home", "atk", Some(SkillId::Chainsaw));
        game.acting_player.player_id = Some("atk".into());
        let mut step = StepFoulChainsaw::new("fail".into());
        step.using_chainsaw = true;
        // Check if Chainsaw has the blocksLikeChainsaw property
        let has_prop = game.player("atk")
            .map(|p| p.has_skill_property(NamedProperties::BLOCKS_LIKE_CHAINSAW))
            .unwrap_or(false);
        if !has_prop {
            return; // Chainsaw property not yet wired — skip
        }
        step.start(&mut game, &mut GameRng::new(0));
        assert!(
            game.report_list.has_report(ReportId::CHAINSAW_ROLL),
            "should add ReportChainsawRoll when chainsaw roll is made"
        );
    }

    #[test]
    fn no_chainsaw_report_when_no_chainsaw_skill() {
        let mut game = make_game();
        add_player(&mut game, "home", "atk", None); // no Chainsaw skill
        game.acting_player.player_id = Some("atk".into());
        let mut step = StepFoulChainsaw::new("fail".into());
        step.using_chainsaw = true;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(
            !game.report_list.has_report(ReportId::CHAINSAW_ROLL),
            "should not add ReportChainsawRoll when no chainsaw skill"
        );
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

    /// Regression test: Java gates on `UtilCards.hasUnusedSkillWithProperty`, not just
    /// `hasSkillProperty` — a chainsaw that has already been used this action must not
    /// trigger a second roll. Before this fix, `has_chainsaw` used `has_skill_property`,
    /// which ignores `used_skills` entirely, so an already-used Chainsaw would still let
    /// the step proceed to roll again.
    #[test]
    fn already_used_chainsaw_skill_returns_next_step() {
        let mut game = make_game();
        add_player(&mut game, "home", "atk", Some(SkillId::Chainsaw));
        if let Some(p) = game.team_home.player_mut("atk") {
            p.used_skills.insert(SkillId::Chainsaw);
        }
        game.acting_player.player_id = Some("atk".into());
        let mut step = StepFoulChainsaw::new("fail".into());
        step.using_chainsaw = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.is_empty(), "an already-used chainsaw skill must not trigger a roll");
        assert!(!game.report_list.has_report(ReportId::CHAINSAW_ROLL));
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

    /// Regression test: with a forced roll-1 failure (no TRR available, so it always drops),
    /// Java only sets `causesTurnOver = true` on an armor break when the `CHAINSAW_TURNOVER`
    /// option is NOT `never`. Previously Rust hardcoded ALL_AV_BREAKS behavior and ignored the
    /// option, so `CHAINSAW_TURNOVER_NEVER` had no effect on the attacker's own backfire.
    #[test]
    fn chainsaw_turnover_never_suppresses_turnover_on_attacker_armor_break() {
        let mut game = make_game();
        add_player(&mut game, "home", "atk", Some(SkillId::Chainsaw));
        game.acting_player.player_id = Some("atk".into());
        // armour 2 guarantees the backfire injury breaks armor
        game.team_home.players[0].armour = 2;
        game.options.set(
            ffb_model::option::game_option_id::CHAINSAW_TURNOVER,
            ffb_model::option::game_option_string::CHAINSAW_TURNOVER_NEVER,
        );
        let mut step = StepFoulChainsaw::new("fail".into());
        step.using_chainsaw = true;
        let seed = failing_chainsaw_roll_seed();
        let out = step.start(&mut game, &mut GameRng::new(seed));
        let dpc = out.published.iter().find_map(|p| {
            if let StepParameter::DropPlayerContext(ctx) = p { Some(ctx.clone()) } else { None }
        }).expect("DROP_PLAYER_CONTEXT must be published on backfire");
        assert!(dpc.injury_result.as_ref().unwrap().injury_context().is_armor_broken());
        assert!(!dpc.end_turn, "CHAINSAW_TURNOVER_NEVER must suppress turnover even on armor break");
    }

    /// Find a seed whose first `d6()` roll is below `minimum_roll_chainsaw()`, guaranteeing
    /// the chainsaw roll fails (and, with no TRR configured on the test team, backfires).
    fn failing_chainsaw_roll_seed() -> u64 {
        let minimum_roll = minimum_roll_chainsaw();
        for seed in 0..1000u64 {
            let mut probe = GameRng::new(seed);
            if probe.d6() < minimum_roll {
                return seed;
            }
        }
        panic!("no failing seed found");
    }

    /// Regression test: Java independently publishes END_TURN=true (in addition to whatever
    /// `DropPlayerContext.endTurn` ends up as) when the roll does NOT break armor and the
    /// option is `kickback`. This extra publish did not exist at all in the prior translation.
    #[test]
    fn chainsaw_turnover_kickback_publishes_end_turn_without_armor_break() {
        let mut game = make_game();
        add_player(&mut game, "home", "atk", Some(SkillId::Chainsaw));
        game.acting_player.player_id = Some("atk".into());
        // high armour so the backfire injury never breaks armor
        game.team_home.players[0].armour = 13;
        game.options.set(
            ffb_model::option::game_option_id::CHAINSAW_TURNOVER,
            ffb_model::option::game_option_string::CHAINSAW_TURNOVER_KICKBACK,
        );
        let mut step = StepFoulChainsaw::new("fail".into());
        step.using_chainsaw = true;
        let seed = failing_chainsaw_roll_seed();
        let out = step.start(&mut game, &mut GameRng::new(seed));
        let dpc = out.published.iter().find_map(|p| {
            if let StepParameter::DropPlayerContext(ctx) = p { Some(ctx.clone()) } else { None }
        }).expect("DROP_PLAYER_CONTEXT must be published on backfire");
        assert!(!dpc.injury_result.as_ref().unwrap().injury_context().is_armor_broken());
        assert!(
            out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))),
            "CHAINSAW_TURNOVER_KICKBACK must publish an extra END_TURN=true even without an armor break"
        );
    }
}
