use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_argue_the_call_roll::ReportArgueTheCallRoll;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `ArgueTheCallMessage.java`.
pub struct ArgueTheCallMessage;

impl ReportMessage for ArgueTheCallMessage {
    type Report = ReportArgueTheCallRoll;

    fn report_id(&self) -> ReportId {
        ReportId::ARGUE_THE_CALL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let player = report.get_player_id().and_then(|id| game.player(id));

        status_report.println_indent_style(indent, TextStyle::ROLL, &format!("Argue the Call Roll [ {} ]", report.get_roll()));

        let mut minimum_roll = 6;
        if report.is_friends_with_ref() {
            status_report.println_indent_style(
                indent + 1,
                TextStyle::EXPLANATION,
                "Being friends with the ref allows argue to succeed on 5+.",
            );
            minimum_roll = 5;
        }

        let target = minimum_roll;
        let biased_refs = report.get_biased_refs();
        let mut builder = String::new();
        if biased_refs > 0 {
            builder.push_str(&format!(" + {} Biased Referee", biased_refs));
            if biased_refs > 1 {
                builder.push('s');
            }
            minimum_roll -= biased_refs;
        }
        let modifiers = builder;

        if report.is_successful() {
            status_report.print_indent(indent + 1, "The ref refrains from banning ");
            print_player(status_report, game, indent + 1, false, player);
            let mut status = String::new();
            let is_box = player.and_then(|p| game.field_model.player_coordinate(&p.id)).map(|c| c.is_box_coordinate()).unwrap_or(false);
            if !is_box {
                status.push_str(&format!(" and {}", player.map(|p| p.gender.nominative()).unwrap_or("")));
                if report.is_stays_on_pitch() {
                    status.push_str(" stays on the pitch");
                } else {
                    status.push_str(" is sent to the reserve instead");
                }
            }
            status.push('.');
            status_report.println_indent_style(indent + 1, TextStyle::NONE, &status);
            status_report.println_indent_style(
                indent + 1,
                TextStyle::NEEDED_ROLL,
                &format!("Succeeded on a roll of {} (Roll{} >= {})", minimum_roll, modifiers, target),
            );
        } else {
            status_report.print_indent(indent + 1, "The ref bans ");
            print_player(status_report, game, indent + 1, false, player);
            status_report.println_indent_style(indent + 1, TextStyle::NONE, " from the game.");
            status_report.println_indent_style(
                indent + 1,
                TextStyle::NEEDED_ROLL,
                &format!("Would have succeeded on a roll of {} (Roll{} >= {})", minimum_roll, modifiers, target),
            );
        }

        if report.is_coach_banned() {
            status_report.print_indent_style(indent + 1, TextStyle::NONE, "Coach ");
            let is_home = player.is_some_and(|p| game.team_home.has_player(&p.id));
            if is_home {
                status_report.print_indent_style(indent + 1, TextStyle::HOME, &game.team_home.coach.clone());
            } else {
                status_report.print_indent_style(indent + 1, TextStyle::AWAY, &game.team_away.coach.clone());
            }
            status_report.println_indent_style(indent + 1, TextStyle::NONE, " is also banned from the game.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_team(id: &str, coach: &str, players: Vec<Player>) -> Team {
        Team {
            id: id.into(),
            name: format!("Team {id}"),
            race: "Human".into(),
            roster_id: "human".into(),
            coach: coach.into(),
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
        let player = Player { id: "p1".into(), name: "Grobnik".into(), gender: PlayerGender::Male, ..Player::default() };
        Game::new(
            make_team("home", "HomeCoach", vec![player]),
            make_team("away", "AwayCoach", vec![]),
            Rules::Bb2020,
        )
    }

    #[test]
    fn successful_reports_stays_on_pitch() {
        let game = make_game();
        let report = ReportArgueTheCallRoll::new(Some("p1".into()), true, false, 6, true, false, 0);
        let mut sr = StatusReport::new();
        ArgueTheCallMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[0].text.as_deref(), Some("Argue the Call Roll [ 6 ]"));
        assert_eq!(sr.rendered_runs[2].text.as_deref(), Some("The ref refrains from banning "));
        assert_eq!(sr.rendered_runs[3].text.as_deref(), Some("Grobnik"));
        assert_eq!(sr.rendered_runs[4].text.as_deref(), Some(" and he stays on the pitch."));
        assert_eq!(sr.rendered_runs[6].text.as_deref(), Some("Succeeded on a roll of 6 (Roll >= 6)"));
    }

    #[test]
    fn successful_reports_sent_to_reserve() {
        let game = make_game();
        let report = ReportArgueTheCallRoll::new(Some("p1".into()), true, false, 6, false, false, 0);
        let mut sr = StatusReport::new();
        ArgueTheCallMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[4].text.as_deref(), Some(" and he is sent to the reserve instead."));
    }

    #[test]
    fn failure_reports_banned() {
        let game = make_game();
        let report = ReportArgueTheCallRoll::new(Some("p1".into()), false, false, 3, false, false, 0);
        let mut sr = StatusReport::new();
        ArgueTheCallMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[2].text.as_deref(), Some("The ref bans "));
        assert_eq!(sr.rendered_runs[4].text.as_deref(), Some(" from the game."));
        assert_eq!(sr.rendered_runs[6].text.as_deref(), Some("Would have succeeded on a roll of 6 (Roll >= 6)"));
    }

    #[test]
    fn friends_with_ref_lowers_target_and_adds_explanation() {
        let game = make_game();
        let report = ReportArgueTheCallRoll::new(Some("p1".into()), true, false, 5, true, true, 0);
        let mut sr = StatusReport::new();
        ArgueTheCallMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[0].text.as_deref(), Some("Argue the Call Roll [ 5 ]"));
        assert_eq!(sr.rendered_runs[2].text.as_deref(), Some("Being friends with the ref allows argue to succeed on 5+."));
    }

    #[test]
    fn biased_refs_add_modifier_and_lower_minimum() {
        let game = make_game();
        let report = ReportArgueTheCallRoll::new(Some("p1".into()), true, false, 6, true, false, 2);
        let mut sr = StatusReport::new();
        ArgueTheCallMessage.render(&mut sr, &game, &report);
        assert_eq!(
            sr.rendered_runs[6].text.as_deref(),
            Some("Succeeded on a roll of 4 (Roll + 2 Biased Referees >= 6)")
        );
    }

    #[test]
    fn coach_banned_prints_home_coach() {
        let game = make_game();
        let report = ReportArgueTheCallRoll::new(Some("p1".into()), false, true, 2, false, false, 0);
        let mut sr = StatusReport::new();
        ArgueTheCallMessage.render(&mut sr, &game, &report);
        let coach_name_run = sr.rendered_runs.iter().find(|r| r.text.as_deref() == Some("HomeCoach")).unwrap();
        assert_eq!(coach_name_run.text_style, Some(TextStyle::HOME));
        let last_text = sr.rendered_runs.iter().rev().find(|r| r.text.is_some()).unwrap();
        assert_eq!(last_text.text.as_deref(), Some(" is also banned from the game."));
    }
}
