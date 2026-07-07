/// 1:1 translation of com.fumbbl.ffb.server.step.action.block.StepDauntless (COMMON rules)
/// and its BB2020/BB2025 hook com.fumbbl.ffb.server.skillbehaviour.mixed.DauntlessBehaviour.
///
/// Dauntless allows a weaker attacker to match the defender's strength on a successful roll.
/// If attacker.ST >= defender.ST, or a special action (stab/chainsaw/vomit/breathe fire/chomp)
/// is used, the skill is skipped.  Minimum roll = min(6, defenderST − attackerST + 1).
/// On success: SuccessfulDauntless(true) published.  On failure: NEXT_STEP (re-roll stub).
use ffb_model::enums::{ReRollSource, SkillId};
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::report::report_dauntless_roll::ReportDauntlessRoll;
use ffb_mechanics::mechanics::minimum_roll_dauntless;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

pub struct StepDauntless {
    /// Java: state.usingStab
    pub using_stab: bool,
    /// Java: state.usingChainsaw
    pub using_chainsaw: bool,
    /// Java: state.usingVomit
    pub using_vomit: bool,
    /// Java: state.usingBreatheFire
    pub using_breathe_fire: bool,
    /// Java: state.usingChomp
    pub using_chomp: bool,
    // AbstractStepWithReRoll fields
    pub re_rolled_action: Option<String>,
    pub re_roll_source: Option<String>,
}

impl StepDauntless {
    pub fn new() -> Self {
        Self {
            using_stab: false,
            using_chainsaw: false,
            using_vomit: false,
            using_breathe_fire: false,
            using_chomp: false,
            re_rolled_action: None,
            re_roll_source: None,
        }
    }

    /// Java: StepState.usesSpecialAction()
    fn uses_special_action(&self) -> bool {
        self.using_stab
            || self.using_chainsaw
            || self.using_vomit
            || self.using_breathe_fire
            || self.using_chomp
    }
}

impl Default for StepDauntless {
    fn default() -> Self { Self::new() }
}

impl Step for StepDauntless {
    fn id(&self) -> StepId { StepId::Dauntless }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if let Action::UseReRoll { use_reroll: false } = action {
            self.re_roll_source = None;
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::UsingStab(v)        => { self.using_stab = *v; true }
            StepParameter::UsingChainsaw(v)    => { self.using_chainsaw = *v; true }
            StepParameter::UsingVomit(v)       => { self.using_vomit = *v; true }
            StepParameter::UsingBreatheFire(v) => { self.using_breathe_fire = *v; true }
            StepParameter::UsingChomp(v)       => { self.using_chomp = *v; true }
            _ => false,
        }
    }
}

impl StepDauntless {
    /// Java: DauntlessBehaviour(mixed).handleExecuteStepHook
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let player_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        let re_rolled = self.re_rolled_action.as_deref() == Some("DAUNTLESS");

        // Java: if (status == null && hasDauntless && strengthInRange && !usesSpecialAction)
        //       || status == WAITING_FOR_RE_ROLL
        if !re_rolled {
            let has_dauntless = game.player(&player_id)
                .map(|p| p.has_skill(SkillId::Dauntless) && !p.used_skills.contains(&SkillId::Dauntless))
                .unwrap_or(false);

            if !has_dauntless || self.uses_special_action() {
                return StepOutcome::next();
            }
        }

        let attacker_st = game.player(&player_id).map(|p| p.strength).unwrap_or(3);
        let defender_st = game.defender_id.as_ref()
            .and_then(|id| game.player(id))
            .map(|p| p.strength)
            .unwrap_or(3);

        let in_range = attacker_st < defender_st && attacker_st + 6 > defender_st;
        if !in_range {
            return StepOutcome::next();
        }

        // Java: if (status == WAITING_FOR_RE_ROLL) { if (source == null || !useReRoll) doRoll = false }
        let mut do_roll = true;
        if re_rolled {
            if let Some(ref source_name) = self.re_roll_source.clone() {
                let source = ReRollSource::new(source_name.as_str());
                if !use_reroll(game, &source, &player_id) {
                    do_roll = false;
                }
            } else {
                do_roll = false; // player declined
            }
        }

        let min_roll = minimum_roll_dauntless(attacker_st, defender_st);

        if !do_roll {
            return StepOutcome::next();
        }

