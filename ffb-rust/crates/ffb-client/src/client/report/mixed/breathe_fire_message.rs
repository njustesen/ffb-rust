use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::enums::PlayerGender;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_breathe_fire::ReportBreatheFire;
use ffb_model::report::report_id::ReportId;

/// Java: `PlayerGender.getSelf()` not yet in ffb-model enum (himself/herself/themself/itself).
fn gender_self(gender: PlayerGender) -> &'static str {
    match gender {
        PlayerGender::Male => "himself",
        PlayerGender::Female => "herself",
        PlayerGender::Nonbinary => "themself",
        PlayerGender::Neutral => "itself",
    }
}

/// 1:1 translation of `BreatheFireMessage.java`.
pub struct BreatheFireMessage;

impl BreatheFireMessage {
    /// Java: `private void printNeededRoll(boolean, BreatheFireResult)`.
    fn print_needed_roll(&self, status_report: &mut StatusReport, indent: i32, modified: bool, potential_result: &str) {
        let mut needed_roll = String::from(" (Roll");

        if modified {
            needed_roll.push_str(" - 1 opponent has strength 5 or more");
        }

        needed_roll.push_str(" >= ");

        match potential_result {
            "KNOCK_DOWN" => needed_roll.push_str("6 to knock down opponent"),
            "PRONE" => needed_roll.push_str("4 to place opponent prone"),
            "NO_EFFECT" => needed_roll.push_str("2 to avoid knock down"),
            _ => {}
        }

        needed_roll.push_str(").");

        status_report.println_indent_style(indent, TextStyle::NEEDED_ROLL, &needed_roll);
    }
}

impl ReportMessage for BreatheFireMessage {
    type Report = ReportBreatheFire;

    fn report_id(&self) -> ReportId {
        ReportId::BREATHE_FIRE
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let player = game.acting_player.player_id.as_deref().and_then(|id| game.player(id));

        status_report.println_indent_style(indent, TextStyle::ROLL, &format!("Breathe Fire Roll [ {} ]", report.get_roll()));
        print_player(status_report, game, indent + 1, false, player);

        // java: BreatheFireResult enum name matching — verified against ffb-java
        // BreatheFireResult.java constants: KNOCK_DOWN, PRONE, NO_EFFECT, FAILURE.
        match report.get_result() {
            "KNOCK_DOWN" => {
                status_report.print_indent_style(indent + 1, TextStyle::NONE, " engulfs ");
                let defender = report.get_defender_id().and_then(|id| game.player(id));
                print_player(status_report, game, indent + 1, false, defender);
                status_report.println_indent_style(indent + 1, TextStyle::NONE, " in flames.");
            }
            "PRONE" => {
                status_report.print_indent_style(indent + 1, TextStyle::NONE, " forces ");
                let defender = report.get_defender_id().and_then(|id| game.player(id));
                print_player(status_report, game, indent + 1, false, defender);
                status_report.println_indent_style(indent + 1, TextStyle::NONE, " to take cover.");
                self.print_needed_roll(status_report, indent + 1, report.is_strong_opponent(), "KNOCK_DOWN");
            }
            "NO_EFFECT" => {
                status_report.print_indent_style(indent + 1, TextStyle::NONE, " misses ");
                let defender = report.get_defender_id().and_then(|id| game.player(id));
                print_player(status_report, game, indent + 1, false, defender);
                status_report.println_indent_style(indent + 1, TextStyle::NONE, ".");
                self.print_needed_roll(status_report, indent + 1, report.is_strong_opponent(), "KNOCK_DOWN");
                self.print_needed_roll(status_report, indent + 1, report.is_strong_opponent(), "PRONE");
            }
            "FAILURE" => {
                let gender_word = player.map(|p| gender_self(p.gender)).unwrap_or("");
                status_report.println_indent_style(indent + 1, TextStyle::NONE, &format!(" engulfs {} in flames.", gender_word));
                self.print_needed_roll(status_report, indent + 1, report.is_strong_opponent(), "KNOCK_DOWN");
                self.print_needed_roll(status_report, indent + 1, report.is_strong_opponent(), "PRONE");
                self.print_needed_roll(status_report, indent + 1, report.is_strong_opponent(), "NO_EFFECT");
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::acting_player::ActingPlayer;
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
        let attacker = Player { id: "p1".into(), name: "Dragon".into(), gender: PlayerGender::Male, ..Player::default() };
        let defender = Player { id: "p2".into(), name: "Victim".into(), ..Player::default() };
        let mut game = Game::new(make_team("home", vec![attacker]), make_team("away", vec![defender]), Rules::Bb2020);
        game.acting_player = ActingPlayer { player_id: Some("p1".into()), ..Default::default() };
        game
    }

    #[test]
    fn knock_down_engulfs_defender() {
        let game = make_game();
        let report = ReportBreatheFire::new(None, true, 6, 2, false, Some("p2".into()), false, "KNOCK_DOWN".into());
        let mut sr = StatusReport::new();
        BreatheFireMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[0].text.as_deref(), Some("Breathe Fire Roll [ 6 ]"));
        assert_eq!(sr.rendered_runs[3].text.as_deref(), Some(" engulfs "));
        assert_eq!(sr.rendered_runs[4].text.as_deref(), Some("Victim"));
        assert_eq!(sr.rendered_runs[5].text.as_deref(), Some(" in flames."));
    }

    #[test]
    fn prone_forces_cover_and_prints_needed_roll() {
        let game = make_game();
        let report = ReportBreatheFire::new(None, true, 4, 2, false, Some("p2".into()), true, "PRONE".into());
        let mut sr = StatusReport::new();
        BreatheFireMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[5].text.as_deref(), Some(" to take cover."));
        assert_eq!(
            sr.rendered_runs[7].text.as_deref(),
            Some(" (Roll - 1 opponent has strength 5 or more >= 6 to knock down opponent).")
        );
    }

    #[test]
    fn no_effect_prints_two_needed_rolls() {
        let game = make_game();
        let report = ReportBreatheFire::new(None, false, 3, 2, false, Some("p2".into()), false, "NO_EFFECT".into());
        let mut sr = StatusReport::new();
        BreatheFireMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[5].text.as_deref(), Some("."));
        assert_eq!(sr.rendered_runs[7].text.as_deref(), Some(" (Roll >= 6 to knock down opponent)."));
        assert_eq!(sr.rendered_runs[9].text.as_deref(), Some(" (Roll >= 4 to place opponent prone)."));
    }

    #[test]
    fn failure_engulfs_self_and_prints_three_needed_rolls() {
        let game = make_game();
        let report = ReportBreatheFire::new(None, false, 1, 2, false, Some("p2".into()), false, "FAILURE".into());
        let mut sr = StatusReport::new();
        BreatheFireMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[3].text.as_deref(), Some(" engulfs himself in flames."));
        assert_eq!(sr.rendered_runs[5].text.as_deref(), Some(" (Roll >= 6 to knock down opponent)."));
        assert_eq!(sr.rendered_runs[7].text.as_deref(), Some(" (Roll >= 4 to place opponent prone)."));
        assert_eq!(sr.rendered_runs[9].text.as_deref(), Some(" (Roll >= 2 to avoid knock down)."));
    }

    #[test]
    fn report_id_and_key() {
        assert_eq!(BreatheFireMessage.report_id(), ReportId::BREATHE_FIRE);
        assert_eq!(BreatheFireMessage.get_key(), "breatheFire");
    }
}
