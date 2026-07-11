use crate::client::paragraph_style::ParagraphStyle;
use crate::client::text_style::TextStyle;
use ffb_model::model::client_mode::ClientMode;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::report::i_report::IReport;
use ffb_model::report::report_list::ReportList;
use ffb_protocol::commands::server_command_join::ServerCommandJoin;
use ffb_protocol::commands::server_command_leave::ServerCommandLeave;
use ffb_protocol::server_status::ServerStatus;

/// One emitted (paragraph style, text style, text) run.
/// Java: `getUserInterface().getLog().append(pParagraphStyle, pTextStyle, pText)` — the
/// Swing log widget is replaced with this headless, testable sink.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedRun {
    pub paragraph_style: Option<ParagraphStyle>,
    pub text_style: Option<TextStyle>,
    pub text: Option<String>,
}

/// 1:1 translation of `StatusReport.java`. `FantasyFootballClient`/`UserInterface` are not
/// wired through here (headless); the client mode/game/team-coach data needed by the
/// original `getClient()` calls are passed in directly by callers instead.
pub struct StatusReport {
    indent: i32,
    pub petty_cash_report_received: bool,
    pub cards_bought_report_received: bool,
    pub inducements_bought_report_received: bool,
    /// Java: `getUserInterface().getLog().append(...)` — captured runs instead of painted.
    pub rendered_runs: Vec<RenderedRun>,
}

impl StatusReport {
    pub fn new() -> Self {
        Self {
            indent: 0,
            petty_cash_report_received: false,
            cards_bought_report_received: false,
            inducements_bought_report_received: false,
            rendered_runs: Vec::new(),
        }
    }

    pub fn get_indent(&self) -> i32 {
        self.indent
    }

    pub fn set_indent(&mut self, indent: i32) {
        self.indent = indent;
    }

    pub fn report_version(&mut self, version: &str) {
        self.println_indent(0, &format!("FantasyFootballClient Version {version}"));
    }

    pub fn report_connecting(&mut self, host: &str, port: i32) {
        self.println_indent(0, &format!("Connecting to {host}:{port} ..."));
    }

    pub fn report_icon_load_failure(&mut self, icon_url: &str) {
        self.println_indent(0, &format!("Unable to load icon from URL {icon_url}."));
    }

    pub fn report_timeout(&mut self) {
        self.println_style(
            Some(ParagraphStyle::SPACE_ABOVE_BELOW),
            Some(TextStyle::BOLD),
            "The timelimit has been reached for this turn.",
        );
    }

    pub fn report_game_name(&mut self, game_name: &str) {
        if !game_name.is_empty() {
            self.println_indent(0, &format!("You have started a new game named \"{game_name}\"."));
        }
    }

    pub fn report_socket_closed(&mut self, client_mode: ClientMode) {
        if client_mode != ClientMode::REPLAY {
            self.println_style(
                Some(ParagraphStyle::SPACE_ABOVE),
                Some(TextStyle::NONE),
                "The connection to the server has been closed.",
            );
            self.println_style(
                Some(ParagraphStyle::SPACE_BELOW),
                Some(TextStyle::NONE),
                "To re-connect you need to restart the client.",
            );
        }
    }

    pub fn report_connection_established(&mut self, successful: bool) {
        if successful {
            self.println_indent(0, "Connection established.");
        } else {
            self.println_indent(0, "Cannot connect to the server.");
        }
    }

    pub fn report_join(&mut self, game: &Game, join_command: &ServerCommandJoin) {
        match join_command.client_mode {
            ClientMode::PLAYER => {
                self.print_coach_name(game, &join_command.coach);
                self.println_indent_style(0, TextStyle::BOLD, " joins the game.");
            }
            ClientMode::SPECTATOR => {
                self.print_indent(0, "Spectator ");
                self.print_indent(0, &join_command.coach);
                self.println_indent(0, " joins the game.");
            }
            ClientMode::REPLAY => {}
        }
    }

