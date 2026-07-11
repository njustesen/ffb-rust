use crate::client::paragraph_style::ParagraphStyle;
use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::enums::TurnMode;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_turn_end::ReportTurnEnd;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `TurnEndMessage.java`.
pub struct TurnEndMessage;

impl ReportMessage for TurnEndMessage {
    type Report = ReportTurnEnd;

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        status_report.set_indent(0);
        let indent = status_report.get_indent();

        let touchdown_player = report.get_player_id_touchdown().and_then(|id| game.player(id));
        if let Some(touchdown_player) = touchdown_player {
            print_player(status_report, game, indent, true, Some(touchdown_player));
            status_report.println_indent_style(indent + 1, TextStyle::BOLD, " scores a touchdown.");
        }

        // java: `KnockoutRecovery.getRoll()`/`getBloodweiserBabes()`/`getReRollReason()` are
        // not present on `ReportTurnEnd`'s local reduced `KnockoutRecovery` type here, so the
        // "Knockout Recovery Roll [ X ]" header and re-roll-reason sub-line are not
        // reproducible from the available data; only the recovering/unconscious outcome is
        // rendered, using the plain `recovered` field directly (no `isRecovering()` method).
        for knockout_recovery in report.get_knockout_recoveries() {
            let player = game.player(&knockout_recovery.player_id);
            print_player(status_report, game, indent + 1, false, player);
            if knockout_recovery.recovered {
                status_report.println_indent(indent + 1, " is regaining consciousness.");
            } else {
                status_report.println_indent(indent + 1, " stays unconscious.");
            }
        }

        // java: `HeatExhaustion.isExhausted()` has no equivalent field on this reduced local
        // `HeatExhaustion` type — every entry present in `get_heat_exhaustions()` is
        // implicitly exhausted here (matching `to_json_value()`'s hardcoded `"exhausted": true`).
        if !report.get_heat_exhaustions().is_empty() {
            status_report.println_indent_style(indent, TextStyle::ROLL, &format!("Heat Exhaustion Roll [ {} ] ", report.get_heat_roll()));
            for heat_exhaustion in report.get_heat_exhaustions() {
                let player = game.player(&heat_exhaustion.player_id);
                print_player(status_report, game, indent + 1, false, player);
                status_report.println_indent(indent + 1, " is suffering from heat exhaustion.");
            }
        }

        // java: `if (unzappedPlayers != null)` — `get_unzapped_players()` returns `&[String]`
        // (never null, just possibly empty), so a plain loop already covers both the null and
        // empty-list no-op cases.
        for player_id in report.get_unzapped_players() {
            let player = game.player(player_id);
            print_player(status_report, game, indent, true, player);
            status_report.println_indent(indent, " recovers from Zap! spell effect.");
        }

        if TurnMode::Regular == game.turn_mode {
            if game.home_playing {
                let status = format!("{} start turn {}.", game.team_home.name, game.turn_data_home.turn_nr);
                status_report.println_style(Some(ParagraphStyle::SPACE_ABOVE_BELOW), Some(TextStyle::TURN_HOME), &status);
            } else {
                let status = format!("{} start turn {}.", game.team_away.name, game.turn_data_away.turn_nr);
                status_report.println_style(Some(ParagraphStyle::SPACE_ABOVE_BELOW), Some(TextStyle::TURN_AWAY), &status);
            }
        }
    }

    fn report_id(&self) -> ReportId {
        ReportId::TURN_END
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;
    use ffb_model::report::mixed::report_turn_end::{HeatExhaustion, KnockoutRecovery};

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
        let home = make_team(
            "home",
            vec![make_player("scorer", "Scorer"), make_player("ko1", "Knocked"), make_player("he1", "Heated"), make_player("zap1", "Zapped")],
        );
        let away = make_team("away", vec![]);
        Game::new(home, away, Rules::Bb2020)
    }

    #[test]
    fn touchdown_and_regular_turn_start_home() {
        let mut game = make_game();
        game.home_playing = true;
        game.turn_mode = TurnMode::Regular;
        game.turn_data_home.turn_nr = 7;
        let report = ReportTurnEnd::new(Some("scorer".into()), vec![], vec![], vec![], 0);
        let mut status_report = StatusReport::new();
        TurnEndMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Scorer"));
        assert_eq!(status_report.rendered_runs[0].text_style, Some(TextStyle::HOME_BOLD));
        assert_eq!(status_report.rendered_runs[1].text.as_deref(), Some(" scores a touchdown."));
        let last = status_report.rendered_runs.last().unwrap();
        assert_eq!(last.paragraph_style, None); // terminator run from println_style
        let turn_run = &status_report.rendered_runs[status_report.rendered_runs.len() - 2];
        assert_eq!(turn_run.text.as_deref(), Some("Team home start turn 7."));
        assert_eq!(turn_run.text_style, Some(TextStyle::TURN_HOME));
        assert_eq!(turn_run.paragraph_style, Some(ParagraphStyle::SPACE_ABOVE_BELOW));
    }

    #[test]
    fn knockout_recovery_recovering_and_unconscious() {
        let game = make_game();
        let report = ReportTurnEnd::new(
            None,
            vec![KnockoutRecovery::new("ko1".into(), true)],
            vec![],
            vec![],
            0,
        );
        let mut status_report = StatusReport::new();
        TurnEndMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Knocked"));
        assert_eq!(status_report.rendered_runs[1].text.as_deref(), Some(" is regaining consciousness."));

        let mut status_report2 = StatusReport::new();
        let report2 = ReportTurnEnd::new(None, vec![KnockoutRecovery::new("ko1".into(), false)], vec![], vec![], 0);
        TurnEndMessage.render(&mut status_report2, &game, &report2);
        assert_eq!(status_report2.rendered_runs[1].text.as_deref(), Some(" stays unconscious."));
    }

    #[test]
    fn heat_exhaustion_and_unzapped_players() {
        let game = make_game();
        let report = ReportTurnEnd::new(
            None,
            vec![],
            vec![HeatExhaustion::new("he1".into(), 3)],
            vec!["zap1".into()],
            9,
        );
        let mut status_report = StatusReport::new();
        TurnEndMessage.render(&mut status_report, &game, &report);
        // run0 = header text, run1 = println terminator, run2 = player, run3 = outcome text,
        // run4 = terminator, run5 = zapped player, run6 = zap recovery text, run7 = terminator.
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Heat Exhaustion Roll [ 9 ] "));
        assert_eq!(status_report.rendered_runs[0].text_style, Some(TextStyle::ROLL));
        assert_eq!(status_report.rendered_runs[2].text.as_deref(), Some("Heated"));
        assert_eq!(status_report.rendered_runs[3].text.as_deref(), Some(" is suffering from heat exhaustion."));
        assert_eq!(status_report.rendered_runs[5].text.as_deref(), Some("Zapped"));
        assert_eq!(status_report.rendered_runs[5].text_style, Some(TextStyle::HOME_BOLD));
        assert_eq!(status_report.rendered_runs[6].text.as_deref(), Some(" recovers from Zap! spell effect."));
    }

    #[test]
    fn no_regular_turn_line_when_not_in_regular_mode() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Kickoff;
        let report = ReportTurnEnd::new(None, vec![], vec![], vec![], 0);
        let mut status_report = StatusReport::new();
        TurnEndMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.is_empty());
    }

    #[test]
    fn report_id_and_key() {
        assert_eq!(TurnEndMessage.report_id(), ReportId::TURN_END);
        assert_eq!(TurnEndMessage.get_key(), "turnEnd");
    }
}
