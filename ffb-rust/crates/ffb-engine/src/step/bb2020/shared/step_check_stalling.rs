use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::option::game_option_id;
use ffb_model::option::util_game_option::is_option_enabled;
use ffb_model::report::mixed::report_staller_detected::ReportStallerDetected;
use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds};
use ffb_model::util::pathfinding::path_finder_with_pass_block_support::PathFinderWithPassBlockSupport;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_player::UtilPlayer;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2020.shared.StepCheckStalling.
///
/// Checks if the ball-carrier is stalling (BB2020).
///
/// Java init param: IGNORE_ACTED_FLAG (default: true).
/// Java start(): if performCheck() → findStallingPlayer() → if found, add staller.
/// Then NEXT_STEP.
///
/// Note: `StepParameter::IgnoreActedFlag` is handled via `set_parameter`.
///
/// The whole check is gated on `prayerState.shouldNotStall(actingTeam)`, which only the
/// **Throw a Rock** prayer sets (`bb2020/prayers/throw_a_rock_handler.rs` puts it on the praying
/// team's OPPONENT). So this runs only while that prayer is active, and a detected staller is what
/// `bb2020::StepEndTurn::handle_stallers` later turns into `STALLING_PLAYER`'s d6 rock roll.
pub struct StepCheckStalling {
    /// Java: fIgnoreActedFlag (default true in Java init)
    pub ignore_acted_flag: bool,
}

impl StepCheckStalling {
    pub fn new() -> Self {
        Self {
            ignore_acted_flag: true,
        }
    }

    /// Java: getResult().addReport(new ReportStallerDetected(stallingPlayer.getId()))
    /// Called when a stalling player is found. Wired here for structural completeness;
    /// the caller (start) currently skips detection because pathfinding is not yet ported.
    pub fn report_staller_detected(game: &mut Game, player_id: Option<String>) {
        game.report_list.add(ReportStallerDetected::new(player_id));
    }
}

impl StepCheckStalling {
    /// Java: the `rollAtActivation` set — a player who must roll a negatrait when activated is
    /// never considered to be stalling, because it may not get to act at all.
    const ROLL_AT_ACTIVATION: [&'static str; 4] = [
        NamedProperties::APPLIES_CONFUSION,
        NamedProperties::NEEDS_TO_ROLL_FOR_ACTION_BLOCKING_IS_EASIER,
        NamedProperties::NEEDS_TO_ROLL_FOR_ACTION_BUT_KEEPS_TACKLEZONE,
        NamedProperties::BECOMES_IMMOVABLE,
    ];

    /// Java: `performCheck()`.
    ///
    /// `ENABLE_STALLING_CHECK`'s factory default is TRUE in both engines
    /// (`GameOptionFactory.java:269`), and Java reads it through `getOptionWithDefault`, so this
    /// must go via `is_option_enabled`, which consults the factory default rather than only the
    /// stored map.
    fn perform_check(&self, game: &Game) -> bool {
        let acting_team_id = if game.home_playing { &game.team_home.id } else { &game.team_away.id };
        is_option_enabled(game, game_option_id::ENABLE_STALLING_CHECK)
            && game.prayer_state.should_not_stall(acting_team_id)
            && (self.ignore_acted_flag || game.acting_player.acted())
    }

    /// Java: `findStallingPlayer()` — the suspect, kept only if it is actually considered stalling.
    fn find_stalling_player(game: &Game) -> Option<String> {
        let suspect = Self::find_stalling_suspect(game)?;
        if Self::is_considered_stalling(game, &suspect) { Some(suspect) } else { None }
    }

    /// Java: `findStallingSuspect()` — the acting team's own ball carrier, unless already flagged.
    fn find_stalling_suspect(game: &Game) -> Option<String> {
        let ball_coord = game.field_model.ball_coordinate?;
        let carrier = game.field_model.player_at(ball_coord)?.clone();
        let acting_team = if game.home_playing { &game.team_home } else { &game.team_away };
        if UtilPlayer::has_ball(game, &carrier)
            && acting_team.has_player(&carrier)
            && !game.prayer_state.is_stalling(&carrier)
        {
            Some(carrier)
        } else {
            None
        }
    }

