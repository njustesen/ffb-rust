use crate::client::report::bb2016::nerves_of_steel_message::NervesOfSteelMessage;
use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_mechanics::bb2016::pass_mechanic::PassMechanic;
use ffb_mechanics::pass_mechanic::PassMechanic as PassMechanicTrait;
use ffb_model::model::game::Game;
use ffb_model::report::bb2016::report_nerves_of_steel::ReportNervesOfSteel;
use ffb_model::report::bb2016::report_pass_roll::ReportPassRoll;
use ffb_model::report::report_id::ReportId;

pub struct PassRollMessage;

impl ReportMessage for PassRollMessage {
    type Report = ReportPassRoll;

    fn report_id(&self) -> ReportId {
        ReportId::PASS_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let mut needed_roll: Option<String> = None;
        let mechanic = PassMechanic;
        let thrower = report.get_player_id().and_then(|id| game.player(id));
        if !report.is_re_rolled() {
            print_player(status_report, game, indent, true, thrower);
            let catcher = game.pass_coordinate.and_then(|c| game.field_model.player_at(c)).and_then(|id| game.player(id));
            if report.is_hail_mary_pass() {
                if report.is_bomb() {
                    status_report.println_indent_style(indent, TextStyle::BOLD, " throws a Hail Mary bomb:");
                } else {
                    status_report.println_indent_style(indent, TextStyle::BOLD, " throws a Hail Mary pass:");
                }
            } else if catcher.is_some() {
                if report.is_bomb() {
                    status_report.print_indent_style(indent, TextStyle::BOLD, " throws a bomb at ");
                } else {
                    status_report.print_indent_style(indent, TextStyle::BOLD, " passes the ball to ");
                }
                print_player(status_report, game, indent, true, catcher);
                status_report.println_indent_style(indent, TextStyle::BOLD, ":");
            } else if report.is_bomb() {
                status_report.println_indent_style(indent, TextStyle::BOLD, " throws a bomb to an empty field:");
            } else {
                status_report.println_indent_style(indent, TextStyle::BOLD, " passes the ball to an empty field:");
            }
        }
        // java: `report.hasRollModifier(pmf.forName("Nerves of Steel"))` — the resolved
        // ReportPassRoll only carries modifier name strings (no PassModifierFactory lookup),
        // so membership is checked directly against the modifier name.
        if report.get_roll_modifiers().iter().any(|m| m == "Nerves of Steel") {
            if let Some(player) = game.acting_player.player_id.as_deref().and_then(|id| game.player(id)) {
                let nerves_report = ReportNervesOfSteel::new(player.id.clone(), "pass".into());
                NervesOfSteelMessage.render(status_report, game, &nerves_report);
            }
        }

        let thrower_ref = thrower;
        let status = thrower_ref.map(|t| mechanic.format_report_roll(report.get_roll(), t)).unwrap_or_default();
        status_report.println_indent_style(indent + 1, TextStyle::ROLL, &status);
        print_player(status_report, game, indent + 2, false, thrower);
        let result = report.get_result();
        if result == "ACCURATE" {
            if report.is_bomb() {
                status_report.println_indent(indent + 2, " throws the bomb successfully.");
            } else {
                status_report.println_indent(indent + 2, " passes the ball.");
            }
            if !report.is_re_rolled() {
                needed_roll = Some(format!("Succeeded on a roll of {}+", report.get_minimum_roll()));
            }
        } else {
            if result == "SAVED_FUMBLE" {
                status_report.println_indent(indent + 2, " holds on to the ball.");
            } else if result == "FUMBLE" {
                if report.is_bomb() {
                    status_report.println_indent(indent + 2, " fumbles the bomb.");
                } else {
                    status_report.println_indent(indent + 2, " fumbles the ball.");
                }
            } else if result == "WILDLY_INACCURATE" {
                status_report.println_indent(indent + 2, " lets the throw deviate.");
            } else {
                status_report.println_indent(indent + 2, " misses the throw.");
            }
            if !report.is_re_rolled() {
                needed_roll = Some(format!("Roll a {}+ to succeed", report.get_minimum_roll()));
            }
        }
        if let Some(mut needed_roll) = needed_roll {
            if !report.is_hail_mary_pass() {
                if let (Some(distance_name), Some(thrower)) = (report.get_passing_distance(), thrower) {
                    if let Some(distance) = ffb_model::enums::PassingDistance::from_name(distance_name) {
                        needed_roll.push_str(&mechanic.format_roll_requirement(
                            distance,
                            &status_report.format_roll_modifiers(report.get_roll_modifiers()),
                            thrower,
                        ));
                    }
                }
            }
            status_report.println_indent_style(indent + 2, TextStyle::NEEDED_ROLL, &needed_roll);
        }
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
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            ..Default::default()
        });
        Game::new(home, make_team("away"), Rules::Bb2016)
    }

    #[test]
    fn get_key_is_pass_roll() {
        assert_eq!(PassRollMessage.get_key(), "passRoll");
    }

    #[test]
    fn accurate_pass_reports_success() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPassRoll::new(
            Some("thrower".into()), true, 5, 2, false, vec![], Some("Quick Pass".into()), false, false, "ACCURATE".into(),
        );
        PassRollMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" passes the ball.")));
    }

    #[test]
    fn fumble_reports_fumble() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPassRoll::new(
            Some("thrower".into()), false, 1, 2, false, vec![], Some("Quick Pass".into()), false, false, "FUMBLE".into(),
        );
        PassRollMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" fumbles the ball.")));
    }

    #[test]
    fn hail_mary_pass_skips_catcher_lookup() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPassRoll::new(
            Some("thrower".into()), true, 6, 2, false, vec![], None, true, false, "ACCURATE".into(),
        );
        PassRollMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" throws a Hail Mary pass:")));
    }
}
