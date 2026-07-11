use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_then_i_started_blastin::ReportThenIStartedBlastin;
use ffb_model::report::report_id::ReportId;

// java: PlayerGender.getSelf() — not yet ported to the shared enum, so translated locally here.
fn player_gender_self(gender: ffb_model::enums::PlayerGender) -> &'static str {
    use ffb_model::enums::PlayerGender::*;
    match gender {
        Male => "himself",
        Female => "herself",
        Nonbinary => "themself",
        Neutral => "itself",
    }
}

/// 1:1 translation of `ThenIStartedBlastinMessage.java`.
pub struct ThenIStartedBlastinMessage;

impl ReportMessage for ThenIStartedBlastinMessage {
    type Report = ReportThenIStartedBlastin;

    fn report_id(&self) -> ReportId {
        ReportId::THEN_I_STARTED_BLASTIN
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        if report.get_roll() > 0 {
            status_report.println_indent_style(indent, TextStyle::ROLL, &format!("\"Blastin' Solves Everything\" Roll [ {} ]", report.get_roll()));
        }
        let thrower = report.get_player_id().and_then(|id| game.player(id));
        print_player(status_report, game, indent, false, thrower);
        // java: StringTool.isProvided(report.getTargetPlayerId())
        if report.get_target_player_id().is_some_and(|s| !s.is_empty()) {
            status_report.print_indent_style(indent, TextStyle::NONE, " hits ");
            if report.is_fumble() {
                let self_word = thrower.map(|p| player_gender_self(p.gender)).unwrap_or("");
                status_report.print_indent_style(indent, TextStyle::NONE, self_word);
            } else if report.is_success() {
                let target = report.get_target_player_id().and_then(|id| game.player(id));
                print_player(status_report, game, indent, false, target);
            } else {
                status_report.print_indent_style(indent, TextStyle::NONE, "a player chosen by the opposing coach");
            }
        } else {
            status_report.print_indent_style(indent, TextStyle::NONE, " starts blastin' ");
        }
        status_report.println_indent_style(indent, TextStyle::NONE, ".");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, PlayerType, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_player(id: &str, name: &str, gender: PlayerGender) -> Player {
        Player { id: id.into(), name: name.into(), player_type: PlayerType::default(), gender, ..Default::default() }
    }

    fn make_team(id: &str, players: Vec<Player>) -> Team {
        Team {
            id: id.into(), name: format!("Team {id}"), race: "Human".into(), roster_id: "human".into(),
            coach: "Coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players, vampire_lord: false, necromancer: false,
        }
    }

    fn make_game() -> Game {
        let home = make_team("home", vec![make_player("thrower", "Thrower", PlayerGender::Male)]);
        let away = make_team("away", vec![make_player("target", "Target", PlayerGender::Female)]);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn report_id_is_then_i_started_blastin() {
        assert_eq!(ThenIStartedBlastinMessage.report_id(), ReportId::THEN_I_STARTED_BLASTIN);
    }

    #[test]
    fn no_target_starts_blastin() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportThenIStartedBlastin::new(Some("thrower".into()), None, 4, false, false);
        ThenIStartedBlastinMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == " starts blastin' "));
        assert!(texts.iter().any(|t| t == "\"Blastin' Solves Everything\" Roll [ 4 ]"));
    }

    #[test]
    fn fumble_hits_self() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportThenIStartedBlastin::new(Some("thrower".into()), Some("target".into()), 1, false, true);
        ThenIStartedBlastinMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == "himself"));
    }

    #[test]
    fn success_hits_target_player() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportThenIStartedBlastin::new(Some("thrower".into()), Some("target".into()), 5, true, false);
        ThenIStartedBlastinMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == "Target"));
    }

    #[test]
    fn miss_hits_a_player_chosen_by_opposing_coach() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportThenIStartedBlastin::new(Some("thrower".into()), Some("target".into()), 2, false, false);
        ThenIStartedBlastinMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == "a player chosen by the opposing coach"));
    }

    #[test]
    fn roll_zero_skips_roll_line() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportThenIStartedBlastin::new(Some("thrower".into()), None, 0, false, false);
        ThenIStartedBlastinMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(!texts.iter().any(|t| t.contains("Blastin'")));
    }
}
