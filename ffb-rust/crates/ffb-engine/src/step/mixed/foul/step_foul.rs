use ffb_model::enums::ApothecaryMode;
use ffb_model::model::game::Game;
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
/// (stub: defaults to CHAINSAW_TURNOVER_ALL_AV_BREAKS behavior for foul).
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

        // Java: prayerState.hasFoulingFrenzy(actingTeam) ? InjuryTypeFoulForSpp : InjuryTypeFoul
        // (both variants further wrap chainsaw handling via the `usingChainsaw` ctor arg).
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

        // Java: if (isArmorBroken && chainsawOption == ALL_AV_BREAKS) publishParameter(END_TURN, true)
        // Stub: default to ALL_AV_BREAKS behavior.
        if injury_result.injury_context().is_armor_broken() {
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

    /// Regression test: Java picks `InjuryTypeFoulForSpp` over `InjuryTypeFoul` when
    /// `getGameState().getPrayerState().hasFoulingFrenzy(actingTeam)` is true. Before this
    /// fix, the Rust translation always used the non-SPP variant (comment: "Prayer state
    /// not yet ported"), even though `PrayerState::has_fouling_frenzy` was already fully
    /// implemented in `ffb-model` — fouls made under an active Fouling Frenzy prayer never
    /// scored SPPs for the attacker.
    #[test]
    fn fouling_frenzy_uses_spp_injury_type() {
        let mut game = make_game();
        add_player(&mut game, "home", "att");
        add_player(&mut game, "away", "def");
        game.acting_player.player_id = Some("att".into());
        game.defender_id = Some("def".into());
        game.home_playing = true;
        game.prayer_state.add_fouling_frenzy("home");
        let mut step = StepFoul::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        let dpc = out.published.iter().find_map(|p| {
            if let StepParameter::DropPlayerContext(ctx) = p { Some(ctx.clone()) } else { None }
        }).expect("DROP_PLAYER_CONTEXT not published");
        let ir = dpc.injury_result.expect("injury result must be present");
        assert!(ir.injury_context().is_worth_spps, "fouling frenzy foul must be worth SPPs");
    }

    #[test]
    fn without_fouling_frenzy_is_not_worth_spps() {
        let mut game = make_game();
        add_player(&mut game, "home", "att");
        add_player(&mut game, "away", "def");
        game.acting_player.player_id = Some("att".into());
        game.defender_id = Some("def".into());
        game.home_playing = true;
        let mut step = StepFoul::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        let dpc = out.published.iter().find_map(|p| {
            if let StepParameter::DropPlayerContext(ctx) = p { Some(ctx.clone()) } else { None }
        }).expect("DROP_PLAYER_CONTEXT not published");
        let ir = dpc.injury_result.expect("injury result must be present");
        assert!(!ir.injury_context().is_worth_spps, "regular foul must not be worth SPPs");
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
}