        let roll = rng.d6();
        let success = roll >= min_roll;

        let event = GameEvent::DauntlessRoll {
            player_id: player_id.clone(),
            roll,
            success,
        };

        // Java: step.getResult().addReport(new ReportDauntlessRoll(...))
        game.report_list.add(ReportDauntlessRoll::new(
            Some(player_id.clone()),
            success,
            roll,
            min_roll,
            re_rolled,
            defender_st,
            game.defender_id.clone(),
        ));

        if success {
            StepOutcome::next()
                .with_event(event)
                .publish(StepParameter::SuccessfulDauntless(true))
        } else {
            // Java: if (status == null && askForReRollIfAvailable(...)) → CONTINUE
            if !re_rolled {
                if let Some(prompt) = ask_for_reroll_if_available(game, "DAUNTLESS", min_roll, false) {
                    self.re_rolled_action = Some("DAUNTLESS".into());
                    self.re_roll_source = Some("TRR".into());
                    return StepOutcome::cont().with_event(event).with_prompt(prompt);
                }
            }
            StepOutcome::next().with_event(event)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::Action;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::Rules;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::types::FieldCoordinate;

    fn add_player(
        team: &mut ffb_model::model::team::Team,
        id: &str,
        nr: i32,
        strength: i32,
        skills: Vec<SkillId>,
    ) {
        team.players.push(ffb_model::model::player::Player {
            id: id.into(),
            name: id.into(),
            nr,
            position_id: "pos".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6,
            strength,
            agility: 3,
            passing: 4,
            armour: 8,
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

    fn make_game(attacker_st: i32, attacker_skills: Vec<SkillId>, defender_st: i32) -> Game {
        let mut home = test_team("home", 0);
        let mut away = test_team("away", 0);
        add_player(&mut home, "att", 1, attacker_st, attacker_skills);
        add_player(&mut away, "def", 2, defender_st, vec![]);
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.home_playing = true;
        game.acting_player.player_id = Some("att".into());
        game.defender_id = Some("def".into());
        game.field_model.set_player_coordinate("att", FieldCoordinate::new(5, 5));
        game.field_model.set_player_coordinate("def", FieldCoordinate::new(6, 5));
        game
    }

    fn seed_for_d6(target: i32) -> u64 {
        for s in 0u64..10_000 {
            if GameRng::new(s).d6() == target {
                return s;
            }
        }
        panic!("no seed for d6={}", target);
    }

    #[test]
    fn no_acting_player_returns_next() {
        let mut game = make_game(2, vec![SkillId::Dauntless], 4);
        game.acting_player.player_id = None;
        let outcome = StepDauntless::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.events.is_empty());
    }

    #[test]
    fn no_dauntless_skill_returns_next() {
        let mut game = make_game(2, vec![], 4);
        let outcome = StepDauntless::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.events.is_empty());
    }

    #[test]
    fn equal_strength_skips_roll() {
        // attacker.ST == defender.ST → not in range (not strictly less than)
        let mut game = make_game(3, vec![SkillId::Dauntless], 3);
        let outcome = StepDauntless::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.events.is_empty());
    }

    #[test]
    fn attacker_stronger_skips_roll() {
        let mut game = make_game(4, vec![SkillId::Dauntless], 3);
        let outcome = StepDauntless::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.events.is_empty());
    }

