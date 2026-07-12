use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::inducement::bb2020::prayer::Prayer;
use ffb_model::model::game::Game;
use ffb_model::report::bb2020::report_prayer_roll::ReportPrayerRoll;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `PrayerRollMessage.java` (bb2020).
pub struct PrayerRollMessage;

/// Java: `Prayer.getDescription()` — the bb2020 `Prayer` enum in
/// `ffb_model::inducement::bb2020::prayer` does not yet expose a `get_description()` method
/// (only `get_name()`, `get_duration()`, `affects_both_teams()`, `is_changing_player()`,
/// `event_message()`). Ported here verbatim from
/// `ffb-java/ffb/ffb-common/src/main/java/com/fumbbl/ffb/inducement/bb2020/Prayer.java`'s
/// constructor description strings, since the field is out of scope for this file
/// (crates/ffb-client only).
fn prayer_description(prayer: Prayer) -> &'static str {
    match prayer {
        Prayer::TREACHEROUS_TRAPDOOR => {
            "Trapdoors appear. On a roll of 1 a player stepping on them falls through them"
        }
        Prayer::FRIENDS_WITH_THE_REF => "Argue the call succeeds on 5+",
        Prayer::STILETTO => {
            "One random player available to play during this drive without Loner gains Stab"
        }
        Prayer::IRON_MAN => {
            "One chosen player available to play during this drive without Loner improves AV by 1 (Max 11+)"
        }
        Prayer::KNUCKLE_DUSTERS => {
            "One chosen player available to play during this drive without Loner gains Mighty Blow (+1)"
        }
        Prayer::BAD_HABITS => {
            "D3 random opponent players available to play during this drive without Loner gain Loner (2+)"
        }
        Prayer::GREASY_CLEATS => {
            "One random opponent player available to play during this drive has his MA reduced by 1"
        }
        Prayer::BLESSED_STATUE_OF_NUFFLE => {
            "One chosen player available to play during this drive without Loner gains Pro"
        }
        Prayer::MOLES_UNDER_THE_PITCH => {
            "Rushes have a -1 modifier (-2 if both coaches rolled this result)"
        }
        Prayer::PERFECT_PASSING => "Completions generate 2 instead of 1 spp",
        Prayer::FAN_INTERACTION => "Casualties caused by crowd pushes generate 2 spp",
        Prayer::NECESSARY_VIOLENCE => "Casualties generate 3 instead of 2 spp",
        Prayer::FOULING_FRENZY => "Casualties caused by fouls generate 2 spp",
        Prayer::THROW_A_ROCK => {
            "If an opposing player should stall they get hit by a rock on a 5+ and are knocked down immediately"
        }
        Prayer::UNDER_SCRUTINY => "Fouls by opposing players are always spotted",
        Prayer::INTENSIVE_TRAINING => {
            "One random player available to play during this drive without Loner gains a chosen Primary skill"
        }
    }
}

impl ReportMessage for PrayerRollMessage {
    type Report = ReportPrayerRoll;

    fn report_id(&self) -> ReportId {
        ReportId::PRAYER_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        // java: game.<PrayerFactory>getFactory(FactoryType.Factory.PRAYER).forRoll(report.getRoll())
        // Java: `((GameOptionBoolean) game.getOptions().getOptionWithDefault(...)).isEnabled()`.
        let use_league_table = game
            .options
            .get_option_with_default(ffb_model::option::game_option_id::GameOptionId::INDUCEMENT_PRAYERS_USE_LEAGUE_TABLE)
            .get_value_as_string()
            == "true";
        let mut factory = ffb_model::factory::bb2020::prayer_factory::PrayerFactory::new();
        factory.initialize(use_league_table);
        let prayer = ffb_model::factory::prayer_factory::PrayerFactory::for_roll(&factory, report.get_roll());

        status_report.println_indent_style(
            status_report.get_indent(),
            TextStyle::ROLL,
            &format!("Prayer Roll [ {} ]", report.get_roll()),
        );

        if let Some(prayer) = prayer {
            status_report.println_indent_style(status_report.get_indent() + 1, TextStyle::BOLD, prayer.get_name());
            status_report.println_indent_style(
                status_report.get_indent() + 2,
                TextStyle::EXPLANATION,
                &format!("{}: {}", prayer.get_duration().get_description(), prayer_description(prayer)),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;

    fn make_game() -> Game {
        let home = Team {
            id: "home".into(), name: "home".into(), race: String::new(),
            roster_id: String::new(), coach: String::new(), rerolls: 0,
            apothecaries: 0, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0,
            bloodweiser_kegs: 0, riotous_rookies: 0, fan_factor: 0,
            assistant_coaches: 0, cheerleaders: 0, dedicated_fans: 0,
            treasury: 0, team_value: 0, players: vec![], special_rules: vec![],
            vampire_lord: false,
            necromancer: false,
        };
        let away = Team { id: "away".into(), ..home.clone() };
        Game::new(home, away, Rules::Bb2020)
    }

    #[test]
    fn report_id_and_key() {
        assert_eq!(PrayerRollMessage.report_id(), ReportId::PRAYER_ROLL);
        assert_eq!(PrayerRollMessage.get_key(), "prayerRoll");
    }

    #[test]
    fn renders_roll_line_and_prayer_details_for_treacherous_trapdoor() {
        let game = make_game();
        let report = ReportPrayerRoll::new(1);
        let mut status_report = StatusReport::new();
        PrayerRollMessage.render(&mut status_report, &game, &report);

        // Each println_*_style call pushes a content run followed by a `None` terminator run.
        assert_eq!(status_report.rendered_runs.len(), 6);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Prayer Roll [ 1 ]"));
        assert_eq!(status_report.rendered_runs[0].text_style, Some(TextStyle::ROLL));
        assert_eq!(status_report.rendered_runs[2].text.as_deref(), Some("Treacherous Trapdoor"));
        assert_eq!(status_report.rendered_runs[2].text_style, Some(TextStyle::BOLD));
        assert_eq!(
            status_report.rendered_runs[4].text.as_deref(),
            Some("For this half: Trapdoors appear. On a roll of 1 a player stepping on them falls through them")
        );
        assert_eq!(status_report.rendered_runs[4].text_style, Some(TextStyle::EXPLANATION));
    }

    #[test]
    fn renders_perfect_passing_for_roll_10() {
        let game = make_game();
        let report = ReportPrayerRoll::new(10);
        let mut status_report = StatusReport::new();
        PrayerRollMessage.render(&mut status_report, &game, &report);

        assert_eq!(status_report.rendered_runs[2].text.as_deref(), Some("Perfect Passing"));
        assert_eq!(
            status_report.rendered_runs[4].text.as_deref(),
            Some("For the entire game: Completions generate 2 instead of 1 spp")
        );
    }

    #[test]
    fn out_of_range_roll_only_renders_roll_line() {
        let game = make_game();
        let report = ReportPrayerRoll::new(99);
        let mut status_report = StatusReport::new();
        PrayerRollMessage.render(&mut status_report, &game, &report);

        assert_eq!(status_report.rendered_runs.len(), 2);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Prayer Roll [ 99 ]"));
    }
}
