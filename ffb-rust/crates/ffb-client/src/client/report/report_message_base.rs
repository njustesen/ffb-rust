use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::enums::Direction;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::report::i_report::IReport;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `ReportMessageBase<T extends IReport>`. Rust has no class
/// inheritance, so this becomes a trait; concrete `*Message` structs implement it against
/// their concrete `ReportXxx` type (Java's generic `T`).
pub trait ReportMessage {
    type Report: IReport;

    /// Java: `getClass().getAnnotation(ReportMessageType.class).value()`.
    fn report_id(&self) -> ReportId;

    /// Java: `IKeyedItem.getKey()`.
    fn get_key(&self) -> &'static str {
        self.report_id().get_name()
    }

    /// Java: `renderMessage(Game, IReport)` — the cast-and-dispatch entry point.
    fn render_message(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        self.render(status_report, game, report);
    }

    /// Java: `protected abstract void render(T report)`.
    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report);
}

/// Java: `mapToLocal(Direction)` — delegates to
/// `statusReport.getClient().getUserInterface().getPitchDimensionProvider()`, which has no
/// headless equivalent yet. Kept as a free function so callers can pass the mapping in
/// explicitly instead of reaching through a UI layer.
pub fn map_to_local(direction: Direction) -> Direction {
    direction
}

/// Java: `print(int, boolean, Player<?>)` passthrough, resolving home/away from `game`.
pub fn print_player(status_report: &mut StatusReport, game: &Game, indent: i32, bold: bool, player: Option<&Player>) {
    let is_home = player.is_some_and(|p| game.team_home.has_player(&p.id));
    status_report.print_player(indent, bold, player, is_home);
}

/// Java: `printTeamName(boolean, String)` passthrough.
pub fn print_team_name(status_report: &mut StatusReport, game: &Game, bold: bool, team_id: &str) {
    status_report.print_team_name(game, bold, team_id);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::report::report_always_hungry_roll::ReportAlwaysHungryRoll;

    struct StubMessage;

    impl ReportMessage for StubMessage {
        type Report = ReportAlwaysHungryRoll;

        fn report_id(&self) -> ReportId {
            ReportId::ALWAYS_HUNGRY_ROLL
        }

        fn render(&self, status_report: &mut StatusReport, _game: &Game, report: &Self::Report) {
            status_report.println_indent_style(0, TextStyle::ROLL, &format!("roll {}", report.get_roll()));
        }
    }

    #[test]
    fn get_key_matches_report_id_name() {
        assert_eq!(StubMessage.get_key(), "alwaysHungryRoll");
    }

    #[test]
    fn map_to_local_is_identity_placeholder() {
        assert_eq!(map_to_local(Direction::North), Direction::North);
    }
}
