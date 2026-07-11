use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use ffb_model::model::game::Game;
use ffb_model::model::pushback_mode::PushbackMode;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_pushback::ReportPushback;

/// 1:1 translation of `PushbackMessage.java`.
pub struct PushbackMessage;

impl ReportMessage for PushbackMessage {
    type Report = ReportPushback;

    fn report_id(&self) -> ReportId {
        ReportId::PUSHBACK
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent() + 1;
        let defender = game.player(report.get_defender_id());

        if report.get_pushback_mode() == PushbackMode::SIDE_STEP {
            print_player(status_report, game, indent, false, defender);
            status_report.println_indent(indent, " uses Sidestep to avoid being pushed.");
        }
        if report.get_pushback_mode() == PushbackMode::GRAB {
            let acting_player = game.acting_player.player_id.as_deref().and_then(|id| game.player(id));
            print_player(status_report, game, indent, false, acting_player);
            let genitive = acting_player.map(|p| p.gender.genitive()).unwrap_or("");
            status_report.println_indent(indent, &format!(" uses Grab to place {genitive} opponent."));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::text_style::TextStyle;
    use ffb_model::enums::{PlayerGender, PlayerType, Rules};
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
        let home = make_team("home", vec![make_player("a1", "Attacker", PlayerGender::Male)]);
        let away = make_team("away", vec![make_player("d1", "Defender", PlayerGender::Female)]);
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.acting_player.player_id = Some("a1".into());
        game
    }

    #[test]
    fn side_step_prints_defender_avoiding_push() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPushback::new("d1".into(), PushbackMode::SIDE_STEP);
        PushbackMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.contains(&"Defender".to_string()));
        assert!(texts.contains(&" uses Sidestep to avoid being pushed.".to_string()));
    }

    #[test]
    fn grab_prints_acting_player_placing_opponent() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPushback::new("d1".into(), PushbackMode::GRAB);
        PushbackMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.contains(&"Attacker".to_string()));
        assert!(texts.iter().any(|t| t.contains("uses Grab to place his opponent.")));
    }

    #[test]
    fn regular_mode_prints_nothing() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPushback::new("d1".into(), PushbackMode::REGULAR);
        PushbackMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.is_empty());
    }

    #[test]
    fn side_step_uses_indent_plus_one() {
        let mut status_report = StatusReport::new();
        status_report.set_indent(2);
        let game = make_game();
        let report = ReportPushback::new("d1".into(), PushbackMode::SIDE_STEP);
        PushbackMessage.render(&mut status_report, &game, &report);
        let run = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some("Defender")).unwrap();
        assert_eq!(run.text_style, Some(TextStyle::AWAY));
    }

    #[test]
    fn report_id_is_pushback() {
        assert_eq!(PushbackMessage.report_id(), ReportId::PUSHBACK);
    }
}
