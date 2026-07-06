/// 1:1 translation of com.fumbbl.ffb.server.mechanic.StateMechanic (abstract base).
///
/// Java abstract class → Rust trait. Concrete edition implementations:
///   bb2025::StateMechanic (BB2025), mixed::StateMechanic (BB2016/BB2020).
///
/// Java methods take `IStep` for report emission + game state access.
/// In Rust the methods take `&mut Game` directly.
/// Notes:
///   - add_apothecaries / add_re_rolls: wired via TurnData.inducement_set
///   - ReportLeader: caller responsibility — emitted when update_leader_re_rolls_for_team returns Some
///   - ReportPumpUpTheCrowdReRoll: wired via GameEvent::PumpUpTheCrowdReRoll in handle_injury_side_effects
use ffb_model::enums::LeaderState;
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::model::team::Team;
use ffb_model::model::field_model::FieldModel;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::inducement::usage::Usage;
use ffb_model::injury::context::InjuryModification;
use ffb_model::report::SkipInjuryParts;
use ffb_model::report::mixed::report_injury::ReportInjury;
use crate::injury::InjuryContext;
use crate::injury::InjuryResult;

pub trait StateMechanic: Send + Sync {
    // ── Abstract (edition-specific) ───────────────────────────────────────────

    /// Java: updateLeaderReRollsForTeam(TurnData, Team, FieldModel, IStep).
    /// `home_team` selects which TurnData + Team to use.
    /// Returns Some(new_state) when the leader state changed (caller adds ReportLeader).
    /// Returns None when no change.
    fn update_leader_re_rolls_for_team(
        &self,
        game: &mut Game,
        home_team: bool,
    ) -> Option<LeaderState>;

    /// Java: startHalf(IStep, int).
    /// Mutates game state: half counter, turn numbers, home_playing, ball cleared,
    /// leader state reset. Returns inducement-registration events (wandering apos, extra training).
    fn start_half(&self, game: &mut Game, half: i32) -> Vec<GameEvent>;

    /// Java: reportInjury(IStep, InjuryResult).
    /// 1:1 translation of bb2025/StateMechanic.reportInjury().
    fn report_injury(&self, game: &mut Game, injury_result: &mut InjuryResult) {
        let ctx = injury_result.injury_context();
        let pre_regen = injury_result.is_pre_regeneration();

        // Determine which parts of the injury to skip in the report.
        let mut skip = if pre_regen { SkipInjuryParts::Cas } else { SkipInjuryParts::EverythingButCas };

        if ctx.modification != InjuryModification::NONE {
            // injuryContext IS a ModifiedInjuryContext
            if ctx.modification == InjuryModification::INJURY {
                skip = if pre_regen { SkipInjuryParts::ArmourAndCas } else { SkipInjuryParts::EverythingButCas };
            }
        } else if injury_result.injury_context().modified_injury_context.is_some() {
            let mod_ctx_modification = injury_result.injury_context()
                .modified_injury_context.as_ref().unwrap().modification;
            if injury_result.is_already_reported() {
                skip = match mod_ctx_modification {
                    InjuryModification::ARMOUR => if pre_regen { SkipInjuryParts::ArmourAndCas } else { SkipInjuryParts::Armour },
                    InjuryModification::INJURY => SkipInjuryParts::ArmourAndInjury,
                    InjuryModification::NONE => skip,
                };
                injury_result.set_already_reported(false);
            } else {
                // playSound = false (client-only: no-op in headless)
                skip = match mod_ctx_modification {
                    InjuryModification::ARMOUR => SkipInjuryParts::Injury,
                    InjuryModification::INJURY => SkipInjuryParts::Cas,
                    InjuryModification::NONE => skip,
                };
            }
        }

        if injury_result.is_already_reported() {
            return;
        }

        let report = build_report_injury(injury_result.injury_context(), skip);
        game.report_list.add(report);
        // Java: step.getResult().setSound() — client-only, no-op in headless
        injury_result.set_already_reported(true);
    }

    /// Java: handlePumpUp(IStep, InjuryResult).
    /// Returns true if a pump-up re-roll was granted.
    /// Takes `&InjuryContext` (both `injury::InjuryResult` and `injury_result::InjuryResult` expose this via `.injury_context()`).
    fn handle_pump_up(&self, game: &mut Game, injury_context: &InjuryContext) -> bool;

