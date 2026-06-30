use ffb_model::enums::{ApothecaryMode, PlayerAction, ReRollSource};
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_cards::UtilCards;
use ffb_model::util::util_player::UtilPlayer;
use crate::action::Action;
use crate::drop_player_context::{DropPlayerContext, SteadyFootingContext};
use crate::step::framework::{Step, StepOutcome, StepParameter};
use crate::step::framework::StepId;
use crate::step::util_server_injury::handle_injury_by_name;
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.block.StepBlockChainsaw.
///
/// Handles the Chainsaw skill during blocks (roll d6 ≥ 4 = hit).
/// - Attacker has no `blocksLikeChainsaw` or `usingChainsaw = false` → NEXT_STEP (normal block).
/// - Successful hit → injury on defender → publish `DROP_PLAYER_CONTEXT` → NEXT_STEP.
/// - Failed roll or re-roll unavailable → chainsaw backfires on attacker → publish
///   `STEADY_FOOTING_CONTEXT(DropPlayerContext{attacker})` → NEXT_STEP.
///
/// Game option `CHAINSAW_TURNOVER` is not yet ported; behaviour is `all_av_breaks` (if armor
/// broken → causesTurnOver = true). `KICKBACK` and `NEVER` variants are stubs.
pub struct StepBlockChainsaw {
    pub goto_label_on_success: String,
    pub goto_label_on_failure: String,
    pub using_chainsaw: bool,
    // AbstractStepWithReRoll stubs
    pub re_rolled_action: Option<String>,
    pub re_roll_source: Option<String>,
}

impl StepBlockChainsaw {
    pub fn new(goto_label_on_success: String, goto_label_on_failure: String) -> Self {
        Self {
            goto_label_on_success,
            goto_label_on_failure,
            using_chainsaw: false,
            re_rolled_action: None,
            re_roll_source: None,
        }
    }
}

impl Step for StepBlockChainsaw {
    fn id(&self) -> StepId { StepId::BlockChainsaw }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::UseReRoll { use_reroll: false } => {
                self.re_roll_source = None;
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::UsingChainsaw(v) => { self.using_chainsaw = *v; true }
            StepParameter::GotoLabelOnSuccess(v) => { self.goto_label_on_success = v.clone(); true }
            StepParameter::GotoLabelOnFailure(v) => { self.goto_label_on_failure = v.clone(); true }
            _ => false,
        }
    }
}

impl StepBlockChainsaw {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let attacker_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        let attacker_has_chainsaw = game.player(&attacker_id)
            .map(|p| p.has_skill_property(NamedProperties::BLOCKS_LIKE_CHAINSAW))
            .unwrap_or(false);

        if !attacker_has_chainsaw || !self.using_chainsaw {
            return StepOutcome::next();
        }

        // Java: actingPlayer.markSkillUsed(blocksLikeChainsaw)
        {
            let sid = game.player(&attacker_id).and_then(|p| UtilCards::get_unused_skill_with_property(
                p, NamedProperties::BLOCKS_LIKE_CHAINSAW));
            if let Some(sid) = sid {
                let is_home = game.team_home.player(&attacker_id).is_some();
                if is_home { game.team_home.player_mut(&attacker_id).map(|p| p.used_skills.insert(sid)); }
                else { game.team_away.player_mut(&attacker_id).map(|p| p.used_skills.insert(sid)); }
            }
        }

        // Java: if (actingPlayer.getPlayerAction() == PlayerAction.MAXIMUM_CARNAGE) → markSkillUsed(canPerformSecondChainsawAttack)
        if game.acting_player.player_action == Some(PlayerAction::MaximumCarnage) {
            let sid = game.player(&attacker_id).and_then(|p| UtilCards::get_unused_skill_with_property(
                p, NamedProperties::CAN_PERFORM_SECOND_CHAINSAW_ATTACK));
            if let Some(sid) = sid {
                let is_home = game.team_home.player(&attacker_id).is_some();
                if is_home { game.team_home.player_mut(&attacker_id).map(|p| p.used_skills.insert(sid)); }
                else { game.team_away.player_mut(&attacker_id).map(|p| p.used_skills.insert(sid)); }
            }
        }

        // Java: if (CHAINSAW == reRolledAction) { if (source == null || !useReRoll) → drop }
        let mut drop_chainsaw_player = false;
        if self.re_rolled_action.as_deref() == Some("CHAINSAW") {
            if let Some(ref source_name) = self.re_roll_source.clone() {
                let source = ReRollSource::new(source_name.as_str());
                if !use_reroll(game, &source, &attacker_id) {
                    drop_chainsaw_player = true;
                }
            } else {
                drop_chainsaw_player = true; // player declined
            }
        }

