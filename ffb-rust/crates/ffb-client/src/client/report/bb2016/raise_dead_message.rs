use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_mechanics::bb2016::injury_mechanic::InjuryMechanic;
use ffb_mechanics::injury_mechanic::InjuryMechanic as InjuryMechanicTrait;
use ffb_model::model::game::Game;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_raise_dead::ReportRaiseDead;

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
            let mechanic = InjuryMechanic::new();
            status_report.print_indent(indent, &mechanic.raised_by_nurgle_message());
        } else {
            status_report.print_indent(indent, " is raised from the dead to join team ");
        }
        let is_home = raised_player.is_some_and(|p| game.team_home.has_player(&p.id));
        if is_home {
            status_report.print_indent_style(indent, TextStyle::HOME, &game.team_home.name.clone());
        } else {
            status_report.print_indent_style(indent, TextStyle::AWAY, &game.team_away.name.clone());
        }
        if report.is_nurgles_rot() {
            status_report.println_indent_style(indent, TextStyle::NONE, " as a Rotter in the next game.");
        } else {
            status_report.println_indent_style(indent, TextStyle::NONE, " as a Zombie.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, PlayerType, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team {
            id: id.into(), name: format!("Team {id}"), race: "Human".into(), roster_id: "human".into(), coach: "Coach".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false,
        }
    }

    fn make_game() -> Game {
        let mut home = make_team("home");
        home.players.push(Player {
            id: "p1".into(), name: "Grubb".into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            ..Default::default()
        });
        Game::new(home, make_team("away"), Rules::Bb2016)
    }

    #[test]
    fn get_key_is_raise_dead() {
        assert_eq!(RaiseDeadMessage.get_key(), "raiseDead");
    }

    #[test]
    fn zombie_reports_join_team() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportRaiseDead::new("p1".into(), None, false);
        RaiseDeadMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" is raised from the dead to join team ")));
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" as a Zombie.")));
    }

    #[test]
    fn nurgles_rot_reports_rotter() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportRaiseDead::new("p1".into(), None, true);
        RaiseDeadMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" as a Rotter in the next game.")));
    }
}