    /// Java: `isConsideredStalling(Game, Player)` -- one short-circuited `&&` chain.
    fn is_considered_stalling(game: &Game, player_id: &str) -> bool {
        if game.acting_player.player_id.as_deref() == Some(player_id) {
            return false;
        }
        let player = match game.player(player_id) { Some(p) => p, None => return false };
        // Java: `player.getSkillsIncludingTemporaryOnes()` flat-mapped to properties, noneMatch.
        // `has_skill_property_in` walks the same set (starting + extra + temporary) and is
        // edition-aware, which matters because these properties are registered per edition.
        if Self::ROLL_AT_ACTIVATION.iter().any(|prop| player.has_skill_property_in(game.rules, prop)) {
            return false;
        }
        if !game.field_model.player_state(player_id).map(|st| st.is_active()).unwrap_or(false) {
            return false;
        }
        let coord = match game.field_model.player_coordinate(player_id) {
            Some(c) => c,
            None => return false,
        };
        // Java: `!ArrayTool.isProvided(findAdjacentPlayersWithTacklezones(game, otherTeam, coord, false))`
        // -- a marked carrier is not stalling, it is pinned.
        let other_team = if game.team_home.has_player(player_id) { &game.team_away } else { &game.team_home };
        if !UtilPlayer::find_adjacent_players_with_tacklezones(game, other_team, coord, false).is_empty() {
            return false;
        }
        Self::has_open_path_to_endzone(game, player)
    }

    /// Java: `hasOpenPathToEndzone(Game, Player)` — a path to ANY square of the opposing endzone,
    /// costed against the player's FULL movement (Java passes `0` as the current move).
    fn has_open_path_to_endzone(game: &Game, player: &ffb_model::model::player::Player) -> bool {
        let endzone = if game.team_home.has_player(&player.id) {
            FieldCoordinateBounds::ENDZONE_AWAY
        } else {
            FieldCoordinateBounds::ENDZONE_HOME
        };
        let end_coords: std::collections::HashSet<FieldCoordinate> =
            endzone.coordinates().into_iter().collect();
        PathFinderWithPassBlockSupport::new()
            .get_shortest_path_for_player(game, &end_coords, player, 0)
            .is_some()
    }
}

impl Default for StepCheckStalling {
    fn default() -> Self { Self::new() }
}

impl Step for StepCheckStalling {
    fn id(&self) -> StepId { StepId::CheckStalling }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: if (performCheck()) { stallingPlayer = findStallingPlayer(); if (stallingPlayer != null) {
        //           addReport(new ReportStallerDetected(stallingPlayer.getId()));
        //           prayerState.addStaller(stallingPlayer); } }
        if self.perform_check(game) {
            if let Some(player_id) = Self::find_stalling_player(game) {
                Self::report_staller_detected(game, Some(player_id.clone()));
                game.prayer_state.add_staller(&player_id);
            }
        }
        StepOutcome::next()
    }

    fn handle_command(&mut self, _action: &Action, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        StepOutcome::next()
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::IgnoreActedFlag(v) => { self.ignore_acted_flag = *v; true }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::Rules;
    use ffb_model::report::report_id::ReportId;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2020)
    }

    #[test]
    fn new_has_ignore_acted_flag_true() {
        let step = StepCheckStalling::new();
        assert!(step.ignore_acted_flag);
    }

    #[test]
    fn start_returns_next_step() {
        let mut game = make_game();
        let mut step = StepCheckStalling::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn handle_command_returns_next_step() {
        let mut game = make_game();
        let mut step = StepCheckStalling::new();
        let out = step.handle_command(&Action::Acknowledge, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_ignore_acted_flag_accepted() {
        let mut step = StepCheckStalling::new();
        assert!(step.ignore_acted_flag); // default = true
        assert!(step.set_parameter(&StepParameter::IgnoreActedFlag(false)));
        assert!(!step.ignore_acted_flag);
        assert!(step.set_parameter(&StepParameter::IgnoreActedFlag(true)));
        assert!(step.ignore_acted_flag);
    }

    #[test]
    fn set_parameter_unknown_returns_false() {
        let mut step = StepCheckStalling::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }

    #[test]
    fn report_staller_detected_with_player_id_emits_report() {
        let mut game = make_game();
        StepCheckStalling::report_staller_detected(&mut game, Some("home_01".into()));
        assert!(game.report_list.has_report(ReportId::STALLER_DETECTED));
    }

    #[test]
    fn report_staller_detected_without_player_id_emits_report() {
        let mut game = make_game();
        StepCheckStalling::report_staller_detected(&mut game, None);
        assert!(game.report_list.has_report(ReportId::STALLER_DETECTED));
    }
}
