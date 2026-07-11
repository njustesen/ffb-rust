use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use ffb_model::model::game::Game;
use ffb_model::report::report_game_options::ReportGameOptions;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `GameOptionsMessage.java`. Java note: "no longer used, remains
/// for compatibility with older versions" — the render body is intentionally empty.
pub struct GameOptionsMessage;

impl ReportMessage for GameOptionsMessage {
    type Report = ReportGameOptions;

    fn report_id(&self) -> ReportId {
        ReportId::GAME_OPTIONS
    }

    fn render(&self, _status_report: &mut StatusReport, _game: &Game, _report: &Self::Report) {
        // Java: render(ReportGameOptions report) { } — empty body.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::game::Game;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team {
            id: id.to_string(),
            name: id.to_string(),
            race: "human".into(),
            roster_id: "human".into(),
            coach: "coach".into(),
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
            special_rules: Vec::new(),
            players: Vec::new(),
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_game() -> Game {
        Game::new(make_team("home"), make_team("away"), Rules::Bb2025)
    }

    #[test]
    fn report_id_matches() {
        assert_eq!(GameOptionsMessage.report_id(), ReportId::GAME_OPTIONS);
    }

    #[test]
    fn render_produces_no_output() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportGameOptions::new(true, 60, false, true, false, true);
        GameOptionsMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.is_empty());
    }

    #[test]
    fn render_does_not_change_indent() {
        let mut status_report = StatusReport::new();
        status_report.set_indent(3);
        let game = make_game();
        let report = ReportGameOptions::default();
        GameOptionsMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.get_indent(), 3);
    }
}
