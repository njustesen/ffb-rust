use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_weeping_dagger_roll::ReportWeepingDaggerRoll;

pub struct WeepingDaggerRollMessage;

impl ReportMessage for WeepingDaggerRollMessage {
    type Report = ReportWeepingDaggerRoll;

    fn report_id(&self) -> ReportId {
        ReportId::WEEPING_DAGGER_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let player_id = report.get_player_id();
        let player = player_id.and_then(|id| game.player(id));
        let indent = status_report.get_indent();
        let status = format!("Weeping Dagger Roll [ {} ]", report.get_roll());
        status_report.println_indent_style(indent, TextStyle::ROLL, &status);
        print_player(status_report, game, indent + 1, false, player);
        if report.is_successful() {
            let status = format!(" poisons {} opponent.", player.map(|p| p.gender.genitive()).unwrap_or(""));
            status_report.println_indent(indent + 1, &status);
        } else {
            let status = format!(" fails to poison {} opponent.", player.map(|p| p.gender.genitive()).unwrap_or(""));
            status_report.println_indent(indent + 1, &status);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team {
            id: id.to_string(), name: id.to_string(), race: "human".into(), roster_id: "human".into(),
            coach: "coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: Vec::new(), players: Vec::new(), vampire_lord: false, necromancer: false,
        }
    }

    fn make_game() -> Game {
        Game::new(make_team("home"), make_team("away"), Rules::Bb2025)
    }

    fn add_player(game: &mut Game, id: &str, name: &str, gender: PlayerGender) {
        let mut player = Player::default();
        player.id = id.to_string();
        player.name = name.to_string();
        player.gender = gender;
        game.team_home.players.push(player);
    }

    #[test]
    fn report_id_matches() {
        assert_eq!(WeepingDaggerRollMessage.report_id(), ReportId::WEEPING_DAGGER_ROLL);
    }

    #[test]
    fn successful_roll_poisons_opponent() {
        let mut game = make_game();
        add_player(&mut game, "p1", "Snik", PlayerGender::Male);
        let report = ReportWeepingDaggerRoll::new(Some("p1".into()), true, 5, 2, false, vec![]);
        let mut status_report = StatusReport::new();
        WeepingDaggerRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<Option<String>> = status_report.rendered_runs.iter().map(|r| r.text.clone()).collect();
        assert!(texts.contains(&Some("Weeping Dagger Roll [ 5 ]".to_string())));
        assert!(texts.contains(&Some(" poisons his opponent.".to_string())));
    }

    #[test]
    fn failed_roll_fails_to_poison() {
        let mut game = make_game();
        add_player(&mut game, "p1", "Snik", PlayerGender::Female);
        let report = ReportWeepingDaggerRoll::new(Some("p1".into()), false, 1, 2, false, vec![]);
        let mut status_report = StatusReport::new();
        WeepingDaggerRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<Option<String>> = status_report.rendered_runs.iter().map(|r| r.text.clone()).collect();
        assert!(texts.contains(&Some(" fails to poison her opponent.".to_string())));
    }

    #[test]
    fn nonbinary_gender_uses_their() {
        let mut game = make_game();
        add_player(&mut game, "p1", "Snik", PlayerGender::Nonbinary);
        let report = ReportWeepingDaggerRoll::new(Some("p1".into()), true, 5, 2, false, vec![]);
        let mut status_report = StatusReport::new();
        WeepingDaggerRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<Option<String>> = status_report.rendered_runs.iter().map(|r| r.text.clone()).collect();
        assert!(texts.contains(&Some(" poisons their opponent.".to_string())));
    }
}