    pub fn report_leave(&mut self, game: &Game, leave_command: &ServerCommandLeave) {
        match leave_command.client_mode {
            ClientMode::PLAYER => {
                self.print_coach_name(game, &leave_command.coach);
                self.println_indent_style(0, TextStyle::BOLD, " leaves the game.");
            }
            ClientMode::SPECTATOR => {
                self.print_indent(0, "Spectator ");
                self.print_indent(0, &leave_command.coach);
                self.println_indent(0, " leaves the game.");
            }
            ClientMode::REPLAY => {}
        }
    }

    fn print_coach_name(&mut self, game: &Game, coach: &str) {
        self.print_indent_style(0, TextStyle::BOLD, "Player ");
        if !game.team_home.coach.is_empty() && game.team_home.coach == coach {
            self.print_indent_style(0, TextStyle::HOME_BOLD, coach);
        } else {
            self.print_indent_style(0, TextStyle::AWAY_BOLD, coach);
        }
    }

    /// Java: `formatRollModifiers(RollModifier<?>[])`. The Rust report data model already
    /// resolves modifiers to plain name strings (no retained sign/magnitude), so each
    /// modifier is rendered as `" - <name>"` — the `isModifierIncluded()`/numeric-value
    /// branch has no equivalent data to draw on here.
    pub fn format_roll_modifiers(&self, roll_modifiers: &[String]) -> String {
        let mut modifiers = String::new();
        for name in roll_modifiers {
            modifiers.push_str(" - ");
            modifiers.push_str(name);
        }
        modifiers
    }

    pub fn report_status(&mut self, status: ServerStatus) {
        self.println();
        self.println_indent_style(0, TextStyle::BOLD, status.message());
        self.println();
    }

    pub fn report(&mut self, report: &dyn IReport, render: impl FnOnce(&mut Self, &dyn IReport)) {
        render(self, report);
    }

    pub fn report_list(&mut self, report_list: &ReportList, mut render: impl FnMut(&mut Self, &dyn IReport)) {
        for report in report_list.get_reports() {
            render(self, report.as_ref());
        }
    }

    fn find_paragraph_style(indent: i32) -> Option<ParagraphStyle> {
        match indent {
            0 => Some(ParagraphStyle::INDENT_0),
            1 => Some(ParagraphStyle::INDENT_1),
            2 => Some(ParagraphStyle::INDENT_2),
            3 => Some(ParagraphStyle::INDENT_3),
            4 => Some(ParagraphStyle::INDENT_4),
            5 => Some(ParagraphStyle::INDENT_5),
            6 => Some(ParagraphStyle::INDENT_6),
            _ => None,
        }
    }

    pub fn print_indent_style(&mut self, indent: i32, text_style: TextStyle, text: &str) {
        self.print_style(Self::find_paragraph_style(indent), Some(text_style), text);
    }

    pub fn print_indent(&mut self, indent: i32, text: &str) {
        self.print_style(Self::find_paragraph_style(indent), None, text);
    }

    pub fn print_style(&mut self, paragraph_style: Option<ParagraphStyle>, text_style: Option<TextStyle>, text: &str) {
        self.rendered_runs.push(RenderedRun {
            paragraph_style,
            text_style,
            text: Some(text.to_string()),
        });
    }

    pub fn println_indent_style(&mut self, indent: i32, text_style: TextStyle, text: &str) {
        self.println_style(Self::find_paragraph_style(indent), Some(text_style), text);
    }

    pub fn println_indent(&mut self, indent: i32, text: &str) {
        self.println_style(Self::find_paragraph_style(indent), None, text);
    }

    pub fn println(&mut self) {
        // Java: `println(findParagraphStyle(0), null, null)` — both the printed run and
        // its terminator carry a null text.
        self.rendered_runs.push(RenderedRun { paragraph_style: Self::find_paragraph_style(0), text_style: None, text: None });
        self.rendered_runs.push(RenderedRun { paragraph_style: None, text_style: None, text: None });
    }

