use crate::client::report::report_message_base::{map_to_local, print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::bb2025::report_punt_direction::ReportPuntDirection;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `PuntDirectionMessage.java`.
pub struct PuntDirectionMessage;

impl ReportMessage for PuntDirectionMessage {
    type Report = ReportPuntDirection;

    fn report_id(&self) -> ReportId {
        ReportId::PUNT_DIRECTION_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let direction_roll = report.get_direction_roll();
        let player = game.player(report.get_player_id());

        if let Some(direction) = report.get_direction() {
            let direction_name = map_to_local(direction).name();
            status_report.println_indent_style(
                indent,
                TextStyle::ROLL,
                &format!("Punt Direction Roll [ {direction_roll} ] {direction_name}"),
            );
            print_player(status_report, game, indent + 1, false, player);
            let mut status = format!(" punts the ball {direction_name}");
            if report.is_out_of_bounds() {
                status.push_str(" putting it out of bounds");
            }
            status.push('.');
            status_report.println_indent(indent + 1, &status);
        } else {
            print_player(status_report, game, indent + 1, false, player);
            status_report.println_indent(indent + 1, " intentionally punts the ball out of bounds.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Direction, PlayerGender, PlayerType, Rules};
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
        let home = make_team("home", vec![make_player("p1", "Punter")]);
        let away = make_team("away", vec![]);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn direction_present_prints_roll_and_direction() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPuntDirection::new(Some(Direction::North), 3, "p1".into(), false);
        PuntDirectionMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.starts_with("Punt Direction Roll [ 3 ]")));
        assert!(texts.iter().any(|t| t.contains("punts the ball")));
    }

    #[test]
    fn out_of_bounds_appends_text_when_direction_present() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPuntDirection::new(Some(Direction::North), 3, "p1".into(), true);
        PuntDirectionMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.contains("putting it out of bounds")));
    }

    #[test]
    fn no_direction_intentionally_out_of_bounds() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPuntDirection::new(None, 0, "p1".into(), true);
        PuntDirectionMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.contains(&" intentionally punts the ball out of bounds.".to_string()));
        assert!(!texts.iter().any(|t| t.starts_with("Punt Direction Roll")));
    }

    #[test]
    fn player_is_printed() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPuntDirection::new(Some(Direction::North), 3, "p1".into(), false);
        PuntDirectionMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.contains(&"Punter".to_string()));
    }

    #[test]
    fn report_id_is_punt_direction_roll() {
        assert_eq!(PuntDirectionMessage.report_id(), ReportId::PUNT_DIRECTION_ROLL);
    }
}
