/// 1:1 translation of com.fumbbl.ffb.server.step.action.foul.StepReferee (COMMON rules)
/// and its BB2025 hook com.fumbbl.ffb.server.skillbehaviour.bb2025.SneakyGitBehaviour
/// (the StepReferee modifier).
///
/// Checks whether the referee spots the foul. Needs GOTO_LABEL_ON_END init parameter.
/// Expects INJURY_RESULT (ApothecaryMode::Defender) to be set by a preceding step.
///
/// If no injury result is set, the step is a no-op (NEXT_STEP).
/// If the referee spots the foul → NEXT_STEP (eject player downstream).
/// If the referee misses → GOTO_LABEL_ON_END (skip eject).
use ffb_model::enums::SkillId;
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::report::mixed::report_referee::ReportReferee;
use ffb_model::report::report_id::ReportId;
use crate::action::Action;
use crate::injury::InjuryResult;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

pub struct StepReferee {
    /// Java: state.gotoLabelOnEnd — GOTO_LABEL_ON_END init parameter.
    pub goto_label_on_end: String,
    /// Java: state.injuryResultDefender — set by INJURY_RESULT parameter (ApothecaryMode::Defender).
    pub injury_result_defender: Option<Box<InjuryResult>>,
}

impl StepReferee {
    pub fn new() -> Self {
        Self {
            goto_label_on_end: String::new(),
            injury_result_defender: None,
        }
    }
}

impl Default for StepReferee {
    fn default() -> Self { Self::new() }
}

