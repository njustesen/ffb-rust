use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::report_hand_over::ReportHandOver;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `HandOverMessage.java`.
pub struct HandOverMessage;

impl ReportMessage for HandOverMessage {
    type Report = ReportHandOver;

    fn report_id(&self) -> ReportId {
        ReportId::HAND_OVER
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let thrower = game.acting_player.player_id.as_deref().and_then(|id| game.player(id));
        let catcher = game.player(report.get_catcher_id());
        print_player(status_report, game, status_report.get_indent(), true, thrower);
        status_report.print_indent_style(status_report.get_indent(), TextStyle::BOLD, " hands over the ball to ");
        print_player(status_report, game, status_report.get_indent(), true, catcher);
        status_report.println_indent_style(status_report.get_indent(), TextStyle::BOLD, ":");
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
        assert_eq!(HandOverMessage.report_id(), ReportId::HAND_OVER);
    }

    #[test]
    fn renders_thrower_hands_over_to_catcher() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        add_player(&mut game, true, "thrower");
        add_player(&mut game, false, "catcher");
        game.acting_player.player_id = Some("thrower".into());
        let report = ReportHandOver::new("catcher".into());
        HandOverMessage.render(&mut status_report, &game, &report);

        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert_eq!(texts, vec!["thrower", " hands over the ball to ", "catcher", ":"]);
    }

    #[test]
    fn uses_bold_text_style_for_verb_and_colon() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        add_player(&mut game, true, "thrower");
        add_player(&mut game, false, "catcher");
        game.acting_player.player_id = Some("thrower".into());
        let report = ReportHandOver::new("catcher".into());
        HandOverMessage.render(&mut status_report, &game, &report);

        assert_eq!(status_report.rendered_runs[1].text_style, Some(TextStyle::BOLD));
        assert_eq!(status_report.rendered_runs[3].text_style, Some(TextStyle::BOLD));
    }

    #[test]
    fn thrower_and_catcher_use_home_away_bold_styles() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        add_player(&mut game, true, "thrower");
        add_player(&mut game, false, "catcher");
        game.acting_player.player_id = Some("thrower".into());
        let report = ReportHandOver::new("catcher".into());
        HandOverMessage.render(&mut status_report, &game, &report);

        assert_eq!(status_report.rendered_runs[0].text_style, Some(TextStyle::HOME_BOLD));
        assert_eq!(status_report.rendered_runs[2].text_style, Some(TextStyle::AWAY_BOLD));
    }
}
