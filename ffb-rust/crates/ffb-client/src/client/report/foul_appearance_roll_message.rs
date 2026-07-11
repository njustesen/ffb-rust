use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::report_foul_appearance_roll::ReportFoulAppearanceRoll;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `FoulAppearanceRollMessage.java`.
pub struct FoulAppearanceRollMessage;

impl ReportMessage for FoulAppearanceRollMessage {
    type Report = ReportFoulAppearanceRoll;

    fn report_id(&self) -> ReportId {
        ReportId::FOUL_APPEARANCE_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let player = game.acting_player.player_id.as_deref().and_then(|id| game.player(id));
        let status = format!("Foul Appearance Roll [ {} ]", report.base.get_roll());
        status_report.println_indent_style(status_report.get_indent(), TextStyle::ROLL, &status);
        print_player(status_report, game, status_report.get_indent() + 1, false, player);
        if report.base.is_successful() {
            status_report.print_indent(status_report.get_indent() + 1, " resists the Foul Appearance");
        } else {
            status_report.print_indent(status_report.get_indent() + 1, " cannot overcome the Foul Appearance");
        }

        if report.get_defender_id().is_some_and(|id| !id.is_empty()) {
            let defender = report.get_defender_id().and_then(|id| game.player(id));
            if defender.is_some() {
                status_report.print_indent(status_report.get_indent() + 1, " of ");
                print_player(status_report, game, status_report.get_indent() + 1, false, defender);
            }
        }

        status_report.println_indent(status_report.get_indent() + 1, ".");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::game::Game;
    use ffb_model::model::player::Player;
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

    fn add_player(game: &mut Game, home: bool, id: &str) {
        let mut player = Player::default();
        player.id = id.to_string();
        player.name = id.to_string();
        if home {
            game.team_home.players.push(player);
        } else {
            game.team_away.players.push(player);
        }
    }

    #[test]
    fn report_id_matches() {
        assert_eq!(FoulAppearanceRollMessage.report_id(), ReportId::FOUL_APPEARANCE_ROLL);
    }

    #[test]
    fn successful_roll_without_defender() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        add_player(&mut game, true, "attacker");
        game.acting_player.player_id = Some("attacker".into());
        let report = ReportFoulAppearanceRoll::new(Some("attacker".into()), true, 4, 2, false, vec![], None);
        FoulAppearanceRollMessage.render(&mut status_report, &game, &report);

        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.iter().any(|t| t.contains("resists the Foul Appearance")));
        assert!(!texts.iter().any(|t| t.contains(" of ")));
    }

    #[test]
    fn unsuccessful_roll_with_defender() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        add_player(&mut game, true, "attacker");
        add_player(&mut game, false, "defender");
        game.acting_player.player_id = Some("attacker".into());
        let report = ReportFoulAppearanceRoll::new(Some("attacker".into()), false, 1, 3, false, vec![], Some("defender".into()));
        FoulAppearanceRollMessage.render(&mut status_report, &game, &report);

        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.iter().any(|t| t.contains("cannot overcome the Foul Appearance")));
        assert!(texts.iter().any(|t| t == &" of "));
        assert!(texts.iter().any(|t| t == &"defender"));
    }

    #[test]
    fn missing_defender_id_skips_of_clause() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        add_player(&mut game, true, "attacker");
        game.acting_player.player_id = Some("attacker".into());
        let report = ReportFoulAppearanceRoll::new(Some("attacker".into()), false, 1, 3, false, vec![], None);
        FoulAppearanceRollMessage.render(&mut status_report, &game, &report);

        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(!texts.iter().any(|t| t == &" of "));
    }

    #[test]
    fn ends_with_period() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        add_player(&mut game, true, "attacker");
        game.acting_player.player_id = Some("attacker".into());
        let report = ReportFoulAppearanceRoll::new(Some("attacker".into()), true, 4, 2, false, vec![], None);
        FoulAppearanceRollMessage.render(&mut status_report, &game, &report);

        let last_text = status_report.rendered_runs.iter().rev().find_map(|r| r.text.as_deref());
        assert_eq!(last_text, Some("."));
    }
}
