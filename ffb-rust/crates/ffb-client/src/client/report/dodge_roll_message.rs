use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_mechanics::agility_mechanic::AgilityMechanic as AgilityMechanicTrait;
use ffb_mechanics::modifiers::RollModifier;
use ffb_model::enums::Rules;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::mixed::report_dodge_roll::ReportDodgeRoll;

/// Java: `game.getRules().getFactory(Factory.MECHANIC).forName(Mechanic.Type.AGILITY.name())`.
/// Mirrors the edition dispatch idiom used in `ffb-engine/src/mechanic/mod.rs`.
fn agility_mechanic_for(rules: Rules) -> Box<dyn AgilityMechanicTrait> {
    match rules {
        Rules::Bb2016 => Box::new(ffb_mechanics::bb2016::agility_mechanic::AgilityMechanic::new()),
        Rules::Bb2020 => Box::new(ffb_mechanics::bb2020::agility_mechanic::AgilityMechanic::new()),
        Rules::Bb2025 | Rules::Common => Box::new(ffb_mechanics::bb2025::agility_mechanic::AgilityMechanic::new()),
    }
}

/// 1:1 translation of `DodgeRollMessage.java`.
///
/// Java: the renderer is declared generically as `ReportMessageBase<ReportSkillRoll>` (the
/// abstract base class implements `IReport`, so any concrete subclass instance -- bb2016's
/// `ReportDodgeRoll` or bb2020/bb2025's `mixed.ReportDodgeRoll` -- can satisfy it at runtime).
/// The Rust model instead ported those as two distinct, non-unified structs and left the base
/// `ReportSkillRoll` without an `IReport` impl (it has no fixed `ReportId` of its own), so the
/// `ReportMessage` trait's `type Report: IReport` bound cannot be satisfied by the base type
/// directly. This picks `mixed::ReportDodgeRoll` (the bb2020/bb2025 variant) as the concrete
/// type -- its fields are a superset of the base `ReportSkillRoll` fields this render logic
/// actually reads, so the rendering below is identical for either edition's report.
pub struct DodgeRollMessage;

impl ReportMessage for DodgeRollMessage {
    type Report = ReportDodgeRoll;

