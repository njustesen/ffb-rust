use ffb_model::enums::ApothecaryMode;
use ffb_model::model::game::Game;
use ffb_model::option::{game_option_id, game_option_string};
use ffb_model::util::rng::GameRng;
use ffb_model::report::report_foul::ReportFoul;
use crate::action::Action;
use crate::drop_player_context::DropPlayerContext;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_injury::handle_injury_by_name;

/// 1:1 translation of com.fumbbl.ffb.server.step.mixed.foul.StepFoul.
///
/// Resolves the actual foul: rolls armor + injury for the defender, then publishes
/// DROP_PLAYER_CONTEXT so StepHandleDropPlayerContext can drop the player.
///
/// Published: DROP_PLAYER_CONTEXT. Conditionally: END_TURN=true when armor broken
/// and the CHAINSAW_TURNOVER option is ALL_AV_BREAKS.
///
/// Java: @RulesCollection(BB2020, BB2025)
pub struct StepFoul {
    /// Java: usingChainsaw — set by USING_CHAINSAW parameter
    pub using_chainsaw: bool,
}

impl StepFoul {
    pub fn new() -> Self { Self { using_chainsaw: false } }
}

impl Default for StepFoul {
    fn default() -> Self { Self::new() }
}

impl Step for StepFoul {
    fn id(&self) -> StepId { StepId::Foul }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::UsingChainsaw(v) => { self.using_chainsaw = *v; true }
            _ => false,
        }
    }
}

impl StepFoul {
    fn execute_step(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let defender_id = match game.defender_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };
        // Java: getResult().addReport(new ReportFoul(game.getDefenderId()))
        game.report_list.add(ReportFoul::new(defender_id.clone()));
        let attacker_id = game.acting_player.player_id.clone();

        let defender_coord = game.field_model.player_coordinate(&defender_id)
            .unwrap_or(ffb_model::types::FieldCoordinate::new(0, 0));

        // Java: getGameState().getPrayerState().hasFoulingFrenzy(game.getActingTeam())
        //       ? new InjuryTypeFoulForSpp(usingChainsaw) : new InjuryTypeFoul(usingChainsaw)
        let acting_team_id = game.active_team().id.clone();
        let fouling_frenzy = game.prayer_state.has_fouling_frenzy(&acting_team_id);
        let injury_type_name = match (fouling_frenzy, self.using_chainsaw) {
            (true, true) => "InjuryTypeFoulChainsawForSpp",
            (true, false) => "InjuryTypeFoulForSpp",
            (false, true) => "InjuryTypeFoulChainsaw",
            (false, false) => "InjuryTypeFoul",
        };

        let injury_result = handle_injury_by_name(
            game, rng, injury_type_name,
            attacker_id.as_deref(),
            &defender_id,
            defender_coord,
            None, None,
            ApothecaryMode::Defender,
        );

        let mut out = StepOutcome::next();

        // Java: String chainsawOption = game.getOptions().getOptionWithDefault(CHAINSAW_TURNOVER).getValueAsString();
        //       if (isArmorBroken && CHAINSAW_TURNOVER_ALL_AV_BREAKS.equalsIgnoreCase(chainsawOption))
        //           publishParameter(END_TURN, true)
        let chainsaw_option = game.options.get(game_option_id::CHAINSAW_TURNOVER)
            .unwrap_or(game_option_string::CHAINSAW_TURNOVER_KICKBACK);
        if injury_result.injury_context().is_armor_broken()
            && chainsaw_option.eq_ignore_ascii_case(game_option_string::CHAINSAW_TURNOVER_ALL_AV_BREAKS)
        {
            out = out.publish(StepParameter::EndTurn(true));
        }

        // Java: new DropPlayerContext(injuryResult, defenderId, DEFENDER, eligibleForSafePairsOfHands=true)
        let dpc = DropPlayerContext {
            injury_result: Some(Box::new(injury_result)),
            player_id: Some(defender_id),
            apothecary_mode: Some(ApothecaryMode::Defender),
            eligible_for_safe_pair_of_hands: true,
            ..DropPlayerContext::new()
        };
        out.publish(StepParameter::DropPlayerContext(Box::new(dpc)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::{Rules, PS_STANDING};
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::types::FieldCoordinate;
    use ffb_model::report::report_id::ReportId;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_player(game: &mut Game, team: &str, id: &str) {
        let p = Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
};
        if team == "home" {
            game.team_home.players.push(p);
        } else {
            game.team_away.players.push(p);
        }
        game.field_model.set_player_coordinate(id, FieldCoordinate::new(5, 5));
        game.field_model.set_player_state(id, ffb_model::enums::PlayerState::new(PS_STANDING));
    }

    #[test]
    fn foul_report_added_when_defender_present() {
        let mut game = make_game();
        add_player(&mut game, "home", "attacker");
        add_player(&mut game, "away", "defender");
        game.acting_player.player_id = Some("attacker".into());
        game.defender_id = Some("defender".into());
        let mut step = StepFoul::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(
            game.report_list.has_report(ReportId::FOUL),
            "should add ReportFoul when defender is present"
        );
    }

    #[test]
    fn no_foul_report_when_no_defender() {
        let mut game = make_game();
        let mut step = StepFoul::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(
            !game.report_list.has_report(ReportId::FOUL),
            "should not add ReportFoul when no defender"
        );
    }

    #[test]
    fn no_defender_returns_next_step() {
        let mut game = make_game();
        let mut step = StepFoul::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.is_empty());
    }

