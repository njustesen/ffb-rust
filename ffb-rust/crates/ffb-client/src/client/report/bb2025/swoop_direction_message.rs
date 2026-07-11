use crate::client::report::report_message_base::{map_to_local, print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::enums::PlayerGender;
use ffb_model::model::game::Game;
use ffb_model::report::bb2025::report_swoop_direction::ReportSwoopDirection;
use ffb_model::report::report_id::ReportId;

/// Java: `PlayerGender.getDative()`. The Rust `PlayerGender` enum (crates/ffb-model/src/enums/player.rs)
/// only exposes `nominative()`/`genitive()`; the dative form is reimplemented here inline
/// since this is the only translated call site so far.
fn dative(gender: PlayerGender) -> &'static str {
    match gender {
        PlayerGender::Male => "him",
        PlayerGender::Female => "her",
        PlayerGender::Nonbinary => "them",
        PlayerGender::Neutral => "it",
    }
}

/// 1:1 translation of `SwoopDirectionMessage.java`.
pub struct SwoopDirectionMessage;

impl ReportMessage for SwoopDirectionMessage {
    type Report = ReportSwoopDirection;

    fn report_id(&self) -> ReportId {
        ReportId::SWOOP_DIRECTION_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let direction_roll = report.get_direction_roll();
        let direction = report.get_direction();
        let player = game.player(report.get_player_id());
        let direction_name = direction.map(|d| map_to_local(d).name()).unwrap_or("");
        let indent = status_report.get_indent();
        let status = format!("Swoop Direction Roll [ {direction_roll} ] {direction_name}");
        status_report.println_indent_style(indent, TextStyle::ROLL, &status);
        print_player(status_report, game, indent + 1, false, player);
        let mut status = format!(" swoops {direction_name}");
        if report.is_out_of_bounds() {
            let dative = player.map(|p| dative(p.gender)).unwrap_or("");
            status.push_str(" which takes ");
            status.push_str(dative);
            status.push_str(" out of bounds");
        }
        status.push('.');
        status_report.println_indent(indent + 1, &status);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Direction, PlayerType, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_player(id: &str, name: &str, gender: PlayerGender) -> Player {
        Player {
            id: id.into(),
            name: name.into(),
            gender,
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
        let home = make_team("home", vec![make_player("p1", "Swooper", PlayerGender::Male)]);
        let away = make_team("away", vec![]);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn report_id_is_swoop_direction_roll() {
        assert_eq!(SwoopDirectionMessage.report_id(), ReportId::SWOOP_DIRECTION_ROLL);
    }

    #[test]
    fn in_bounds_swoop_has_no_dative_clause() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportSwoopDirection::new(Some(Direction::East), 3, "p1".into(), false);
        SwoopDirectionMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == "Swoop Direction Roll [ 3 ] East"));
        assert!(texts.iter().any(|t| t == " swoops East."));
    }

    #[test]
    fn out_of_bounds_swoop_includes_dative() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportSwoopDirection::new(Some(Direction::North), 6, "p1".into(), true);
        SwoopDirectionMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == " swoops North which takes him out of bounds."));
    }

    #[test]
    fn dative_matches_gender() {
        assert_eq!(dative(PlayerGender::Male), "him");
        assert_eq!(dative(PlayerGender::Female), "her");
        assert_eq!(dative(PlayerGender::Nonbinary), "them");
        assert_eq!(dative(PlayerGender::Neutral), "it");
    }
}
