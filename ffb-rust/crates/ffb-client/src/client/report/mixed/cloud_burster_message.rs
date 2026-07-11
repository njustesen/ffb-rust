use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_cloud_burster::ReportCloudBurster;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `CloudBursterMessage.java`. Java hardcodes indents `1`/`2` (not
/// `getIndent()`) — preserved literally here.
pub struct CloudBursterMessage;

impl ReportMessage for CloudBursterMessage {
    type Report = ReportCloudBurster;

    fn report_id(&self) -> ReportId {
        ReportId::CLOUD_BURSTER
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let throwing_player = report.get_thrower_id().and_then(|id| game.player(id));
        let intercepting_player = report.get_interceptor_id().and_then(|id| game.player(id));
        let thrower = throwing_player.map(|p| p.name.as_str()).unwrap_or("");
        let interceptor = intercepting_player.map(|p| p.name.as_str()).unwrap_or("");
        let genitiv = intercepting_player.map(|p| p.gender.genitive()).unwrap_or("");
        let home_is_throwing = report.get_thrower_team_id().is_some_and(|id| game.team_home.id == id);
        let thrower_style = if home_is_throwing { TextStyle::HOME_BOLD } else { TextStyle::AWAY_BOLD };
        let interceptor_style = if home_is_throwing { TextStyle::AWAY_BOLD } else { TextStyle::HOME_BOLD };

        status_report.print_indent_style(1, thrower_style, thrower);
        status_report.println_indent_style(1, TextStyle::BOLD, " uses CloudBurster");
        status_report.print_indent_style(2, interceptor_style, interceptor);
        status_report.println_indent_style(2, TextStyle::NONE, &format!(" has to reroll {} successful interception.", genitiv));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

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
        let thrower = Player { id: "t1".into(), name: "Thrower".into(), ..Player::default() };
        let interceptor = Player { id: "i1".into(), name: "Interceptor".into(), gender: PlayerGender::Female, ..Player::default() };
        Game::new(make_team("home", vec![thrower]), make_team("away", vec![interceptor]), Rules::Bb2020)
    }

    #[test]
    fn home_throwing_uses_home_bold_for_thrower() {
        let game = make_game();
        let report = ReportCloudBurster::new(Some("t1".into()), Some("i1".into()), Some("home".into()));
        let mut sr = StatusReport::new();
        CloudBursterMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[0].text.as_deref(), Some("Thrower"));
        assert_eq!(sr.rendered_runs[0].text_style, Some(TextStyle::HOME_BOLD));
        assert_eq!(sr.rendered_runs[1].text.as_deref(), Some(" uses CloudBurster"));
        assert_eq!(sr.rendered_runs[3].text.as_deref(), Some("Interceptor"));
        assert_eq!(sr.rendered_runs[3].text_style, Some(TextStyle::AWAY_BOLD));
        assert_eq!(sr.rendered_runs[4].text.as_deref(), Some(" has to reroll her successful interception."));
    }

    #[test]
    fn away_throwing_uses_away_bold_for_thrower() {
        let game = make_game();
        let report = ReportCloudBurster::new(Some("t1".into()), Some("i1".into()), Some("away".into()));
        let mut sr = StatusReport::new();
        CloudBursterMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[0].text_style, Some(TextStyle::AWAY_BOLD));
        assert_eq!(sr.rendered_runs[3].text_style, Some(TextStyle::HOME_BOLD));
    }

    #[test]
    fn report_id_and_key() {
        assert_eq!(CloudBursterMessage.report_id(), ReportId::CLOUD_BURSTER);
        assert_eq!(CloudBursterMessage.get_key(), "cloudBurster");
    }
}
