use crate::client::report::bb2016::nerves_of_steel_message::NervesOfSteelMessage;
use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::enums::PassingDistance;
use ffb_model::model::game::Game;
use ffb_model::report::bb2016::report_nerves_of_steel::ReportNervesOfSteel;
use ffb_model::report::bb2016::report_throw_team_mate_roll::ReportThrowTeamMateRoll;
use ffb_model::report::report_id::ReportId;

pub struct ThrowTeamMateRollMessage;

impl ReportMessage for ThrowTeamMateRollMessage {
    type Report = ReportThrowTeamMateRoll;

    fn report_id(&self) -> ReportId {
        ReportId::THROW_TEAM_MATE_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let thrower = game.acting_player.player_id.as_deref().and_then(|id| game.player(id));
        let thrown_player = game.player(report.get_thrown_player_id());
        let mut needed_roll: Option<String> = None;
        if !report.is_re_rolled() {
            print_player(status_report, game, indent, true, thrower);
            status_report.print_indent_style(indent, TextStyle::BOLD, " tries to throw ");
            print_player(status_report, game, indent, true, thrown_player);
            status_report.println_indent_style(indent, TextStyle::BOLD, ":");
        }
        // java: `report.hasRollModifier(pmf.forName("Nerves of Steel"))` — the resolved
        // ReportThrowTeamMateRoll only carries modifier name strings, so membership is
        // checked directly against the modifier name (see PassRollMessage for the same gap).
        if report.get_roll_modifiers().iter().any(|m| m == "Nerves of Steel") {
            if let Some(player) = thrower {
                let nerves_report = ReportNervesOfSteel::new(player.id.clone(), "pass".into());
                NervesOfSteelMessage.render(status_report, game, &nerves_report);
            }
        }
        let status = format!("Throw Team-Mate Roll [ {} ]", report.get_roll());
        status_report.println_indent_style(indent + 1, TextStyle::ROLL, &status);
        print_player(status_report, game, indent + 2, false, thrower);
        if report.is_successful() {
            let gender = thrower.map(|p| p.gender).unwrap_or_default();
            let status = format!(" throws {} team-mate successfully.", gender.genitive());
            status_report.println_indent(indent + 2, &status);
        } else {
            status_report.println_indent(indent + 2, " fumbles the throw.");
        }
        if report.is_successful() && !report.is_re_rolled() {
            needed_roll = Some(format!("Succeeded on a roll of {}+ to avoid a fumble", report.get_minimum_roll()));
        }
        if !report.is_successful() && !report.is_re_rolled() {
            needed_roll = Some(format!("Roll a {}+ to avoid a fumble", report.get_minimum_roll()));
        }
        if let Some(mut needed_roll) = needed_roll {
            needed_roll.push_str(" (Roll ");
            if let Some(passing_distance) = report.get_passing_distance().and_then(PassingDistance::from_name) {
                if passing_distance.modifier_2016() >= 0 {
                    needed_roll.push_str(" + ");
                } else {
                    needed_roll.push_str(" - ");
                }
                needed_roll.push_str(&format!("{} {}", passing_distance.modifier_2016().abs(), passing_distance.name()));
            }
            needed_roll.push_str(&status_report.format_roll_modifiers(report.get_roll_modifiers()));
            needed_roll.push_str(" > 1).");
            status_report.println_indent_style(indent + 2, TextStyle::NEEDED_ROLL, &needed_roll);
        }
        status_report.set_indent(status_report.get_indent() + 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, PlayerType, Rules};
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
            id: "thrower".into(), name: "Thrower".into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 5, agility: 3, passing: 4, armour: 8,
            ..Default::default()
        });
        home.players.push(Player {
            id: "thrown".into(), name: "Thrown".into(), nr: 2, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            ..Default::default()
        });
        let mut game = Game::new(home, make_team("away"), Rules::Bb2016);
        game.acting_player.player_id = Some("thrower".into());
        game
    }

    #[test]
    fn get_key_is_throw_team_mate_roll() {
        assert_eq!(ThrowTeamMateRollMessage.get_key(), "throwTeamMateRoll");
    }

    #[test]
    fn successful_throw_reports_success_and_needed_roll() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportThrowTeamMateRoll::new(Some("thrower".into()), true, 4, 2, false, vec![], Some("Quick Pass".into()), "thrown".into());
        ThrowTeamMateRollMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" throws his team-mate successfully.")));
        let needed = status_report.rendered_runs.iter().find(|r| r.text_style == Some(TextStyle::NEEDED_ROLL)).unwrap();
        assert_eq!(needed.text.as_deref(), Some("Succeeded on a roll of 2+ to avoid a fumble (Roll  + 1 Quick Pass > 1)."));
    }

    #[test]
    fn failed_throw_reports_fumble() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportThrowTeamMateRoll::new(Some("thrower".into()), false, 1, 2, false, vec![], Some("Quick Pass".into()), "thrown".into());
        ThrowTeamMateRollMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" fumbles the throw.")));
    }

    #[test]
    fn indent_incremented_after_render() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportThrowTeamMateRoll::new(Some("thrower".into()), true, 4, 2, true, vec![], None, "thrown".into());
        let before = status_report.get_indent();
        ThrowTeamMateRollMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.get_indent(), before + 1);
    }
}