impl Step for StepReferee {
    fn id(&self) -> StepId { StepId::Referee }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(v) => {
                self.goto_label_on_end = v.clone();
                true
            }
            StepParameter::InjuryResult(r) => {
                // Java: if INJURY_RESULT && apothecaryMode == DEFENDER → store
                if r.injury_context().get_apothecary_mode() == ffb_model::enums::ApothecaryMode::Defender {
                    self.injury_result_defender = Some(r.clone());
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

impl StepReferee {
    /// Java: SneakyGitBehaviour.handleExecuteStepHook(StepReferee, StepReferee.StepState)
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: if (state.injuryResultDefender != null) { executeStepHooks(this, state) }
        // When no injury result is present, the step is a no-op → NEXT_STEP.
        let injury_result = match &self.injury_result_defender {
            Some(r) => r,
            None => return StepOutcome::next(),
        };

        let ctx = injury_result.injury_context();

        let player_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        let has_sneaky_git = game.player(&player_id)
            .map(|p| p.has_skill(SkillId::SneakyGit))
            .unwrap_or(false);

        // Java: game.isActive(NamedProperties.foulBreaksArmourWithoutRoll)
        // Stub: NamedProperties not yet implemented → always false.
        let foul_breaks_armour_without_roll = false;

        // Java: GameOptionId.SNEAKY_GIT_BAN_TO_KO option
        // Stub: game options not yet implemented → false.
        let sneaky_git_ban_to_ko = false;

        let mut referee_spots_foul = false;

        // Java: if (!game.isActive(foulBreaksArmourWithoutRoll) && (!hasSneakyGit || armorBroken || (hasSneakyGit && banToKo)))
        if !foul_breaks_armour_without_roll
            && (!has_sneaky_git
                || ctx.is_armor_broken()
                || (has_sneaky_git && sneaky_git_ban_to_ko))
        {
            if let Some(armor) = ctx.get_armor_roll() {
                referee_spots_foul = armor[0] == armor[1];
            }
        }

        // Java: if (!refereeSpotsFoul && isArmorBroken()) → check injuryRoll doubles
        if !referee_spots_foul && ctx.is_armor_broken() {
            if let Some(injury) = ctx.get_injury_roll() {
                referee_spots_foul = injury[0] == injury[1];
            }
        }

        // Java: underScrutiny = prayerState.isUnderScrutiny(actingPlayer.getPlayer().getTeam())
        // Stub: prayer state not yet implemented → false.
        let under_scrutiny = false;
        referee_spots_foul |= under_scrutiny && ctx.is_armor_broken();

        // Java: addReport(new ReportReferee(refereeSpotsFoul, underScrutiny))
        game.report_list.add(ReportReferee::new(referee_spots_foul, under_scrutiny));
        let referee_event = GameEvent::RefereeSpotsFoul { referee_spots_foul, under_scrutiny };

        // Java: if (!refereeSpotsFoul) { loop over opponentInducementSet.value(Usage.SPOT_FOUL) }
        // Stub: InducementSet not yet implemented → 0 biased refs.
        // biased_ref_count = 0 → loop never runs.
        let biased_ref_count: u32 = 0;
        let mut biased_ref_events: Vec<GameEvent> = Vec::new();
        for _ in 0..biased_ref_count {
            let roll = rng.d6();
            let spotted = roll > 4;
            biased_ref_events.push(GameEvent::BiasedRefRoll { roll, referee_spots_foul: spotted });
            if spotted {
                referee_spots_foul = true;
                break;
            }
        }

        // Java: if (refereeSpotsFoul) → NEXT_STEP else → GOTO_LABEL(gotoLabelOnEnd)
        let mut outcome = if referee_spots_foul {
            StepOutcome::next()
        } else {
            StepOutcome::goto(&self.goto_label_on_end)
        };

        outcome = outcome.with_event(referee_event);
        for ev in biased_ref_events {
            outcome = outcome.with_event(ev);
        }
        outcome
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::injury::InjuryContext;
    use crate::step::framework::StepAction;
    use crate::step::framework::test_team;
    use ffb_model::enums::{ApothecaryMode, Rules, TurnMode, PS_STANDING};
    use ffb_model::enums::PlayerState;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::types::FieldCoordinate;

    fn make_injury_result(
        armor_roll: Option<[i32; 2]>,
        armor_broken: bool,
        injury_roll: Option<[i32; 2]>,
    ) -> Box<InjuryResult> {
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        ctx.armor_roll = armor_roll;
        ctx.armor_broken = armor_broken;
        ctx.injury_roll = injury_roll;
        Box::new(InjuryResult { injury_context: ctx, knocked_out: false, rip: false, already_reported: false, pre_regeneration: true })
    }

    fn make_game_with_player(skills: Vec<SkillId>) -> (Game, String) {
        let pid = "p1".to_string();
        let mut home = test_team("home", 0);
        home.players.push(ffb_model::model::player::Player {
            id: pid.clone(),
            name: "fouler".into(),
            nr: 1,
            position_id: "pos".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: skills.into_iter()
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
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.acting_player.player_id = Some(pid.clone());
        game.field_model.set_player_state(&pid, PlayerState::new(PS_STANDING).change_active(true));
        game.field_model.set_player_coordinate(&pid, FieldCoordinate::new(5, 7));
        (game, pid)
    }

    #[test]
    fn no_injury_result_skips_logic() {
        let (mut game, _) = make_game_with_player(vec![]);
        let outcome = StepReferee::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.events.is_empty());
    }

    #[test]
    fn non_defender_injury_result_not_stored() {
        let mut step = StepReferee::new();
        let non_defender = Box::new(InjuryResult::new(ApothecaryMode::Attacker));
        assert!(!step.set_parameter(&StepParameter::InjuryResult(non_defender)));
        assert!(step.injury_result_defender.is_none());
    }

    #[test]
    fn defender_injury_result_is_stored() {
        let mut step = StepReferee::new();
        let defender = Box::new(InjuryResult::new(ApothecaryMode::Defender));
        assert!(step.set_parameter(&StepParameter::InjuryResult(defender)));
        assert!(step.injury_result_defender.is_some());
    }

    #[test]
    fn goto_label_on_end_stored() {
        let mut step = StepReferee::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnEnd("END".into())));
        assert_eq!(step.goto_label_on_end, "END");
    }

    #[test]
    fn armor_doubles_referee_spots_foul() {
        let (mut game, _) = make_game_with_player(vec![]);
        let mut step = StepReferee::new();
        step.goto_label_on_end = "NO_FOUL".into();
        step.injury_result_defender = Some(make_injury_result(Some([4, 4]), false, None));

        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep, "armor doubles → referee spots foul");
        assert!(outcome.events.iter().any(|e| matches!(
            e, GameEvent::RefereeSpotsFoul { referee_spots_foul: true, .. }
        )));
    }

    #[test]
    fn no_armor_doubles_referee_misses() {
        let (mut game, _) = make_game_with_player(vec![]);
        let mut step = StepReferee::new();
        step.goto_label_on_end = "NO_FOUL".into();
        step.injury_result_defender = Some(make_injury_result(Some([3, 4]), false, None));

        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::GotoLabel);
        assert_eq!(outcome.goto_label.as_deref(), Some("NO_FOUL"));
        assert!(outcome.events.iter().any(|e| matches!(
            e, GameEvent::RefereeSpotsFoul { referee_spots_foul: false, .. }
        )));
    }

