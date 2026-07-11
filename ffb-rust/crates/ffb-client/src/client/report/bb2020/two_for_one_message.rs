use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::bb2020::report_two_for_one::ReportTwoForOne;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `TwoForOneMessage.java`.
pub struct TwoForOneMessage;

impl ReportMessage for TwoForOneMessage {
    type Report = ReportTwoForOne;

    fn report_id(&self) -> ReportId {
        ReportId::TWO_FOR_ONE
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let player = game.player(report.get_player_id());
        let partner = game.player(report.get_partner_id());
        let indent = status_report.get_indent();
        let (verb, reason) = if report.is_used() {
            ("gains", "is injured")
        } else {
            ("loses", "has recovered")
        };
        print_player(status_report, game, indent, false, player);
        status_report.print_indent_style(indent, TextStyle::NONE, &format!(" {verb} Loner (2+) because "));
        print_player(status_report, game, indent, false, partner);
        status_report.println_indent_style(indent, TextStyle::NONE, &format!(" {reason}."));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, PlayerType, Rules};
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

    fn make_team(id: &str, players: Vec<Player>) -> Team {
        Team {
            id: id.into(),
            name: "Team".into(),
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
            make_team("home", vec![make_player("p1"), make_player("p2")]),
            make_team("away", vec![]),
            Rules::Bb2020,
        )
    }

    #[test]
    fn used_gains_loner_because_partner_injured() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportTwoForOne::new("p1".into(), "p2".into(), true);
        TwoForOneMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.iter().any(|t| t.contains(" gains Loner (2+) because ")));
        assert!(texts.iter().any(|t| t.contains(" is injured.")));
    }

    #[test]
    fn not_used_loses_loner_because_partner_recovered() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportTwoForOne::new("p1".into(), "p2".into(), false);
        TwoForOneMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.iter().any(|t| t.contains(" loses Loner (2+) because ")));
        assert!(texts.iter().any(|t| t.contains(" has recovered.")));
    }

    #[test]
    fn prints_both_player_names() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportTwoForOne::new("p1".into(), "p2".into(), true);
        TwoForOneMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.contains(&"Player p1"));
        assert!(texts.contains(&"Player p2"));
    }
}
