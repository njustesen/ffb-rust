use crate::client::report::report_message_base::{map_to_local, print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_hit_and_run::ReportHitAndRun;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `HitAndRunMessage.java`.
pub struct HitAndRunMessage;

impl ReportMessage for HitAndRunMessage {
    type Report = ReportHitAndRun;

    fn report_id(&self) -> ReportId {
        ReportId::HIT_AND_RUN
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let player = report.get_player_id().and_then(|id| game.player(id));
        let indent = status_report.get_indent();

        print_player(status_report, game, indent, false, player);
        status_report.print_indent_style(indent, TextStyle::NONE, " moves one square ");
        // java: `mapToLocal(report.getDirection()).getName()` assumes a non-null direction.
        status_report.print_indent_style(indent, TextStyle::NONE, map_to_local(report.get_direction().unwrap()).name());
        status_report.println_indent_style(indent, TextStyle::NONE, ".");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Direction, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn empty_team(id: &str, players: Vec<Player>) -> Team {
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
        Game::new(
            empty_team("home", vec![Player { id: "p1".into(), name: "Grubber".into(), ..Default::default() }]),
            empty_team("away", vec![]),
            Rules::Bb2020,
        )
    }

    #[test]
    fn renders_player_and_direction() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportHitAndRun::new(Some("p1".into()), Some(Direction::North));
        HitAndRunMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[0].text.as_deref(), Some("Grubber"));
        assert_eq!(sr.rendered_runs[1].text.as_deref(), Some(" moves one square "));
        assert_eq!(sr.rendered_runs[2].text.as_deref(), Some("North"));
        assert_eq!(sr.rendered_runs[3].text.as_deref(), Some("."));
    }

    #[test]
    fn renders_different_direction() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportHitAndRun::new(Some("p1".into()), Some(Direction::Southeast));
        HitAndRunMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[2].text.as_deref(), Some("Southeast"));
    }

    #[test]
    fn skips_player_run_when_player_not_found() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportHitAndRun::new(Some("missing".into()), Some(Direction::West));
        HitAndRunMessage.render(&mut sr, &game, &report);
        // print_player emits nothing for an unresolved player.
        assert_eq!(sr.rendered_runs[0].text.as_deref(), Some(" moves one square "));
        assert_eq!(sr.rendered_runs[1].text.as_deref(), Some("West"));
    }
}