    #[test]
    fn injury_doubles_spots_foul_when_armor_broken() {
        let (mut game, _) = make_game_with_player(vec![]);
        let mut step = StepReferee::new();
        step.goto_label_on_end = "NO_FOUL".into();
        // armor_roll: no doubles → no spot yet; armor_broken=true; injury_roll: doubles → spot
        step.injury_result_defender = Some(make_injury_result(Some([3, 4]), true, Some([2, 2])));

        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
    }

    #[test]
    fn injury_doubles_not_checked_when_armor_not_broken() {
        let (mut game, _) = make_game_with_player(vec![]);
        let mut step = StepReferee::new();
        step.goto_label_on_end = "NO_FOUL".into();
        // armor_roll: no doubles; armor_broken=false; injury_roll: doubles (should NOT trigger)
        step.injury_result_defender = Some(make_injury_result(Some([3, 4]), false, Some([2, 2])));

        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::GotoLabel, "injury doubles ignored when armor not broken");
    }

    #[test]
    fn sneaky_git_prevents_armor_roll_check_when_armor_intact() {
        let (mut game, _) = make_game_with_player(vec![SkillId::SneakyGit]);
        let mut step = StepReferee::new();
        step.goto_label_on_end = "NO_FOUL".into();
        // SneakyGit + armor NOT broken + ban_to_ko=false → skip armor roll check
        step.injury_result_defender = Some(make_injury_result(Some([4, 4]), false, None));

        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::GotoLabel,
            "SneakyGit with intact armor should prevent doubles check");
    }

    #[test]
    fn sneaky_git_does_not_prevent_check_when_armor_broken() {
        let (mut game, _) = make_game_with_player(vec![SkillId::SneakyGit]);
        let mut step = StepReferee::new();
        step.goto_label_on_end = "NO_FOUL".into();
        // SneakyGit but armor WAS broken → check armor roll doubles anyway
        step.injury_result_defender = Some(make_injury_result(Some([3, 3]), true, None));

        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep, "armor doubles when armor broken overrides SneakyGit protection");
    }

    #[test]
    fn no_armor_roll_returns_goto_label() {
        let (mut game, _) = make_game_with_player(vec![]);
        let mut step = StepReferee::new();
        step.goto_label_on_end = "END".into();
        // No armor roll at all — no doubles possible
        step.injury_result_defender = Some(make_injury_result(None, false, None));

        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::GotoLabel);
    }

    #[test]
    fn referee_report_event_always_emitted() {
        let (mut game, _) = make_game_with_player(vec![]);
        let mut step = StepReferee::new();
        step.goto_label_on_end = "END".into();
        step.injury_result_defender = Some(make_injury_result(Some([2, 5]), false, None));

        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert!(
            outcome.events.iter().any(|e| matches!(e, GameEvent::RefereeSpotsFoul { .. })),
            "RefereeSpotsFoul event must always be emitted when injury result is set"
        );
    }

    #[test]
    fn referee_spots_foul_adds_referee_report() {
        let (mut game, _) = make_game_with_player(vec![]);
        let mut step = StepReferee::new();
        step.goto_label_on_end = "NO_FOUL".into();
        // Armor doubles → referee spots foul
        step.injury_result_defender = Some(make_injury_result(Some([3, 3]), false, None));
        step.start(&mut game, &mut GameRng::new(0));
        assert!(
            game.report_list.has_report(ReportId::REFEREE),
            "REFEREE report must be added when referee spots the foul"
        );
    }

    #[test]
    fn referee_misses_foul_also_adds_referee_report() {
        let (mut game, _) = make_game_with_player(vec![]);
        let mut step = StepReferee::new();
        step.goto_label_on_end = "NO_FOUL".into();
        // No doubles → referee misses
        step.injury_result_defender = Some(make_injury_result(Some([2, 5]), false, None));
        step.start(&mut game, &mut GameRng::new(0));
        assert!(
            game.report_list.has_report(ReportId::REFEREE),
            "REFEREE report must be added even when referee misses the foul"
        );
    }
}
