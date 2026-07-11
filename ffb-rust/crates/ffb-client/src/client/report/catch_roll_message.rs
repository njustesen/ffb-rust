use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_mechanics::agility_mechanic::AgilityMechanic as AgilityMechanicTrait;
use ffb_mechanics::modifiers::RollModifier;
use ffb_model::enums::Rules;
use ffb_model::model::game::Game;
use ffb_model::report::report_catch_roll::ReportCatchRoll;
use ffb_model::report::report_id::ReportId;

/// Java: `game.getRules().getFactory(Factory.MECHANIC).forName(Mechanic.Type.AGILITY.name())`.
/// Mirrors the edition dispatch idiom used in `ffb-engine/src/mechanic/mod.rs`.
fn agility_mechanic_for(rules: Rules) -> Box<dyn AgilityMechanicTrait> {
    match rules {
        Rules::Bb2016 => Box::new(ffb_mechanics::bb2016::agility_mechanic::AgilityMechanic::new()),
        Rules::Bb2020 => Box::new(ffb_mechanics::bb2020::agility_mechanic::AgilityMechanic::new()),
        Rules::Bb2025 | Rules::Common => Box::new(ffb_mechanics::bb2025::agility_mechanic::AgilityMechanic::new()),
    }
}

/// 1:1 translation of `CatchRollMessage.java`.
pub struct CatchRollMessage;

impl ReportMessage for CatchRollMessage {
    type Report = ReportCatchRoll;

    fn report_id(&self) -> ReportId {
        ReportId::CATCH_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let player = report.get_player_id().and_then(|id| game.player(id));
        if !report.is_re_rolled() {
            print_player(status_report, game, status_report.get_indent(), true, player);
            let text = if report.is_bomb() { " tries to catch the bomb:" } else { " tries to catch the ball:" };
            status_report.println_indent_style(status_report.get_indent(), TextStyle::BOLD, text);
        }
        let status = format!("Catch Roll [ {} ]", report.get_roll());
        status_report.println_indent_style(status_report.get_indent() + 1, TextStyle::ROLL, &status);
        print_player(status_report, game, status_report.get_indent() + 2, false, player);
        let mut needed_roll: Option<String> = None;
        if report.is_successful() {
            let text = if report.is_bomb() { " catches the bomb." } else { " catches the ball." };
            status_report.println_indent(status_report.get_indent() + 2, text);
            if !report.is_re_rolled() {
                needed_roll = Some(format!("Succeeded on a roll of {}+", report.get_minimum_roll()));
            }
        } else {
            status_report.println_indent(status_report.get_indent() + 2, " fails the catch.");
            if !report.is_re_rolled() {
                needed_roll = Some(format!("Roll a {}+ to succeed", report.get_minimum_roll()));
            }
        }
        if let Some(mut needed_roll) = needed_roll {
            if let Some(player) = player {
                // java: mechanic.formatCatchResult(report, player) -- ReportSkillRoll only
                // retains modifier *names* (roll_modifier_names: Vec<String>), not the signed
                // magnitude/report-string data RollModifier objects carry in Java, so the
                // reconstructed modifiers below use modifier=0 for each name.
                let modifiers: Vec<RollModifier> =
                    report.get_roll_modifiers().iter().map(|name| RollModifier::new(name.clone(), 0)).collect();
                let mechanic = agility_mechanic_for(game.rules);
                needed_roll.push_str(&mechanic.format_catch_result(&modifiers, player));
            }
            status_report.println_indent_style(status_report.get_indent() + 2, TextStyle::NEEDED_ROLL, &needed_roll);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::PlayerGender;
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team { id: id.to_string(), name: id.to_string(), race: "human".into(), roster_id: "human".into(),
            coach: "coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: Vec::new(), players: Vec::new(), vampire_lord: false, necromancer: false }
    }

    fn make_game_with_player() -> Game {
        let mut game = Game::new(make_team("home"), make_team("away"), Rules::Bb2025);
        let mut player = Player::default();
        player.id = "p1".into();
        player.name = "Catcher".into();
        player.gender = PlayerGender::Male;
        player.agility = 3;
        game.team_home.players.push(player);
        game
    }

    #[test]
    fn successful_catch_prints_intro_and_success() {
        let mut status_report = StatusReport::new();
        let game = make_game_with_player();
        let report = ReportCatchRoll::new(Some("p1".into()), true, 4, 3, false, vec![], false);
        CatchRollMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Catcher"));
        assert_eq!(status_report.rendered_runs[1].text.as_deref(), Some(" tries to catch the ball:"));
        let roll_text = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some("Catch Roll [ 4 ]"));
        assert!(roll_text.is_some());
        let catch_text = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some(" catches the ball."));
        assert!(catch_text.is_some());
    }

    #[test]
    fn failed_catch_prints_fail_message_and_needed_roll() {
        let mut status_report = StatusReport::new();
        let game = make_game_with_player();
        let report = ReportCatchRoll::new(Some("p1".into()), false, 1, 3, false, vec![], false);
        CatchRollMessage.render(&mut status_report, &game, &report);
        let fail_text = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some(" fails the catch."));
        assert!(fail_text.is_some());
        let needed = status_report.rendered_runs.iter().find(|r| r.text_style == Some(TextStyle::NEEDED_ROLL));
        assert!(needed.is_some());
        assert!(needed.unwrap().text.as_deref().unwrap().starts_with("Roll a 3+ to succeed"));
    }

    #[test]
    fn bomb_catch_uses_bomb_wording() {
        let mut status_report = StatusReport::new();
        let game = make_game_with_player();
        let report = ReportCatchRoll::new(Some("p1".into()), true, 5, 2, false, vec![], true);
        CatchRollMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[1].text.as_deref(), Some(" tries to catch the bomb:"));
        let caught = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some(" catches the bomb."));
        assert!(caught.is_some());
    }

    #[test]
    fn re_rolled_catch_skips_intro_and_needed_roll() {
        let mut status_report = StatusReport::new();
        let game = make_game_with_player();
        let report = ReportCatchRoll::new(Some("p1".into()), true, 4, 3, true, vec![], false);
        CatchRollMessage.render(&mut status_report, &game, &report);
        // no " tries to catch" line and no NEEDED_ROLL style
        assert!(status_report.rendered_runs.iter().all(|r| r.text.as_deref() != Some(" tries to catch the ball:")));
        assert!(status_report.rendered_runs.iter().all(|r| r.text_style != Some(TextStyle::NEEDED_ROLL)));
    }

    #[test]
    fn report_id_is_catch_roll() {
        assert_eq!(CatchRollMessage.report_id(), ReportId::CATCH_ROLL);
    }
}
