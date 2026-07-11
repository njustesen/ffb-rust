use super::apothecary_roll_message::{player_state_description, serious_injury_description};
use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::report::bb2016::report_injury::ReportInjury;
use ffb_model::report::report_id::ReportId;

fn report_injury(status_report: &mut StatusReport, game: &Game, indent: i32, defender: Option<&Player>, injury: ffb_model::enums::PlayerState, serious_injury: Option<&str>) {
    print_player(status_report, game, indent + 1, false, defender);
    status_report.println_indent(indent + 1, &format!(" {}.", player_state_description(injury)));
    if let Some(name) = serious_injury {
        if let Some(description) = serious_injury_description(game, name) {
            print_player(status_report, game, indent + 1, false, defender);
            status_report.println_indent(indent + 1, &format!(" {}.", description));
        }
    }
}

pub struct InjuryMessage;

impl ReportMessage for InjuryMessage {
    type Report = ReportInjury;

    fn report_id(&self) -> ReportId {
        ReportId::INJURY
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let defender = game.player(report.get_defender_id());
        let attacker = report.get_attacker_id().and_then(|id| game.player(id));

        // java: report.getInjuryType().reportInjuryString(status, attacker, defender) — the
        // base InjuryType.reportInjuryString() is a no-op by default and only a handful of
        // edition-specific injury types (Foul, Chainsaw, Stab, ...) override it. The resolved
        // ReportInjury only carries the injury type's name string, not the type object, so
        // that per-type override text cannot be reconstructed here; omitted (matches the
        // common no-op case).

        // report armour roll
        let armor_roll = report.get_armor_roll();
        if !armor_roll.is_empty() {
            let status = format!("Armour Roll [ {} ][ {} ]", armor_roll[0], armor_roll[1]);
            status_report.println_indent_style(indent, TextStyle::ROLL, &status);
            let rolled_total = armor_roll[0] + armor_roll[1];
            // java: per-modifier ArmorModifier.getModifier(attacker, defender) values are not
            // preserved on the resolved report (only modifier names); the numeric total and
            // Claws-usage detection cannot be reconstructed, so only the rolled total and
            // modifier names (via formatRollModifiers-style rendering) are reported.
            let status = format!("Rolled Total of {}{}", rolled_total, status_report.format_roll_modifiers(report.get_armor_modifier_names()));
            status_report.println_indent(indent + 1, &status);
            if report.is_armor_broken() {
                status_report.print_indent(indent + 1, "The armour of ");
                print_player(status_report, game, indent + 1, false, defender);
                status_report.println_indent(indent + 1, " has been broken.");
            } else if let Some(defender) = defender {
                print_player(status_report, game, indent + 1, false, Some(defender));
                let status = format!(" has been saved by {} armour.", defender.gender.genitive());
                status_report.println_indent(indent + 1, &status);
            }
        }

        // report injury roll
        if report.is_armor_broken() {
            let injury_roll = report.get_injury_roll();
            if !injury_roll.is_empty() {
                let status = format!("Injury Roll [ {} ][ {} ]", injury_roll[0], injury_roll[1]);
                status_report.println_indent_style(indent, TextStyle::ROLL, &status);
                if let Some(defender) = defender {
                    if defender.zapped {
                        let status = format!(
                            "{} is badly hurt automatically because {} has been zapped.",
                            defender.name,
                            defender.gender.nominative()
                        );
                        status_report.println_indent_style(indent, TextStyle::NONE, &status);
                    } else {
                        let rolled_total = injury_roll[0] + injury_roll[1];
                        // java: InjuryModifier.getModifier(attacker, defender) values, Stunty
                        // and Thick Skull detection — not reconstructable from the resolved
                        // name-only modifier list; only the rolled total and modifier names
                        // are reported (see armour roll comment above).
                        let status = format!("Rolled Total of {}{}", rolled_total, status_report.format_roll_modifiers(report.get_injury_modifier_names()));
                        status_report.println_indent(indent + 1, &status);
                        if !report.get_casualty_roll().is_empty() {
                            print_player(status_report, game, indent + 1, false, Some(defender));
                            status_report.println_indent(indent + 1, " suffers a casualty.");
                            let casualty_roll = report.get_casualty_roll();
                            let status = format!("Casualty Roll [ {} ][ {} ]", casualty_roll[0], casualty_roll[1]);
                            status_report.println_indent_style(indent, TextStyle::ROLL, &status);
                            if let Some(injury) = report.get_injury() {
                                report_injury(status_report, game, indent, Some(defender), injury, report.get_serious_injury());
                            }
                            if !report.get_casualty_roll_decay().is_empty() {
                                print_player(status_report, game, indent + 1, false, Some(defender));
                                let status = format!(
                                    "'s body is decaying and {} suffers a 2nd casualty.",
                                    defender.gender.nominative()
                                );
                                status_report.println_indent(indent + 1, &status);
                                let casualty_roll_decay = report.get_casualty_roll_decay();
                                let status = format!("Casualty Roll [ {} ][ {} ]", casualty_roll_decay[0], casualty_roll_decay[1]);
                                status_report.println_indent_style(indent, TextStyle::ROLL, &status);
                                if let Some(injury_decay) = report.get_injury_decay() {
                                    report_injury(status_report, game, indent, Some(defender), injury_decay, report.get_serious_injury_decay());
                                }
                            }
                        } else if let Some(injury) = report.get_injury() {
                            report_injury(status_report, game, indent, Some(defender), injury, report.get_serious_injury());
                        }
                    }
                }
            }
        }
        let _ = attacker;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, PlayerType, Rules, PS_BADLY_HURT};
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team { id: id.into(), name: format!("Team {id}"), race: "Human".into(), roster_id: "human".into(), coach: "Coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0, special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false }
    }

    fn make_game() -> Game {
        let mut away = make_team("away");
        away.players.push(Player {
            id: "defender".into(), name: "Defender".into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            ..Default::default()
        });
        Game::new(make_team("home"), away, Rules::Bb2016)
    }

    #[test]
    fn get_key_is_injury() {
        assert_eq!(InjuryMessage.get_key(), "injury");
    }

    #[test]
    fn armor_saved_reports_no_break() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportInjury::new(
            "defender".into(), "".into(), false, vec![], vec![3, 3], vec![], vec![], vec![], None, vec![], None, None, None, None,
        );
        InjuryMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" has been saved by his armour.")));
    }

    #[test]
    fn armor_broken_reports_break_and_injury() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportInjury::new(
            "defender".into(), "".into(), true, vec![], vec![5, 5], vec![], vec![3, 4], vec![], None, vec![], None,
            Some(ffb_model::enums::PlayerState::new(PS_BADLY_HURT)), None, None,
        );
        InjuryMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" has been broken.")));
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" has been badly hurt.")));
    }

    #[test]
    fn casualty_roll_reports_casualty_and_injury() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportInjury::new(
            "defender".into(), "".into(), true, vec![], vec![5, 5], vec![], vec![3, 4], vec![2, 2], None, vec![], None,
            Some(ffb_model::enums::PlayerState::new(PS_BADLY_HURT)), None, None,
        );
        InjuryMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" suffers a casualty.")));
    }
}