    #[test]
    fn strength_gap_too_large_skips_roll() {
        // attacker.ST + 6 <= defender.ST → not in range
        // e.g. attacker=2, defender=8: 2+6=8 which is NOT > 8
        let mut game = make_game(2, vec![SkillId::Dauntless], 8);
        let outcome = StepDauntless::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.events.is_empty());
    }

    #[test]
    fn uses_stab_skips_roll() {
        let mut game = make_game(2, vec![SkillId::Dauntless], 4);
        let mut step = StepDauntless::new();
        step.using_stab = true;
        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.events.is_empty());
    }

    #[test]
    fn uses_chainsaw_skips_roll() {
        let mut game = make_game(2, vec![SkillId::Dauntless], 4);
        let mut step = StepDauntless::new();
        step.using_chainsaw = true;
        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.events.is_empty());
    }

    #[test]
    fn successful_roll_publishes_successful_dauntless() {
        // attacker=2, defender=4 → min_roll = min(6, 4-2+1) = 3 → roll ≥ 3 = success
        let seed = seed_for_d6(4); // 4 >= 3 → success
        let mut game = make_game(2, vec![SkillId::Dauntless], 4);
        let outcome = StepDauntless::new().start(&mut game, &mut GameRng::new(seed));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.published.iter().any(|p| matches!(p, StepParameter::SuccessfulDauntless(true))));
    }

    #[test]
    fn successful_roll_emits_dauntless_roll_event() {
        let seed = seed_for_d6(5);
        let mut game = make_game(2, vec![SkillId::Dauntless], 4);
        let outcome = StepDauntless::new().start(&mut game, &mut GameRng::new(seed));
        assert!(outcome.events.iter().any(|e| matches!(e, GameEvent::DauntlessRoll { success: true, .. })));
    }

    #[test]
    fn failed_roll_returns_next_without_publishing() {
        // attacker=2, defender=4 → min_roll=3 → roll=1 → failure
        let seed = seed_for_d6(1);
        let mut game = make_game(2, vec![SkillId::Dauntless], 4);
        let outcome = StepDauntless::new().start(&mut game, &mut GameRng::new(seed));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(!outcome.published.iter().any(|p| matches!(p, StepParameter::SuccessfulDauntless(_))));
        assert!(outcome.events.iter().any(|e| matches!(e, GameEvent::DauntlessRoll { success: false, .. })));
    }

    #[test]
    fn set_parameter_stores_flags() {
        let mut step = StepDauntless::new();
        assert!(step.set_parameter(&StepParameter::UsingStab(true)));
        assert!(step.set_parameter(&StepParameter::UsingChainsaw(true)));
        assert!(step.set_parameter(&StepParameter::UsingVomit(true)));
        assert!(step.set_parameter(&StepParameter::UsingBreatheFire(true)));
        assert!(step.set_parameter(&StepParameter::UsingChomp(true)));
        assert!(step.using_stab);
        assert!(step.using_chainsaw);
        assert!(step.using_vomit);
        assert!(step.using_breathe_fire);
        assert!(step.using_chomp);
    }

    #[test]
    fn unknown_parameter_returns_false() {
        let mut step = StepDauntless::new();
        assert!(!step.set_parameter(&StepParameter::EndPlayerAction(true)));
    }

    #[test]
    fn failed_roll_with_trr_offers_reroll_prompt() {
        let seed = seed_for_d6(1); // 1 < 3 → failure
        let mut game = make_game(2, vec![SkillId::Dauntless], 4);
        game.turn_data_home.rerolls = 1;
        let mut step = StepDauntless::new();
        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::Continue, "TRR available → offer re-roll");
        assert!(out.prompt.is_some());
        assert_eq!(step.re_rolled_action.as_deref(), Some("DAUNTLESS"));
    }

    #[test]
    fn successful_roll_adds_dauntless_roll_report() {
        let seed = seed_for_d6(5);
        let mut game = make_game(2, vec![SkillId::Dauntless], 4);
        StepDauntless::new().start(&mut game, &mut GameRng::new(seed));
        assert!(
            game.report_list.has_report(ffb_model::report::report_id::ReportId::DAUNTLESS_ROLL),
            "successful roll should add ReportDauntlessRoll"
        );
    }

    #[test]
    fn failed_roll_adds_dauntless_roll_report() {
        let seed = seed_for_d6(1);
        let mut game = make_game(2, vec![SkillId::Dauntless], 4);
        StepDauntless::new().start(&mut game, &mut GameRng::new(seed));
        assert!(
            game.report_list.has_report(ffb_model::report::report_id::ReportId::DAUNTLESS_ROLL),
            "failed roll should add ReportDauntlessRoll"
        );
    }

    #[test]
    fn decline_reroll_returns_next_step() {
        let mut game = make_game(2, vec![SkillId::Dauntless], 4);
        let mut step = StepDauntless::new();
        step.re_rolled_action = Some("DAUNTLESS".into());
        step.re_roll_source = Some("TRR".into());
        let out = step.handle_command(
            &Action::UseReRoll { use_reroll: false },
            &mut game,
            &mut GameRng::new(0),
        );
        // Declined → doRoll = false → NEXT_STEP (dauntless failed, block proceeds normally)
        assert_eq!(out.action, StepAction::NextStep);
    }
}
