use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_mechanics::injury_mechanic::InjuryMechanic as InjuryMechanicTrait;
use ffb_model::enums::Rules;
use ffb_model::model::game::Game;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_raise_dead::ReportRaiseDead;

/// 1:1 translation of `RaiseDeadMessage.java`.
pub struct RaiseDeadMessage;

impl ReportMessage for RaiseDeadMessage {
    type Report = ReportRaiseDead;

    fn report_id(&self) -> ReportId {
        ReportId::RAISE_DEAD
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let raised_player = game.player(report.get_player_id());
        let indent = status_report.get_indent();
        print_player(status_report, game, indent, false, raised_player);
        if report.is_nurgles_rot() {
            let message = match game.rules {
                Rules::Bb2016 => ffb_mechanics::bb2016::injury_mechanic::InjuryMechanic::new().raised_by_nurgle_message(),
                Rules::Bb2020 => ffb_mechanics::bb2020::injury_mechanic::InjuryMechanic::new().raised_by_nurgle_message(),
                Rules::Bb2025 | Rules::Common => ffb_mechanics::bb2025::injury_mechanic::InjuryMechanic::new().raised_by_nurgle_message(),
            };
            status_report.print_indent(indent, &message);
        } else {
            status_report.print_indent(indent, " is raised from the dead to join team ");
        }

        if raised_player.is_some_and(|p| game.team_home.has_player(&p.id)) {
            status_report.print_indent_style(indent, TextStyle::HOME, &game.team_home.name);
        } else {
            status_report.print_indent_style(indent, TextStyle::AWAY, &game.team_away.name);
        }

        status_report.println_indent_style(
            indent,
            TextStyle::NONE,
            &format!(" as a {}.", report.get_position().unwrap_or("")),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, PlayerType, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_player(id: &str, name: &str) -> Player {
        Player {
            id: id.into(),
            name: name.into(),
            gender: PlayerGender::Male,
            player_type: PlayerType::default(),
            ..Default::default()
        }
    }

    fn make_team(id: &str, players: Vec<Player>) -> Team {
        Team {
            id: id.into(),
            name: format!("Team {id}"),
            race: "Human".into(),
            roster_id: "human".into(),
            coach: "Coach".into(),
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
        let home = make_team("home", vec![make_player("p1", "Zombie One")]);
        let away = make_team("away", vec![make_player("p2", "Zombie Two")]);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn report_id_is_raise_dead() {
        assert_eq!(RaiseDeadMessage.report_id(), ReportId::RAISE_DEAD);
    }

    #[test]
    fn raises_home_player_without_nurgles_rot() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportRaiseDead::new("p1".into(), Some("Zombie".into()), false);
        RaiseDeadMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == " is raised from the dead to join team "));
        assert!(texts.iter().any(|t| t == "Team home"));
        assert!(texts.iter().any(|t| t == " as a Zombie."));
    }

    #[test]
    fn raises_away_player_with_nurgles_rot() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportRaiseDead::new("p2".into(), Some("Rotter".into()), true);
        RaiseDeadMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == "Team away"));
        assert!(texts.iter().any(|t| t == " as a Rotter."));
        // Nurgle's Rot branch replaces the plain "is raised from the dead" text.
        assert!(!texts.iter().any(|t| t == " is raised from the dead to join team "));
    }

    #[test]
    fn text_style_reflects_home_vs_away() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportRaiseDead::new("p1".into(), Some("Zombie".into()), false);
        RaiseDeadMessage.render(&mut status_report, &game, &report);
        let home_run = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some("Team home")).unwrap();
        assert_eq!(home_run.text_style, Some(TextStyle::HOME));
    }
}
