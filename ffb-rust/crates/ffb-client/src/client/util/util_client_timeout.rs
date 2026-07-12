//! 1:1 translation of `com.fumbbl.ffb.client.util.UtilClientTimeout`.
//!
//! Java routes everything through `pClient.getUserInterface().getStatusReport()` (Swing-owned).
//! `StatusReport` was made a real, headless struct in Phase ZW.2 Batch C (see its module doc:
//! "`FantasyFootballClient`/`UserInterface` are not wired through here (headless)"), and every
//! already-translated call site (e.g. `client/report/*_message.rs` renderers) reaches it by
//! taking `&mut StatusReport` as an explicit parameter rather than going through
//! `getUserInterface()` — this file follows that same established convention.

use ffb_model::model::StatusType;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::status_report::StatusReport;

/// 1:1 translation of `UtilClientTimeout`.
pub struct UtilClientTimeout;

impl UtilClientTimeout {
    /// Java: `public static void showTimeoutStatus(FantasyFootballClient pClient)`. `status_report`
    /// stands in for `pClient.getUserInterface().getStatusReport()` (see module doc).
    pub fn show_timeout_status(client: &mut FantasyFootballClient, status_report: &mut StatusReport) {
        let timeout = match client.game() {
            Some(game) => Some((game.timeout_possible, game.home_playing)),
            None => None,
        };
        if let Some((timeout_possible, home_playing)) = timeout {
            if timeout_possible {
                if home_playing {
                    client.client_data_mut().set_status(
                        Some("Timeout Possible".to_string()),
                        Some("Coach may force a Timeout on his/her opponent.".to_string()),
                        Some(StatusType::REF),
                    );
                }
                status_report.report_timeout();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::{ClientMode, Game};

    fn make_team(id: &str) -> ffb_model::model::team::Team {
        ffb_model::model::team::Team {
            id: id.into(),
            name: "Team".into(),
            race: "Human".into(),
            roster_id: "human".into(),
            coach: "Coach".into(),
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
            players: vec![],
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn client_with_game(timeout_possible: bool, home_playing: bool) -> FantasyFootballClient {
        let args = vec!["-spectator".to_string(), "-coach".to_string(), "bob".to_string()];
        let parameters = crate::client::client_parameters::ClientParameters::create_valid_params(&args).unwrap();
        let mut client = FantasyFootballClient::new(parameters);
        let mut game = Game::new(make_team("home"), make_team("away"), ffb_model::enums::Rules::Bb2025);
        game.timeout_possible = timeout_possible;
        game.home_playing = home_playing;
        client.set_game(game);
        client
    }

    #[test]
    fn no_game_is_a_no_op() {
        let args = vec!["-spectator".to_string(), "-coach".to_string(), "bob".to_string()];
        let parameters = crate::client::client_parameters::ClientParameters::create_valid_params(&args).unwrap();
        let mut client = FantasyFootballClient::new(parameters);
        let mut status_report = StatusReport::new();
        UtilClientTimeout::show_timeout_status(&mut client, &mut status_report);
        assert!(status_report.rendered_runs.is_empty());
    }

    #[test]
    fn timeout_not_possible_reports_nothing() {
        let mut client = client_with_game(false, true);
        let mut status_report = StatusReport::new();
        UtilClientTimeout::show_timeout_status(&mut client, &mut status_report);
        assert!(status_report.rendered_runs.is_empty());
        assert!(client.client_data().status_message().is_none());
    }

    #[test]
    fn timeout_possible_and_home_playing_sets_status_and_reports() {
        let mut client = client_with_game(true, true);
        let mut status_report = StatusReport::new();
        UtilClientTimeout::show_timeout_status(&mut client, &mut status_report);
        assert_eq!(client.client_data().status_title(), Some("Timeout Possible"));
        assert_eq!(
            client.client_data().status_message(),
            Some("Coach may force a Timeout on his/her opponent.")
        );
        assert!(!status_report.rendered_runs.is_empty());
    }

    #[test]
    fn timeout_possible_but_not_home_playing_only_reports() {
        let mut client = client_with_game(true, false);
        let mut status_report = StatusReport::new();
        UtilClientTimeout::show_timeout_status(&mut client, &mut status_report);
        assert!(client.client_data().status_message().is_none());
        assert!(!status_report.rendered_runs.is_empty());
    }

    #[test]
    fn client_mode_unaffected() {
        let mut client = client_with_game(true, true);
        client.set_mode(ClientMode::SPECTATOR);
        let mut status_report = StatusReport::new();
        UtilClientTimeout::show_timeout_status(&mut client, &mut status_report);
        assert_eq!(client.mode(), Some(ClientMode::SPECTATOR));
    }
}