    // ── Concrete helpers (shared, from base class) ────────────────────────────

    /// Java: `addApothecaries(IStep, boolean)`.
    /// Sets apothecaries from team, then adds wandering apothecaries and plague doctors
    /// from InducementSet (if any).
    ///
    /// Returns `GameEvent::Inducement` for each registered inducement (wandering apo, plague doctor).
    fn add_apothecaries(&self, game: &mut Game, home_team: bool) -> Vec<GameEvent> {
        let mut events = Vec::new();
        if home_team {
            let apo = game.team_home.apothecaries;
            game.turn_data_home.apothecaries = apo;
            let team_id = game.team_home.id.clone();
            let wandering = game.turn_data_home.inducement_set.get_inducements()
                .into_iter()
                .find(|ind| ind.has_usage(Usage::APOTHECARY))
                .filter(|ind| ind.value > 0);
            if let Some(w) = wandering {
                game.turn_data_home.apothecaries += w.value;
                game.turn_data_home.wandering_apothecaries = w.value;
                // Java: pStep.getResult().addReport(new ReportInducement(team.getId(), entry.getKey(), ...))
                events.push(GameEvent::Inducement {
                    team_id: team_id.clone(),
                    inducement_type: w.type_id.clone(),
                    value: w.value,
                });
            }
            let plague = game.turn_data_home.inducement_set.get_inducements()
                .into_iter()
                .find(|ind| ind.has_usage(Usage::APOTHECARY_JOURNEYMEN))
                .filter(|ind| ind.value > 0);
            if let Some(p) = plague {
                game.turn_data_home.plague_doctors = p.value;
                events.push(GameEvent::Inducement {
                    team_id,
                    inducement_type: p.type_id.clone(),
                    value: p.value,
                });
            }
        } else {
            let apo = game.team_away.apothecaries;
            game.turn_data_away.apothecaries = apo;
            let team_id = game.team_away.id.clone();
            let wandering = game.turn_data_away.inducement_set.get_inducements()
                .into_iter()
                .find(|ind| ind.has_usage(Usage::APOTHECARY))
                .filter(|ind| ind.value > 0);
            if let Some(w) = wandering {
                game.turn_data_away.apothecaries += w.value;
                game.turn_data_away.wandering_apothecaries = w.value;
                events.push(GameEvent::Inducement {
                    team_id: team_id.clone(),
                    inducement_type: w.type_id.clone(),
                    value: w.value,
                });
            }
            let plague = game.turn_data_away.inducement_set.get_inducements()
                .into_iter()
                .find(|ind| ind.has_usage(Usage::APOTHECARY_JOURNEYMEN))
                .filter(|ind| ind.value > 0);
            if let Some(p) = plague {
                game.turn_data_away.plague_doctors = p.value;
                events.push(GameEvent::Inducement {
                    team_id,
                    inducement_type: p.type_id.clone(),
                    value: p.value,
                });
            }
        }
        events
    }

    /// Java: `addReRolls(IStep, boolean)`.
    /// Sets rerolls from team, then adds extra training re-rolls from InducementSet (if any).
    ///
    /// Returns `GameEvent::Inducement` for each registered extra-training inducement.
    fn add_re_rolls(&self, game: &mut Game, home_team: bool) -> Vec<GameEvent> {
        let mut events = Vec::new();
        if home_team {
            let rr = game.team_home.rerolls;
            game.turn_data_home.rerolls = rr;
            let team_id = game.team_home.id.clone();
            let extra = game.turn_data_home.inducement_set.get_inducements()
                .into_iter()
                .find(|ind| ind.has_usage(Usage::REROLL))
                .filter(|ind| ind.value > 0);
            if let Some(e) = extra {
                game.turn_data_home.rerolls += e.value;
                // Java: pStep.getResult().addReport(new ReportInducement(team.getId(), entry.getKey(), ...))
                events.push(GameEvent::Inducement {
                    team_id,
                    inducement_type: e.type_id.clone(),
                    value: e.value,
                });
            }
        } else {
            let rr = game.team_away.rerolls;
            game.turn_data_away.rerolls = rr;
            let team_id = game.team_away.id.clone();
            let extra = game.turn_data_away.inducement_set.get_inducements()
                .into_iter()
                .find(|ind| ind.has_usage(Usage::REROLL))
                .filter(|ind| ind.value > 0);
            if let Some(e) = extra {
                game.turn_data_away.rerolls += e.value;
                events.push(GameEvent::Inducement {
                    team_id,
                    inducement_type: e.type_id.clone(),
                    value: e.value,
                });
            }
        }
        events
    }