    #[test]
    fn publishes_drop_player_context() {
        let mut game = make_game();
        add_player(&mut game, "home", "attacker");
        add_player(&mut game, "away", "defender");
        game.acting_player.player_id = Some("attacker".into());
        game.defender_id = Some("defender".into());
        let mut step = StepFoul::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::DropPlayerContext(_))));
    }

    #[test]
    fn set_parameter_using_chainsaw_accepted() {
        let mut step = StepFoul::new();
        assert!(step.set_parameter(&StepParameter::UsingChainsaw(true)));
        assert!(step.using_chainsaw);
    }

    #[test]
    fn unrelated_parameter_rejected() {
        let mut step = StepFoul::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }

    #[test]
    fn drop_player_context_has_correct_defender_id() {
        let mut game = make_game();
        add_player(&mut game, "home", "att");
        add_player(&mut game, "away", "def");
        game.acting_player.player_id = Some("att".into());
        game.defender_id = Some("def".into());
        let mut step = StepFoul::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        let dpc = out.published.iter().find_map(|p| {
            if let StepParameter::DropPlayerContext(ctx) = p { Some(ctx.clone()) } else { None }
        }).expect("DROP_PLAYER_CONTEXT not published");
        assert_eq!(dpc.player_id.as_deref(), Some("def"));
    }

    #[test]
    fn drop_player_context_eligible_for_safe_pair_of_hands() {
        let mut game = make_game();
        add_player(&mut game, "home", "att");
        add_player(&mut game, "away", "def");
        game.acting_player.player_id = Some("att".into());
        game.defender_id = Some("def".into());
        let mut step = StepFoul::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        let dpc = out.published.iter().find_map(|p| {
            if let StepParameter::DropPlayerContext(ctx) = p { Some(ctx.clone()) } else { None }
        }).unwrap();
        assert!(dpc.eligible_for_safe_pair_of_hands);
    }

    /// Regression test: Java's `executeStep` selects `InjuryTypeFoulForSpp` (not plain
    /// `InjuryTypeFoul`) whenever `getPrayerState().hasFoulingFrenzy(actingTeam)` is true,
    /// which grants SPP for the foul injury (`isWorthSpps()==true`). The Rust translation
    /// previously always used `InjuryTypeFoul`/`InjuryTypeFoulChainsaw` regardless of the
    /// Fouling Frenzy prayer, so SPP was never awarded.
    #[test]
    fn fouling_frenzy_prayer_selects_for_spp_injury_type() {
        let mut game = make_game();
        add_player(&mut game, "home", "att");
        add_player(&mut game, "away", "def");
        game.acting_player.player_id = Some("att".into());
        game.defender_id = Some("def".into());
        game.home_playing = true;
        game.prayer_state.add_fouling_frenzy(&game.team_home.id.clone());
        let mut step = StepFoul::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        let dpc = out.published.iter().find_map(|p| {
            if let StepParameter::DropPlayerContext(ctx) = p { Some(ctx.clone()) } else { None }
        }).unwrap();
        let ir = dpc.injury_result.expect("injury result must be set");
        assert!(ir.injury_context().is_worth_spps, "Fouling Frenzy foul must be worth SPP");
    }

    #[test]
    fn without_fouling_frenzy_foul_is_not_worth_spp() {
        let mut game = make_game();
        add_player(&mut game, "home", "att");
        add_player(&mut game, "away", "def");
        game.acting_player.player_id = Some("att".into());
        game.defender_id = Some("def".into());
        let mut step = StepFoul::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        let dpc = out.published.iter().find_map(|p| {
            if let StepParameter::DropPlayerContext(ctx) = p { Some(ctx.clone()) } else { None }
        }).unwrap();
        let ir = dpc.injury_result.expect("injury result must be set");
        assert!(!ir.injury_context().is_worth_spps, "foul without Fouling Frenzy must not be worth SPP");
    }

    /// Regression test: Java only sets END_TURN=true on an armor break when the
    /// `CHAINSAW_TURNOVER` game option is `allAvBreaks`. The Rust translation previously
    /// hardcoded that behavior unconditionally for any armor break, ignoring the option.
    #[test]
    fn chainsaw_turnover_never_option_suppresses_end_turn_on_armor_break() {
        let mut game = make_game();
        // armour 2 guarantees an armor break regardless of roll
        let p = Player {
            id: "def".into(), name: "def".into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 2,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };
        game.team_away.players.push(p);
        game.field_model.set_player_coordinate("def", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("def", ffb_model::enums::PlayerState::new(PS_STANDING));
        add_player(&mut game, "home", "att");
        game.acting_player.player_id = Some("att".into());
        game.defender_id = Some("def".into());
        game.options.set(
            ffb_model::option::game_option_id::CHAINSAW_TURNOVER,
            ffb_model::option::game_option_string::CHAINSAW_TURNOVER_NEVER,
        );
        let mut step = StepFoul::new();
        let out = step.start(&mut game, &mut GameRng::new(1));
        let dpc = out.published.iter().find_map(|p| {
            if let StepParameter::DropPlayerContext(ctx) = p { Some(ctx.clone()) } else { None }
        }).unwrap();
        assert!(dpc.injury_result.unwrap().injury_context().is_armor_broken(), "armour 2 must always break");
        assert!(
            !out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))),
            "CHAINSAW_TURNOVER_NEVER must suppress END_TURN even on armor break"
        );
    }
}
