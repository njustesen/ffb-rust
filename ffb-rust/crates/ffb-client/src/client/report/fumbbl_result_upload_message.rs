use crate::client::paragraph_style::ParagraphStyle;
use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::report_fumbbl_result_upload::ReportFumbblResultUpload;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `FumbblResultUploadMessage.java`.
pub struct FumbblResultUploadMessage;

impl ReportMessage for FumbblResultUploadMessage {
    type Report = ReportFumbblResultUpload;

    fn report_id(&self) -> ReportId {
        ReportId::FUMBBL_RESULT_UPLOAD
    }

    fn render(&self, status_report: &mut StatusReport, _game: &Game, report: &Self::Report) {
        let status = if report.is_successful() {
            "Fumbbl Result Upload ok".to_string()
        } else {
            "Fumbbl Result Upload failed".to_string()
        };
        status_report.println_style(Some(ParagraphStyle::SPACE_ABOVE_BELOW), Some(TextStyle::BOLD), &status);
        status_report.println_indent(status_report.get_indent() + 1, report.get_upload_status());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_id_matches() {
        assert_eq!(FumbblResultUploadMessage.report_id(), ReportId::FUMBBL_RESULT_UPLOAD);
    }

    #[test]
    fn successful_upload_reports_ok() {
        let mut status_report = StatusReport::new();
        let game = ffb_model::model::game::Game::new(
            ffb_model::model::team::Team {
                id: "home".into(), name: "home".into(), race: "human".into(), roster_id: "human".into(),
                coach: "coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
                prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
                assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
                special_rules: Vec::new(), players: Vec::new(), vampire_lord: false, necromancer: false,
            },
            ffb_model::model::team::Team {
                id: "away".into(), name: "away".into(), race: "human".into(), roster_id: "human".into(),
                coach: "coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
                prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
                assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
                special_rules: Vec::new(), players: Vec::new(), vampire_lord: false, necromancer: false,
            },
            ffb_model::enums::Rules::Bb2025,
        );
        let report = ReportFumbblResultUpload::new(true, "Upload complete".into());
        FumbblResultUploadMessage.render(&mut status_report, &game, &report);

        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Fumbbl Result Upload ok"));
        assert_eq!(status_report.rendered_runs[0].paragraph_style, Some(ParagraphStyle::SPACE_ABOVE_BELOW));
        assert_eq!(status_report.rendered_runs[0].text_style, Some(TextStyle::BOLD));
    }

    #[test]
    fn failed_upload_reports_failed_and_status_line() {
        let mut status_report = StatusReport::new();
        let game = make_minimal_game();
        let report = ReportFumbblResultUpload::new(false, "Connection error".into());
        FumbblResultUploadMessage.render(&mut status_report, &game, &report);

        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.contains(&"Fumbbl Result Upload failed"));
        assert!(texts.contains(&"Connection error"));
    }

    #[test]
    fn upload_status_uses_indent_plus_one_paragraph_style() {
        let mut status_report = StatusReport::new();
        let game = make_minimal_game();
        let report = ReportFumbblResultUpload::new(true, "OK".into());
        FumbblResultUploadMessage.render(&mut status_report, &game, &report);

        // second println run is the upload status line
        assert_eq!(status_report.rendered_runs[2].paragraph_style, Some(ParagraphStyle::INDENT_1));
        assert_eq!(status_report.rendered_runs[2].text.as_deref(), Some("OK"));
    }

    fn make_minimal_game() -> Game {
        Game::new(
            ffb_model::model::team::Team {
                id: "home".into(), name: "home".into(), race: "human".into(), roster_id: "human".into(),
                coach: "coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
                prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
                assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
                special_rules: Vec::new(), players: Vec::new(), vampire_lord: false, necromancer: false,
            },
            ffb_model::model::team::Team {
                id: "away".into(), name: "away".into(), race: "human".into(), roster_id: "human".into(),
                coach: "coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
                prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
                assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
                special_rules: Vec::new(), players: Vec::new(), vampire_lord: false, necromancer: false,
            },
            ffb_model::enums::Rules::Bb2025,
        )
    }
}
