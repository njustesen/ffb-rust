use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::report::mixed::report_select_gaze_target::ReportSelectGazeTarget;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `SelectGazeTargetMessage.java`.
pub struct SelectGazeTargetMessage;

impl ReportMessage for SelectGazeTargetMessage {
    type Report = ReportSelectGazeTarget;

    fn report_id(&self) -> ReportId {
        ReportId::SELECT_GAZE_TARGET
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let attacker = report.get_attacker().and_then(|id| game.player(id));
        let defender = report.get_defender().and_then(|id| game.player(id));
        let indent = status_report.get_indent();

        // java: NPEs if attacker/defender are missing; guarded defensively here instead.
        if let (Some(attacker), Some(defender)) = (attacker, defender) {
            let attacker_style = team_style_for_player(game, attacker);
            status_report.print_indent_style(indent, attacker_style, &attacker.name.clone());
            status_report.print_indent_style(indent, TextStyle::NONE, " targets ");
            let defender_style = team_style_for_player(game, defender);
            status_report.print_indent_style(indent, defender_style, &defender.name.clone());
            status_report.println_indent_style(indent, TextStyle::NONE, ".");
        }
    }
}

fn team_style_for_player(game: &Game, player: &Player) -> TextStyle {
    if game.team_home.has_player(&player.id) {
        TextStyle::HOME
    } else {
        TextStyle::AWAY
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;

    fn make_team(id: &str, players: Vec<Player>) -> Team {
        Team {
            id: id.into(),
            name: format!("Team {id}"),
            race: "Human".into(),
            roster_id: "human".into(),
            coach: format!("Coach {id}"),
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

    fn make_player(id: &str, name: &str) -> Player {
        Player {
            id: id.into(),
            name: name.into(),
            ..Default::default()
        }
    }

    fn make_game() -> Game {
        let home = make_team("home", vec![make_player("p1", "Attacker")]);
        let away = make_team("away", vec![make_player("p2", "Defender")]);
        Game::new(home, away, Rules::Bb2020)
    }

    #[test]
    fn attacker_home_defender_away() {
        let game = make_game();
        let report = ReportSelectGazeTarget::new(Some("p1".into()), Some("p2".into()));
        let mut status_report = StatusReport::new();
        SelectGazeTargetMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Attacker"));
        assert_eq!(status_report.rendered_runs[0].text_style, Some(TextStyle::HOME));
        assert_eq!(status_report.rendered_runs[1].text.as_deref(), Some(" targets "));
        assert_eq!(status_report.rendered_runs[2].text.as_deref(), Some("Defender"));
        assert_eq!(status_report.rendered_runs[2].text_style, Some(TextStyle::AWAY));
        assert_eq!(status_report.rendered_runs[3].text.as_deref(), Some("."));
    }

    #[test]
    fn missing_attacker_renders_nothing() {
        let game = make_game();
        let report = ReportSelectGazeTarget::new(None, Some("p2".into()));
        let mut status_report = StatusReport::new();
        SelectGazeTargetMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.is_empty());
    }

    #[test]
    fn report_id_and_key() {
        assert_eq!(SelectGazeTargetMessage.report_id(), ReportId::SELECT_GAZE_TARGET);
        assert_eq!(SelectGazeTargetMessage.get_key(), "selectGazeTarget");
    }
}