    fn report_id(&self) -> ReportId {
        ReportId::DODGE_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let player = report.get_player_id().and_then(|id| game.player(id));

        let status = if report.get_roll() > 0 {
            format!("Dodge Roll [ {} ]", report.get_roll())
        } else {
            "New Dodge Result".to_string()
        };
        status_report.println_indent_style(status_report.get_indent(), TextStyle::ROLL, &status);

        if !report.is_re_rolled() {
            if let Some(player) = player {
                if player.has_skill_property(NamedProperties::IGNORE_TACKLEZONES_WHEN_DODGING) {
                    print_player(status_report, game, status_report.get_indent() + 1, false, Some(player));
                    status_report.println_indent(status_report.get_indent() + 1, " is Stunty and ignores tacklezones.");
                }
                // java: Arrays.stream(report.getRollModifiers()).anyMatch(modifier -> modifier
                // instanceof DodgeModifier && ((DodgeModifier) modifier).isUseStrength()) --
                // ReportSkillRoll only retains modifier *names* (Vec<String>), not the typed
                // DodgeModifier objects, so this checks for the well-known "Break Tackle"
                // modifier name (the only DodgeModifier registered with use_strength=true; see
                // ffb-mechanics/src/modifiers/dodge_modifier_factory.rs) instead of the type.
                if report.get_roll_modifiers().iter().any(|m| m == "Break Tackle") {
                    print_player(status_report, game, status_report.get_indent() + 1, false, Some(player));
                    status_report.println_indent(status_report.get_indent() + 1, " uses Break Tackle to break free.");
                }
            }
        }

        print_player(status_report, game, status_report.get_indent() + 1, false, player);
        let mut needed_roll: Option<String> = None;
        if report.is_successful() {
            status_report.println_indent(status_report.get_indent() + 1, " dodges successfully.");
            if !report.is_re_rolled() {
                needed_roll = Some(format!("Succeeded on a roll of {}+", report.get_minimum_roll()));
            }
        } else {
            status_report.println_indent(status_report.get_indent() + 1, " trips while dodging.");
            if !report.is_re_rolled() {
                needed_roll = Some(format!("Roll a {}+ to succeed", report.get_minimum_roll()));
            }
        }

        if let Some(mut needed_roll) = needed_roll {
            if let Some(player) = player {
                // java: mechanic.formatDodgeResult(report, player) -- ReportSkillRoll only
                // retains modifier *names* (roll_modifier_names: Vec<String>), not the signed
                // magnitude/report-string data RollModifier objects carry in Java, so the
                // reconstructed modifiers below use modifier=0 for each name.
                let modifiers: Vec<RollModifier> =
                    report.get_roll_modifiers().iter().map(|name| RollModifier::new(name.clone(), 0)).collect();
                let mechanic = agility_mechanic_for(game.rules);
                needed_roll.push_str(&mechanic.format_dodge_result(&modifiers, player, None));
            }
            status_report.println_indent_style(status_report.get_indent() + 1, TextStyle::NEEDED_ROLL, &needed_roll);
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
        player.name = "Dodger".into();
        player.gender = PlayerGender::Male;
        player.agility = 3;
        game.team_home.players.push(player);
        game
    }

    #[test]
    fn successful_dodge_prints_success_message() {
        let mut status_report = StatusReport::new();
        let game = make_game_with_player();
        let report = ReportDodgeRoll::new(Some("p1".into()), true, 4, 3, false, vec![], None);
        DodgeRollMessage.render(&mut status_report, &game, &report);
        let msg = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some(" dodges successfully."));
        assert!(msg.is_some());
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Dodge Roll [ 4 ]"));
    }

    #[test]
    fn failed_dodge_prints_trips_message_and_needed_roll() {
        let mut status_report = StatusReport::new();
        let game = make_game_with_player();
        let report = ReportDodgeRoll::new(Some("p1".into()), false, 1, 3, false, vec![], None);
        DodgeRollMessage.render(&mut status_report, &game, &report);
        let msg = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some(" trips while dodging."));
        assert!(msg.is_some());
        let needed = status_report.rendered_runs.iter().find(|r| r.text_style == Some(TextStyle::NEEDED_ROLL));
        assert!(needed.unwrap().text.as_deref().unwrap().starts_with("Roll a 3+ to succeed"));
    }

    #[test]
    fn zero_roll_shows_new_dodge_result() {
        let mut status_report = StatusReport::new();
        let game = make_game_with_player();
        let report = ReportDodgeRoll::new(Some("p1".into()), true, 0, 3, false, vec![], None);
        DodgeRollMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("New Dodge Result"));
    }

    #[test]
    fn break_tackle_modifier_prints_break_free_message() {
        let mut status_report = StatusReport::new();
        let game = make_game_with_player();
        let report = ReportDodgeRoll::new(Some("p1".into()), true, 4, 3, false, vec!["Break Tackle".into()], None);
        DodgeRollMessage.render(&mut status_report, &game, &report);
        let msg = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some(" uses Break Tackle to break free."));
        assert!(msg.is_some());
    }

    #[test]
    fn re_rolled_skips_intro_lines_and_needed_roll() {
        let mut status_report = StatusReport::new();
        let game = make_game_with_player();
        let report = ReportDodgeRoll::new(Some("p1".into()), true, 4, 3, true, vec!["Break Tackle".into()], None);
        DodgeRollMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().all(|r| r.text.as_deref() != Some(" uses Break Tackle to break free.")));
        assert!(status_report.rendered_runs.iter().all(|r| r.text_style != Some(TextStyle::NEEDED_ROLL)));
    }

    #[test]
    fn report_id_is_dodge_roll() {
        assert_eq!(DodgeRollMessage.report_id(), ReportId::DODGE_ROLL);
    }
}