        if drop_chainsaw_player {
            return self.backfire(game, rng, &attacker_id);
        }

        // Java: int roll = rollChainsaw(); int minimumRoll = minimumRollChainsaw() (= 2, only 1 fails)
        let roll = rng.d6();
        let minimum_roll = ffb_mechanics::mechanics::minimum_roll_chainsaw();
        let successful = roll >= minimum_roll;

        if successful {
            let defender_id = match game.defender_id.clone() {
                Some(id) => id,
                None => return StepOutcome::next(),
            };
            let defender_coord = game.field_model.player_coordinate(&defender_id)
                .unwrap_or(FieldCoordinate::new(0, 0));
            let defender_state = game.field_model.player_state(&defender_id);
            let already_dropped = defender_state.map(|s| s.is_prone_or_stunned()).unwrap_or(false);

            let grants_spp = false; // Stub: SPP from special actions not yet ported
            let injury_type_name = if grants_spp { "InjuryTypeChainsawForSpp" } else { "InjuryTypeChainsaw" };

            let injury_result_defender = handle_injury_by_name(
                game, rng, injury_type_name,
                Some(&attacker_id.clone()), &defender_id,
                defender_coord, None, None, ApothecaryMode::Defender,
            );

            let armor_broken = injury_result_defender.injury_context().is_armor_broken();
            let causes_turn_over = armor_broken; // CHAINSAW_TURNOVER_ALL_AV_BREAKS default

            let dpc = DropPlayerContext {
                injury_result: Some(Box::new(injury_result_defender)),
                end_turn: causes_turn_over,
                eligible_for_safe_pair_of_hands: true,
                label: if self.goto_label_on_success.is_empty() { None } else { Some(self.goto_label_on_success.clone()) },
                player_id: Some(defender_id.clone()),
                apothecary_mode: Some(ApothecaryMode::Defender),
                requires_armour_break: true,
                already_dropped,
                ..DropPlayerContext::new()
            };

            StepOutcome::next().publish(StepParameter::DropPlayerContext(Box::new(dpc)))
        } else {
            // Failed roll — Java: if (reRolled || !askForReRollIfAvailable(...)) → drop
            let re_rolled = self.re_rolled_action.as_deref() == Some("CHAINSAW")
                && self.re_roll_source.is_some();
            if !re_rolled {
                if let Some(prompt) = ask_for_reroll_if_available(game, "CHAINSAW", minimum_roll, false) {
                    self.re_rolled_action = Some("CHAINSAW".into());
                    self.re_roll_source = Some("TRR".into());
                    return StepOutcome::cont().with_prompt(prompt);
                }
            }
            self.backfire(game, rng, &attacker_id)
        }
    }

    /// Java: dropChainsawPlayer path — chainsaw backfires, injures attacker.
    fn backfire(&self, game: &mut Game, rng: &mut GameRng, attacker_id: &str) -> StepOutcome {
        let attacker_coord = game.field_model.player_coordinate(attacker_id)
            .unwrap_or(FieldCoordinate::new(0, 0));

        let player_has_ball = UtilPlayer::has_ball(game, attacker_id);

        // Java: UtilServerInjury.handleInjury(this, new InjuryTypeChainsaw(), null, attacker, ...)
        let injury_result_attacker = handle_injury_by_name(
            game, rng, "InjuryTypeChainsaw",
            None, attacker_id,
            attacker_coord, None, None, ApothecaryMode::Attacker,
        );

        // Java: causesTurnOver(hasBall, chainsawOption, injuryContext)
        // Stub: chainsawOption = "all_av_breaks" default
        let armor_broken = injury_result_attacker.injury_context().is_armor_broken();
        let causes_turn_over = if armor_broken {
            // CHAINSAW_TURNOVER_NEVER → only if has ball; otherwise true
            // Stub (not never): always true when armor broken
            true
        } else {
            // CHAINSAW_TURNOVER_KICKBACK → true; others → false
            false
        };
        let _ = player_has_ball; // used by CHAINSAW_TURNOVER_NEVER case; suppressed until options ported

        // Java: modifiedInjuryCausesTurnover — requires ModifiedInjuryContext (not yet ported)
        let modified_injury_causes_turnover = false;

        // Java: endTurnWithoutKnockdown = GameOptionString.CHAINSAW_TURNOVER_KICKBACK.equals(chainsawOption)
        // Stub: not KICKBACK → false
        let end_turn_without_knockdown = false;

        let dpc = DropPlayerContext {
            injury_result: Some(Box::new(injury_result_attacker)),
            end_turn: causes_turn_over,
            eligible_for_safe_pair_of_hands: true,
            label: if self.goto_label_on_failure.is_empty() { None } else { Some(self.goto_label_on_failure.clone()) },
            player_id: Some(attacker_id.to_owned()),
            apothecary_mode: Some(ApothecaryMode::Attacker),
            requires_armour_break: true,
            already_dropped: false,
            modified_injury_ends_turn: modified_injury_causes_turnover,
            end_turn_without_knockdown,
            ..DropPlayerContext::new()
        };

        let ctx = SteadyFootingContext::from_drop_player(dpc);
        StepOutcome::next().publish(StepParameter::SteadyFootingContext(Box::new(ctx)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::{Rules, PS_STANDING, PlayerType, PlayerGender};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::enums::SkillId;
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    fn add_player_with_skill(game: &mut Game, team: &str, id: &str, skill: Option<SkillId>) -> FieldCoordinate {
        let pos = FieldCoordinate::new(5, 5);
        let mut p = Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
        };
        if let Some(s) = skill {
            p.starting_skills.push(SkillWithValue { skill_id: s, value: None });
        }
        if team == "home" {
            game.team_home.players.push(p);
        } else {
            game.team_away.players.push(p);
        }
        game.field_model.set_player_coordinate(id, pos);
        game.field_model.set_player_state(id, ffb_model::enums::PlayerState::new(PS_STANDING));
        pos
    }

    #[test]
    fn not_using_chainsaw_returns_next_step() {
        let mut step = StepBlockChainsaw::new("success".into(), "failure".into());
        step.using_chainsaw = false;
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(42));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn using_chainsaw_but_player_has_no_skill_returns_next_step() {
        let mut step = StepBlockChainsaw::new("success".into(), "failure".into());
        step.using_chainsaw = true;
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(42));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_using_chainsaw_accepted() {
        let mut step = StepBlockChainsaw::new("s".into(), "f".into());
        assert!(!step.using_chainsaw);
        step.set_parameter(&StepParameter::UsingChainsaw(true));
        assert!(step.using_chainsaw);
    }

    /// Successful chainsaw hit → publish DROP_PLAYER_CONTEXT.
    #[test]
    fn successful_hit_publishes_drop_player_context() {
        // Setup: attacker with Chainsaw skill, defender on the field
        let mut game = make_game();
        add_player_with_skill(&mut game, "home", "atk1", Some(SkillId::Chainsaw));
        add_player_with_skill(&mut game, "away", "def1", None);
        game.acting_player.player_id = Some("atk1".into());
        game.defender_id = Some("def1".into());

        // Find seed where d6 >= 2 (successful hit; minimum is 2, only 1 fails)
        for seed in 0..200u64 {
            let mut rng = GameRng::new(seed);
            if rng.d6() >= 2 {
                let mut g = make_game();
                add_player_with_skill(&mut g, "home", "atk1", Some(SkillId::Chainsaw));
                add_player_with_skill(&mut g, "away", "def1", None);
                g.acting_player.player_id = Some("atk1".into());
                g.defender_id = Some("def1".into());

                let mut step = StepBlockChainsaw::new("success".into(), "failure".into());
                step.using_chainsaw = true;
                let out = step.start(&mut g, &mut GameRng::new(seed));
                assert_eq!(out.action, StepAction::NextStep, "seed={seed}");
                assert!(
                    out.published.iter().any(|p| matches!(p, StepParameter::DropPlayerContext(_))),
                    "seed={seed}: expected DropPlayerContext"
                );
                return;
            }
        }
        panic!("no seed produces d6>=2");
    }

    /// Failed chainsaw hit (roll = 1) → publish STEADY_FOOTING_CONTEXT (backfire).
    #[test]
    fn failed_hit_publishes_steady_footing_context() {
        for seed in 0..200u64 {
            let mut rng = GameRng::new(seed);
            if rng.d6() < 2 {
                let mut g = make_game();
                add_player_with_skill(&mut g, "home", "atk1", Some(SkillId::Chainsaw));
                add_player_with_skill(&mut g, "away", "def1", None);
                g.acting_player.player_id = Some("atk1".into());
                g.defender_id = Some("def1".into());

                let mut step = StepBlockChainsaw::new("success".into(), "failure".into());
                step.using_chainsaw = true;
                let out = step.start(&mut g, &mut GameRng::new(seed));
                assert_eq!(out.action, StepAction::NextStep, "seed={seed}");
                assert!(
                    out.published.iter().any(|p| matches!(p, StepParameter::SteadyFootingContext(_))),
                    "seed={seed}: expected SteadyFootingContext"
                );
                return;
            }
        }
        panic!("no seed produces d6<2 (only 1 fails)");
    }
}