    pub fn println_style(&mut self, paragraph_style: Option<ParagraphStyle>, text_style: Option<TextStyle>, text: &str) {
        self.print_style(paragraph_style, text_style, text);
        self.rendered_runs.push(RenderedRun { paragraph_style: None, text_style: None, text: None });
    }

    pub fn print_player(&mut self, indent: i32, bold: bool, player: Option<&Player>, is_home_player: bool) {
        if let Some(player) = player {
            let paragraph_style = Self::find_paragraph_style(indent);
            let text_style = match (is_home_player, bold) {
                (true, true) => TextStyle::HOME_BOLD,
                (true, false) => TextStyle::HOME,
                (false, true) => TextStyle::AWAY_BOLD,
                (false, false) => TextStyle::AWAY,
            };
            self.print_style(paragraph_style, Some(text_style), &player.name);
        }
    }

    pub fn print_team_name(&mut self, game: &Game, bold: bool, team_id: &str) {
        if game.team_home.id == team_id {
            if bold {
                self.print_indent_style(self.get_indent() + 1, TextStyle::HOME_BOLD, &game.team_home.name.clone());
            } else {
                self.print_indent_style(self.get_indent() + 1, TextStyle::HOME, &game.team_home.name.clone());
            }
        } else if bold {
            self.print_indent_style(self.get_indent() + 1, TextStyle::AWAY_BOLD, &game.team_away.name.clone());
        } else {
            self.print_indent_style(self.get_indent() + 1, TextStyle::AWAY, &game.team_away.name.clone());
        }
    }
}

impl Default for StatusReport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indent_get_set() {
        let mut report = StatusReport::new();
        assert_eq!(report.get_indent(), 0);
        report.set_indent(2);
        assert_eq!(report.get_indent(), 2);
    }

    #[test]
    fn report_version_emits_single_run() {
        let mut report = StatusReport::new();
        report.report_version("1.0");
        assert_eq!(report.rendered_runs.len(), 2);
        assert_eq!(report.rendered_runs[0].text.as_deref(), Some("FantasyFootballClient Version 1.0"));
        assert_eq!(report.rendered_runs[0].paragraph_style, Some(ParagraphStyle::INDENT_0));
    }

    #[test]
    fn report_timeout_uses_space_above_below_and_bold() {
        let mut report = StatusReport::new();
        report.report_timeout();
        assert_eq!(report.rendered_runs[0].paragraph_style, Some(ParagraphStyle::SPACE_ABOVE_BELOW));
        assert_eq!(report.rendered_runs[0].text_style, Some(TextStyle::BOLD));
    }

    #[test]
    fn report_game_name_skips_when_empty() {
        let mut report = StatusReport::new();
        report.report_game_name("");
        assert!(report.rendered_runs.is_empty());
    }

    #[test]
    fn format_roll_modifiers_joins_with_minus() {
        let report = StatusReport::new();
        let modifiers = vec!["TackleZone".to_string(), "Blizzard".to_string()];
        assert_eq!(report.format_roll_modifiers(&modifiers), " - TackleZone - Blizzard");
    }

    #[test]
    fn format_roll_modifiers_empty() {
        let report = StatusReport::new();
        assert_eq!(report.format_roll_modifiers(&[]), "");
    }

    #[test]
    fn println_emits_run_and_terminator() {
        let mut report = StatusReport::new();
        report.println();
        assert_eq!(report.rendered_runs.len(), 2);
        assert_eq!(report.rendered_runs[0].text, None);
        assert_eq!(report.rendered_runs[1].text, None);
    }

    #[test]
    fn report_status_brackets_message_with_blank_lines() {
        let mut report = StatusReport::new();
        report.report_status(ServerStatus::ErrorUnknownCoach);
        // blank println (2 runs) + message run/terminator (2 runs) + blank println (2 runs).
        assert_eq!(report.rendered_runs.len(), 6);
        assert_eq!(report.rendered_runs[2].text.as_deref(), Some("Unknown Coach!"));
    }
}
