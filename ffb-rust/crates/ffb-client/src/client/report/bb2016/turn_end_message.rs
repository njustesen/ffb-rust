use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::paragraph_style::ParagraphStyle;
use crate::client::text_style::TextStyle;
use ffb_model::enums::TurnMode;
use ffb_model::model::game::Game;
use ffb_model::report::bb2016::report_turn_end::ReportTurnEnd;
use ffb_model::report::report_id::ReportId;

pub struct TurnEndMessage;

impl ReportMessage for TurnEndMessage {
    type Report = ReportTurnEnd;

    fn report_id(&self) -> ReportId {
        ReportId::TURN_END
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        status_report.set_indent(0);
        let indent = status_report.get_indent();
        let touchdown_player = report.get_player_id_touchdown().and_then(|id| game.player(id));
        if let Some(touchdown_player) = touchdown_player {
            print_player(status_report, game, indent, true, Some(touchdown_player));
            status_report.println_indent_style(indent + 1, TextStyle::BOLD, " scores a touchdown.");
        }
        for knockout_recovery in report.get_knockout_recoveries() {
            let mut status = format!("Knockout Recovery Roll [ {} ] ", knockout_recovery.get_roll());
            if knockout_recovery.get_bloodweiser_babes() > 0 {
                status.push_str(&format!(" + {} Bloodweiser Kegs", knockout_recovery.get_bloodweiser_babes()));
            }
            status_report.println_indent_style(indent, TextStyle::ROLL, &status);
            let player = knockout_recovery.get_player_id().and_then(|id| game.player(id));
            print_player(status_report, game, indent + 1, false, player);
            if knockout_recovery.is_recovering() {
                status_report.println_indent(indent + 1, " is regaining consciousness.");
            } else {
                status_report.println_indent(indent + 1, " stays unconscious.");
            }
        }
        for heat_exhaustion in report.get_heat_exhaustions() {
            let status = format!("Heat Exhaustion Roll [ {} ] ", heat_exhaustion.get_roll());
            status_report.println_indent_style(indent, TextStyle::ROLL, &status);
            let player = heat_exhaustion.get_player_id().and_then(|id| game.player(id));
            print_player(status_report, game, indent + 1, false, player);
            if heat_exhaustion.is_exhausted() {
                status_report.println_indent(indent + 1, " is suffering from heat exhaustion.");
            } else {
                status_report.println_indent(indent + 1, " is unaffected.");
            }
        }
        for player_id in report.get_unzapped_player_ids() {
            let player = game.player(player_id);
            print_player(status_report, game, indent, true, player);
            status_report.println_indent(indent, " recovers from Zap! spell effect.");
        }
        if game.turn_mode == TurnMode::Regular {
            if game.home_playing {
                let status = format!("{} start turn {}.", game.team_home.name, game.turn_data_home.turn_nr);
                status_report.println_style(Some(ParagraphStyle::SPACE_ABOVE_BELOW), Some(TextStyle::TURN_HOME), &status);
            } else {
                let status = format!("{} start turn {}.", game.team_away.name, game.turn_data_away.turn_nr);
                status_report.println_style(Some(ParagraphStyle::SPACE_ABOVE_BELOW), Some(TextStyle::TURN_AWAY), &status);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, PlayerType, Rules};
    use ffb_model::model::heat_exhaustion::HeatExhaustion;
    use ffb_model::model::knockout_recovery::KnockoutRecovery;
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
    fn get_key_is_turn_end() {
        assert_eq!(TurnEndMessage.get_key(), "turnEnd");
    }

    #[test]
    fn touchdown_player_reports_score() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportTurnEnd::new(Some("p1".into()), vec![], vec![], vec![]);
        TurnEndMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" scores a touchdown.")));
    }

    #[test]
    fn regular_turn_mode_reports_turn_start() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.turn_data_home.turn_nr = 3;
        let report = ReportTurnEnd::new(None, vec![], vec![], vec![]);
        TurnEndMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some("Team home start turn 3.")));
    }

    #[test]
    fn knockout_recovery_reports_regaining_consciousness() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportTurnEnd::new(None, vec![KnockoutRecovery::new("p1", true, 5, 0, None)], vec![], vec![]);
        TurnEndMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" is regaining consciousness.")));
    }

    #[test]
    fn heat_exhaustion_reports_exhausted() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportTurnEnd::new(None, vec![], vec![HeatExhaustion::new("p1", true, 2)], vec![]);
        TurnEndMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" is suffering from heat exhaustion.")));
    }
}