    /// Java: resetSpecialSkillAtEndOfDrive(Game).
    fn reset_special_skill_at_end_of_drive(&self, game: &mut Game) {
        use ffb_model::enums::SkillUsageType;
        for p in game.team_home.players.iter_mut().chain(game.team_away.players.iter_mut()) {
            p.reset_used_skills(SkillUsageType::OncePerDrive);
        }
    }

    // ── Shared utility ────────────────────────────────────────────────────────

    /// True if any player from `team` is on the playing field (not in the box)
    /// and has the `grantsTeamReRollWhenOnPitch` skill property.
    fn team_has_leader_on_field(&self, team: &Team, field_model: &FieldModel) -> bool {
        team.players.iter().any(|p| {
            field_model
                .player_coordinate(&p.id)
                .map(|c| !c.is_box_coordinate())
                .unwrap_or(false)
                && p.has_skill_property(NamedProperties::GRANTS_TEAM_RE_ROLL_WHEN_ON_PITCH)
        })
    }
}

/// Java: ReportInjury.init(InjuryContext, SkipInjuryParts) — builds a ReportInjury from context.
fn build_report_injury(ctx: &InjuryContext, skip: SkipInjuryParts) -> ReportInjury {
    ReportInjury::new(
        ctx.attacker_id.clone(),
        ctx.defender_id.clone(),
        ctx.injury_type_name.clone().unwrap_or_default(),
        ctx.armor_broken,
        ctx.armor_modifiers.iter().map(|m| format!("{m:?}")).collect(),
        ctx.armor_roll.map(|r| r.to_vec()).unwrap_or_default(),
        ctx.injury_modifiers.iter().map(|m| format!("{m:?}")).collect(),
        ctx.injury_roll.map(|r| r.to_vec()).unwrap_or_default(),
        ctx.casualty_roll.map(|r| r.to_vec()).unwrap_or_default(),
        ctx.serious_injury.map(|s| format!("{s:?}")),
        ctx.casualty_roll_decay.map(|r| r.to_vec()).unwrap_or_default(),
        ctx.serious_injury_decay.map(|s| format!("{s:?}")),
        ctx.original_serious_injury.map(|s| format!("{s:?}")),
        ctx.injury,
        ctx.injury_decay,
        ctx.casualty_modifiers.iter().map(|m| format!("{m:?}")).collect(),
        skip.to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{LeaderState, Rules, ApothecaryMode};
    use ffb_model::report::ReportId;
    use ffb_model::model::game::Game;
    use ffb_model::types::FieldCoordinate;
    use ffb_model::types::RSV_HOME_X;

    struct TestMechanic;
    impl StateMechanic for TestMechanic {
        fn update_leader_re_rolls_for_team(&self, _g: &mut Game, _home: bool) -> Option<LeaderState> { None }
        fn start_half(&self, _g: &mut Game, _half: i32) -> Vec<GameEvent> { vec![] }
        fn handle_pump_up(&self, _g: &mut Game, _ctx: &InjuryContext) -> bool { false }
    }

    fn make_game() -> Game {
        Game::new(
            crate::step::framework::test_team("home", 0),
            crate::step::framework::test_team("away", 0),
            Rules::Bb2025,
        )
    }

    #[test]
    fn report_injury_emits_to_report_list() {
        let m = TestMechanic;
        let mut g = make_game();
        let mut ir = InjuryResult::new(ApothecaryMode::Defender);
        ir.injury_context.defender_id = Some("p1".into());
        ir.injury_context.injury_type_name = Some("REGULAR".into());
        m.report_injury(&mut g, &mut ir);
        assert!(ir.is_already_reported());
        assert!(g.report_list.has_report(ReportId::INJURY));
    }

    #[test]
    fn report_injury_skips_second_call_when_already_reported() {
        let m = TestMechanic;
        let mut g = make_game();
        let mut ir = InjuryResult::new(ApothecaryMode::Defender);
        m.report_injury(&mut g, &mut ir);
        assert_eq!(g.report_list.size(), 1);
        // second call must be a no-op
        m.report_injury(&mut g, &mut ir);
        assert_eq!(g.report_list.size(), 1);
    }

    #[test]
    fn reset_special_skill_does_not_panic() {
        let m = TestMechanic;
        let mut g = make_game();
        m.reset_special_skill_at_end_of_drive(&mut g);
    }

    #[test]
    fn add_apothecaries_sets_from_team() {
        let m = TestMechanic;
        let mut g = make_game();
        g.team_home.apothecaries = 2;
        m.add_apothecaries(&mut g, true);
        assert_eq!(g.turn_data_home.apothecaries, 2);
    }

    #[test]
    fn add_apothecaries_away_team() {
        let m = TestMechanic;
        let mut g = make_game();
        g.team_away.apothecaries = 1;
        m.add_apothecaries(&mut g, false);
        assert_eq!(g.turn_data_away.apothecaries, 1);
    }

    #[test]
    fn add_re_rolls_sets_from_team() {
        let m = TestMechanic;
        let mut g = make_game();
        g.team_home.rerolls = 3;
        m.add_re_rolls(&mut g, true);
        assert_eq!(g.turn_data_home.rerolls, 3);
    }

    #[test]
    fn add_apothecaries_adds_wandering_from_inducement_set() {
        use ffb_model::inducement::inducement::Inducement;
        let m = TestMechanic;
        let mut g = make_game();
        g.team_home.apothecaries = 1;
        // Add a wandering apothecary inducement
        g.turn_data_home.inducement_set.add_inducement(
            Inducement::new("wanderingApothecary", 1, vec![Usage::APOTHECARY])
        );
        m.add_apothecaries(&mut g, true);
        assert_eq!(g.turn_data_home.apothecaries, 2);
        assert_eq!(g.turn_data_home.wandering_apothecaries, 1);
    }

    #[test]
    fn add_apothecaries_adds_plague_doctors() {
        use ffb_model::inducement::inducement::Inducement;
        let m = TestMechanic;
        let mut g = make_game();
        g.turn_data_home.inducement_set.add_inducement(
            Inducement::new("plagueDoctor", 2, vec![Usage::APOTHECARY_JOURNEYMEN])
        );
        m.add_apothecaries(&mut g, true);
        assert_eq!(g.turn_data_home.plague_doctors, 2);
    }

    #[test]
    fn add_re_rolls_adds_extra_training() {
        use ffb_model::inducement::inducement::Inducement;
        let m = TestMechanic;
        let mut g = make_game();
        g.team_home.rerolls = 2;
        g.turn_data_home.inducement_set.add_inducement(
            Inducement::new("extraTraining", 1, vec![Usage::REROLL])
        );
        m.add_re_rolls(&mut g, true);
        assert_eq!(g.turn_data_home.rerolls, 3);
    }

    #[test]
    fn add_re_rolls_away_with_extra_training() {
        use ffb_model::inducement::inducement::Inducement;
        let m = TestMechanic;
        let mut g = make_game();
        g.team_away.rerolls = 1;
        g.turn_data_away.inducement_set.add_inducement(
            Inducement::new("extraTraining", 2, vec![Usage::REROLL])
        );
        m.add_re_rolls(&mut g, false);
        assert_eq!(g.turn_data_away.rerolls, 3);
    }

    #[test]
    fn team_has_leader_on_field_no_players() {
        let m = TestMechanic;
        let g = make_game();
        assert!(!m.team_has_leader_on_field(&g.team_home, &g.field_model));
    }

    #[test]
    fn team_has_leader_on_field_with_player_in_box() {
        let m = TestMechanic;
        let mut g = make_game();
        // Box coordinate → not on field
        g.field_model.set_player_coordinate("p1", FieldCoordinate::new(RSV_HOME_X, 1));
        assert!(!m.team_has_leader_on_field(&g.team_home, &g.field_model));
    }

    #[test]
    fn team_has_leader_on_field_no_leader_skill() {
        let m = TestMechanic;
        let mut g = make_game();
        // Player on pitch but no leader skill
        g.field_model.set_player_coordinate("p1", FieldCoordinate::new(8, 5));
        assert!(!m.team_has_leader_on_field(&g.team_home, &g.field_model));
    }

    // ── Event emission tests ──────────────────────────────────────────────────

    #[test]
    fn add_apothecaries_no_inducement_emits_no_events() {
        let m = TestMechanic;
        let mut g = make_game();
        let events = m.add_apothecaries(&mut g, true);
        assert!(events.is_empty());
    }

    #[test]
    fn add_apothecaries_wandering_emits_inducement_event() {
        use ffb_model::inducement::inducement::Inducement;
        let m = TestMechanic;
        let mut g = make_game();
        g.turn_data_home.inducement_set.add_inducement(
            Inducement::new("wanderingApothecary", 1, vec![Usage::APOTHECARY])
        );
        let events = m.add_apothecaries(&mut g, true);
        assert_eq!(events.len(), 1);
        assert!(matches!(&events[0], GameEvent::Inducement { inducement_type, value: 1, .. }
            if inducement_type == "wanderingApothecary"));
    }

    #[test]
    fn add_apothecaries_plague_doctor_emits_inducement_event() {
        use ffb_model::inducement::inducement::Inducement;
        let m = TestMechanic;
        let mut g = make_game();
        g.turn_data_home.inducement_set.add_inducement(
            Inducement::new("plagueDoctor", 2, vec![Usage::APOTHECARY_JOURNEYMEN])
        );
        let events = m.add_apothecaries(&mut g, true);
        assert_eq!(events.len(), 1);
        assert!(matches!(&events[0], GameEvent::Inducement { inducement_type, value: 2, .. }
            if inducement_type == "plagueDoctor"));
    }

    #[test]
    fn add_apothecaries_wandering_and_plague_emits_two_events() {
        use ffb_model::inducement::inducement::Inducement;
        let m = TestMechanic;
        let mut g = make_game();
        g.turn_data_home.inducement_set.add_inducement(
            Inducement::new("wanderingApothecary", 1, vec![Usage::APOTHECARY])
        );
        g.turn_data_home.inducement_set.add_inducement(
            Inducement::new("plagueDoctor", 1, vec![Usage::APOTHECARY_JOURNEYMEN])
        );
        let events = m.add_apothecaries(&mut g, true);
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn add_apothecaries_away_team_event_carries_away_team_id() {
        use ffb_model::inducement::inducement::Inducement;
        let m = TestMechanic;
        let mut g = make_game();
        g.turn_data_away.inducement_set.add_inducement(
            Inducement::new("wanderingApothecary", 1, vec![Usage::APOTHECARY])
        );
        let events = m.add_apothecaries(&mut g, false);
        assert_eq!(events.len(), 1);
        if let GameEvent::Inducement { team_id, .. } = &events[0] {
            assert_eq!(team_id, &g.team_away.id);
        } else {
            panic!("expected Inducement event");
        }
    }

    #[test]
    fn add_re_rolls_no_inducement_emits_no_events() {
        let m = TestMechanic;
        let mut g = make_game();
        let events = m.add_re_rolls(&mut g, true);
        assert!(events.is_empty());
    }

    #[test]
    fn add_re_rolls_extra_training_emits_inducement_event() {
        use ffb_model::inducement::inducement::Inducement;
        let m = TestMechanic;
        let mut g = make_game();
        g.turn_data_home.inducement_set.add_inducement(
            Inducement::new("extraTraining", 1, vec![Usage::REROLL])
        );
        let events = m.add_re_rolls(&mut g, true);
        assert_eq!(events.len(), 1);
        assert!(matches!(&events[0], GameEvent::Inducement { inducement_type, value: 1, .. }
            if inducement_type == "extraTraining"));
    }

    #[test]
    fn add_re_rolls_away_team_event_carries_away_team_id() {
        use ffb_model::inducement::inducement::Inducement;
        let m = TestMechanic;
        let mut g = make_game();
        g.turn_data_away.inducement_set.add_inducement(
            Inducement::new("extraTraining", 2, vec![Usage::REROLL])
        );
        let events = m.add_re_rolls(&mut g, false);
        assert_eq!(events.len(), 1);
        if let GameEvent::Inducement { team_id, .. } = &events[0] {
            assert_eq!(team_id, &g.team_away.id);
        } else {
            panic!("expected Inducement event");
        }
    }
}
