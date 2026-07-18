use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::bb2016::report_hypnotic_gaze_roll::ReportHypnoticGazeRoll;
use ffb_model::report::report_id::ReportId;

pub struct HypnoticGazeRollMessage;

impl ReportMessage for HypnoticGazeRollMessage {
    type Report = ReportHypnoticGazeRoll;

    fn report_id(&self) -> ReportId {
        ReportId::HYPNOTIC_GAZE_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let player = game.acting_player.player_id.as_deref().and_then(|id| game.player(id));
        let defender = game.defender_id.as_deref().and_then(|id| game.player(id));
        let mut needed_roll: Option<String> = None;
        if !report.is_re_rolled() {
            print_player(status_report, game, indent, true, player);
            status_report.print_indent_style(indent, TextStyle::BOLD, " gazes upon ");
            print_player(status_report, game, indent, true, defender);
            status_report.println_indent_style(indent, TextStyle::BOLD, ":");
        }
        let status = format!("Hypnotic Gaze Roll [ {} ]", report.get_roll());
        status_report.println_indent_style(indent + 1, TextStyle::ROLL, &status);
        print_player(status_report, game, indent + 2, false, player);
        let gender = player.map(|p| p.gender).unwrap_or_default();
        if report.is_successful() {
            status_report.println_indent(indent + 2, &format!(" hypnotizes {} victim.", gender.genitive()));
            if !report.is_re_rolled() {
                needed_roll = Some(format!("Succeeded on a roll of {}+", report.get_minimum_roll()));
            }
        } else {
            status_report.println_indent(indent + 2, &format!(" fails to affect {} victim.", gender.genitive()));
            if !report.is_re_rolled() {
                needed_roll = Some(format!("Roll a {}+ to succeed", report.get_minimum_roll()));
            }
        }
        if let Some(mut needed_roll) = needed_roll {
            // java: AgilityMechanic.formatHypnoticGazeResult(report, player) — reimplemented
            // locally since the mechanic expects unresolved RollModifier structs, while the
            // resolved ReportSkillRoll only carries modifier name strings.
            if let Some(player) = player {
                // java: `" (AG " + AG + formatRollModifiers(mods) + " + Roll > 6)."` — the
                // literal has a space before "+ Roll", which `formatRollModifiers` does not
                // supply (it only prefixes/suffixes around each modifier, e.g. " - 1 Foo").
                needed_roll.push_str(&format!(
                    " (AG {}{} + Roll > 6).",
                    player.agility_with_modifiers().min(6),
                    status_report.format_roll_modifiers(report.get_roll_modifiers())
                ));
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
        Team { id: id.into(), name: "Team".into(), race: "Human".into(), roster_id: "human".into(), coach: "Coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0, special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false }
    }

    fn make_game() -> Game {
        let mut home = make_team("home");
        home.players.push(Player {
            id: "gazer".into(), name: "Gazer".into(), nr: 1, position_id: "basilisk".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 4, passing: 4, armour: 8,
            ..Default::default()
        });
        let mut away = make_team("away");
        away.players.push(Player {
            id: "victim".into(), name: "Victim".into(), nr: 2, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Female,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            ..Default::default()
        });
        let mut game = Game::new(home, away, Rules::Bb2016);
        game.acting_player.player_id = Some("gazer".into());
        game.defender_id = Some("victim".into());
        game
    }

    #[test]
    fn get_key_is_hypnotic_gaze_roll() {
        assert_eq!(HypnoticGazeRollMessage.get_key(), "hypnoticGazeRoll");
    }

    #[test]
    fn successful_first_roll_prints_intro_and_hypnotizes() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportHypnoticGazeRoll::new(None, true, 5, 2, false, vec![]);
        HypnoticGazeRollMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Gazer"));
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" hypnotizes his victim.")));
    }

    #[test]
    fn re_rolled_skips_intro_line() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportHypnoticGazeRoll::new(None, false, 1, 2, true, vec![]);
        HypnoticGazeRollMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Hypnotic Gaze Roll [ 1 ]"));
    }

    #[test]
    fn failed_roll_reports_needed_roll_with_agility() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportHypnoticGazeRoll::new(None, false, 1, 2, false, vec![]);
        HypnoticGazeRollMessage.render(&mut status_report, &game, &report);
        let needed = status_report.rendered_runs.iter().find(|r| r.text_style == Some(TextStyle::NEEDED_ROLL)).unwrap();
        // java: `" (AG " + AG + formatRollModifiers(mods) + " + Roll > 6)."` — with no roll
        // modifiers, this must keep the space between the AG value and "+ Roll" (previously
        // rendered as "AG 4+ Roll", dropping the space).
        assert_eq!(needed.text.as_deref(), Some("Roll a 2+ to succeed (AG 4 + Roll > 6)."));
    }

    #[test]
    fn failed_roll_reports_needed_roll_with_agility_and_modifier() {
        // Regression test: with a roll modifier present, `formatRollModifiers` returns
        // " - 1 Foo" (no trailing space), so Java's literal `" + Roll > 6)."` must still
        // contribute its own leading space, giving "... Foo + Roll > 6).". Previously the
        // Rust format string glued them together as "...Foo+ Roll > 6)." with no space.
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportHypnoticGazeRoll::new(None, false, 1, 2, false, vec!["Foo".into()]);
        HypnoticGazeRollMessage.render(&mut status_report, &game, &report);
        let needed = status_report.rendered_runs.iter().find(|r| r.text_style == Some(TextStyle::NEEDED_ROLL)).unwrap();
        assert_eq!(needed.text.as_deref(), Some("Roll a 2+ to succeed (AG 4 - Foo + Roll > 6)."));
    }
}
