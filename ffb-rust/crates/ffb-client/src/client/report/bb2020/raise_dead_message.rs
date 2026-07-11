use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_mechanics::injury_mechanic::InjuryMechanic as InjuryMechanicTrait;
use ffb_model::enums::Rules;
use ffb_model::model::game::Game;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_raise_dead::ReportRaiseDead;

/// Java: `game.getFactory(FactoryType.Factory.MECHANIC).forName(Mechanic.Type.INJURY.name())`.
/// There is no `injury_mechanic_for(rules)` helper in `ffb_engine::mechanic` (only bb2020
/// needs the lookup here), so this is a local port of the same dispatch pattern used by
/// `ffb_engine::mechanic::game_mechanic_for`.
fn injury_mechanic_for(rules: Rules) -> Box<dyn InjuryMechanicTrait> {
    match rules {
        Rules::Bb2025 | Rules::Common => Box::new(ffb_mechanics::bb2025::injury_mechanic::InjuryMechanic::new()),
        Rules::Bb2020 => Box::new(ffb_mechanics::bb2020::injury_mechanic::InjuryMechanic::new()),
        Rules::Bb2016 => Box::new(ffb_mechanics::bb2016::injury_mechanic::InjuryMechanic::new()),
    }
}

/// 1:1 translation of `RaiseDeadMessage.java`.
pub struct RaiseDeadMessage;

impl ReportMessage for RaiseDeadMessage {
    type Report = ReportRaiseDead;

    fn report_id(&self) -> ReportId {
        ReportId::RAISE_DEAD
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let raised_player = game.player(report.get_player_id());
        print_player(status_report, game, indent, false, raised_player);
        if report.is_nurgles_rot() {
            let mechanic = injury_mechanic_for(game.rules);
            status_report.print_indent(indent, &mechanic.raised_by_nurgle_message());
        } else {
            status_report.print_indent(indent, " is raised from the dead to join team ");
        }
        let Some(raised_player) = raised_player else { return };
        let is_home = game.team_home.has_player(&raised_player.id);
        if is_home {
            status_report.print_indent_style(indent, TextStyle::HOME, &game.team_home.name.clone());
        } else {
            status_report.print_indent_style(indent, TextStyle::AWAY, &game.team_away.name.clone());
        }
        // java: `team.getRoster().getRaisedRosterPosition()` — `Team` only stores a
        // `roster_id: String` (see model/team.rs), and `Roster`/`RosterPosition` (see
        // model/roster.rs, model/roster_position.rs) have no concept of a "raised roster
        // position" (Zombie/Rotter slot) yet, nor is a `Roster` registry reachable from
        // `Game`. There is no way to resolve this lookup, so it is skipped — matching the
        // Java behavior when `getRaisedRosterPosition()` returns null — and the default
        // position name below is always used.
        let position_name = if report.is_nurgles_rot() { "Rotter" } else { "Zombie" };
        status_report.println_indent_style(indent, TextStyle::NONE, &format!(" as a {position_name}."));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, PlayerType};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_player(id: &str) -> Player {
        Player {
            id: id.into(),
            name: format!("Player {id}"),
            nr: 1,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6,
            strength: 3,
            agility: 3,
            passing: 4,
            armour: 8,
            ..Default::default()
        }
    }

    fn make_team(id: &str, name: &str, players: Vec<Player>) -> Team {
        Team {
            id: id.into(),
            name: name.into(),
            race: String::new(),
            roster_id: String::new(),
            coach: String::new(),
            rerolls: 0,
            apothecaries: 0,
            bribes: 0,
            master_chefs: 0,
            prayers_to_nuffle: 0,
            bloodweiser_kegs: 0,
            riotous_rookies: 0,
            cheerleaders: 0,
            assistant_coaches: 0,
            fan_factor: 0,
            dedicated_fans: 0,
            team_value: 0,
            treasury: 0,
            special_rules: vec![],
            players,
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_game() -> Game {
        Game::new(
            make_team("home", "Home Team", vec![make_player("raised")]),
            make_team("away", "Away Team", vec![]),
            Rules::Bb2020,
        )
    }

    fn texts(status_report: &StatusReport) -> Vec<&str> {
        status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect()
    }

    #[test]
    fn plain_zombie_join_home_team() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportRaiseDead::new("raised".into(), None, false);
        RaiseDeadMessage.render(&mut status_report, &game, &report);
        let t = texts(&status_report);
        assert!(t.iter().any(|s| s.contains("is raised from the dead to join team")));
        assert!(t.contains(&"Home Team"));
        assert!(t.contains(&" as a Zombie."));
    }

    #[test]
    fn nurgles_rot_uses_rotter_and_nurgle_message() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportRaiseDead::new("raised".into(), None, true);
        RaiseDeadMessage.render(&mut status_report, &game, &report);
        let t = texts(&status_report);
        assert!(t.iter().any(|s| s.contains("Plague Ridden")));
        assert!(t.contains(&" as a Rotter."));
    }

    #[test]
    fn away_team_player_uses_away_style() {
        let game = Game::new(
            make_team("home", "Home Team", vec![]),
            make_team("away", "Away Team", vec![make_player("raised")]),
            Rules::Bb2020,
        );
        let mut status_report = StatusReport::new();
        let report = ReportRaiseDead::new("raised".into(), None, false);
        RaiseDeadMessage.render(&mut status_report, &game, &report);
        let t = texts(&status_report);
        assert!(t.contains(&"Away Team"));
        let away_run = status_report
            .rendered_runs
            .iter()
            .find(|r| r.text.as_deref() == Some("Away Team"))
            .unwrap();
        assert_eq!(away_run.text_style, Some(TextStyle::AWAY));
    }

    #[test]
    fn unknown_player_does_not_panic() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportRaiseDead::new("ghost".into(), None, false);
        RaiseDeadMessage.render(&mut status_report, &game, &report);
        // No player found: intro text still printed, but no team/position lines follow.
        let t = texts(&status_report);
        assert!(!t.iter().any(|s| s.ends_with("Zombie.")));
    }
}
