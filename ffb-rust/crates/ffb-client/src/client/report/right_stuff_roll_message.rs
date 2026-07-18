use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_mechanics::agility_mechanic::AgilityMechanic as AgilityMechanicTrait;
use ffb_mechanics::modifiers::RollModifier;
use ffb_model::enums::Rules;
use ffb_model::model::game::Game;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_right_stuff_roll::ReportRightStuffRoll;

/// Java: `game.getRules().getFactory(Factory.MECHANIC).forName(Mechanic.Type.AGILITY.name())`.
/// Mirrors the edition dispatch idiom used in `dodge_roll_message.rs` / `ffb-engine/src/mechanic/mod.rs`.
fn agility_mechanic_for(rules: Rules) -> Box<dyn AgilityMechanicTrait> {
    match rules {
        Rules::Bb2016 => Box::new(ffb_mechanics::bb2016::agility_mechanic::AgilityMechanic::new()),
        Rules::Bb2020 => Box::new(ffb_mechanics::bb2020::agility_mechanic::AgilityMechanic::new()),
        Rules::Bb2025 | Rules::Common => Box::new(ffb_mechanics::bb2025::agility_mechanic::AgilityMechanic::new()),
    }
}

/// 1:1 translation of `RightStuffRollMessage.java`.
pub struct RightStuffRollMessage;

impl ReportMessage for RightStuffRollMessage {
    type Report = ReportRightStuffRoll;

    fn report_id(&self) -> ReportId {
        ReportId::RIGHT_STUFF_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let mut needed_roll: Option<String> = None;
        let status = format!("Landing Roll [ {} ]", report.get_roll());
        status_report.println_indent_style(status_report.get_indent(), TextStyle::ROLL, &status);
        let thrown_player = report.get_player_id().and_then(|id| game.player(id));
        print_player(status_report, game, status_report.get_indent() + 1, false, thrown_player);
        if report.is_successful() {
            let status = format!(
                " lands on {} feet.",
                thrown_player.map(|p| p.gender.genitive()).unwrap_or("")
            );
            status_report.println_indent(status_report.get_indent() + 1, &status);
            if !report.is_re_rolled() {
                needed_roll = Some(format!("Succeeded on a roll of {}+", report.get_minimum_roll()));
            }
        } else {
            status_report.println_indent(status_report.get_indent() + 1, " crashes to the ground.");
            if !report.is_re_rolled() {
                needed_roll = Some(format!("Roll a {}+ to succeed", report.get_minimum_roll()));
            }
        }
        if let (Some(mut needed_roll), Some(thrown_player)) = (needed_roll, thrown_player) {
            // java: mechanic.formatRightStuffResult(report, thrownPlayer) -- ReportSkillRoll only
            // retains modifier *names* (roll_modifier_names: Vec<String>), not the signed
            // magnitude/report-string data RollModifier objects carry in Java, so the
            // reconstructed modifiers below use modifier=0 for each name (same approach as
            // dodge_roll_message.rs).
            let modifiers: Vec<RollModifier> = report
                .get_roll_modifiers()
                .iter()
                .map(|name| RollModifier::new(name.clone(), 0))
                .collect();
            let mechanic = agility_mechanic_for(game.rules);
            needed_roll.push_str(&mechanic.format_right_stuff_result(&modifiers, thrown_player));
            status_report.println_indent_style(status_report.get_indent() + 1, TextStyle::NEEDED_ROLL, &needed_roll);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::player::Player;
    use ffb_model::model::player_gender::PlayerGender;
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

    fn add_player(game: &mut Game, agility: i32) -> String {
        let mut player = Player::default();
        player.id = "p1".into();
        player.name = "Gutter Runner".into();
        player.gender = PlayerGender::Male;
        player.agility = agility;
        player.position_agility = agility;
        game.team_home.players.push(player);
        "p1".into()
    }

    #[test]
    fn report_id_matches() {
        assert_eq!(RightStuffRollMessage.report_id(), ReportId::RIGHT_STUFF_ROLL);
    }

    #[test]
    fn render_successful_lands_on_feet_with_needed_roll() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        add_player(&mut game, 3);
        let report = ReportRightStuffRoll::new(Some("p1".into()), true, 4, 2, false, vec![]);
        RightStuffRollMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Landing Roll [ 4 ]"));
        assert_eq!(status_report.rendered_runs[3].text.as_deref(), Some(" lands on his feet."));
        let last = status_report.rendered_runs.iter().rev().find(|r| r.text.is_some()).unwrap();
        // NB: the exact suffix produced by ffb_mechanics::bb2025::AgilityMechanic::format_result
        // is asserted loosely (starts_with) rather than as a full literal, matching the
        // established pattern in dodge_roll_message.rs -- that mechanic has its own pre-existing
        // formatting quirk (a doubled space before ">=") which is out of scope for this file.
        assert!(last.text.as_deref().unwrap().starts_with("Succeeded on a roll of 2+ (Roll"));
        assert!(last.text.as_deref().unwrap().contains(">= 3+)"));
        assert_eq!(last.text_style, Some(TextStyle::NEEDED_ROLL));
    }

    #[test]
    fn render_failed_crashes_with_needed_roll() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        add_player(&mut game, 3);
        let report = ReportRightStuffRoll::new(Some("p1".into()), false, 1, 2, false, vec![]);
        RightStuffRollMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[3].text.as_deref(), Some(" crashes to the ground."));
        let last = status_report.rendered_runs.iter().rev().find(|r| r.text.is_some()).unwrap();
        assert!(last.text.as_deref().unwrap().starts_with("Roll a 2+ to succeed (Roll"));
        assert!(last.text.as_deref().unwrap().contains(">= 3+)"));
    }

    #[test]
    fn render_re_rolled_omits_needed_roll() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        add_player(&mut game, 3);
        let report = ReportRightStuffRoll::new(Some("p1".into()), true, 4, 2, true, vec![]);
        RightStuffRollMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().all(|r| r.text_style != Some(TextStyle::NEEDED_ROLL)));
    }

    #[test]
    fn render_bb2016_uses_bb2016_needed_roll_format() {
        // Regression test: the renderer used to hardcode the bb2025 "(Roll >= X+)" format
        // regardless of edition. BB2016's AgilityMechanic formats this as "(AG X + Roll > 6)."
        // instead, so a Bb2016 game must dispatch to the bb2016 mechanic, not bb2025's.
        let mut status_report = StatusReport::new();
        let mut game = Game::new(make_team("home"), make_team("away"), Rules::Bb2016);
        add_player(&mut game, 3);
        let report = ReportRightStuffRoll::new(Some("p1".into()), true, 4, 2, false, vec![]);
        RightStuffRollMessage.render(&mut status_report, &game, &report);
        let last = status_report.rendered_runs.iter().rev().find(|r| r.text.is_some()).unwrap();
        assert_eq!(last.text.as_deref(), Some("Succeeded on a roll of 2+ (AG 3+ Roll > 6)."));
    }
}
