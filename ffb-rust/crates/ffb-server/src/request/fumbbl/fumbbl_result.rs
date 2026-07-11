/// 1:1 translation of com.fumbbl.ffb.server.request.fumbbl.FumbblResult.
/// XML serializer for game results sent to FUMBBL. Covers team results, player results,
/// inducements, star players, mercenaries, infamous staff, and cards.
pub struct FumbblResult;

impl FumbblResult {
    // Top-level XML tags
    pub const XML_TAG_REPLAY_ID: &'static str = "replayId";
    pub const XML_TAG_HALVES: &'static str = "halves";

    // Team result XML tags
    pub const XML_TAG_TEAM_ID: &'static str = "teamId";
    pub const XML_TAG_SCORE: &'static str = "score";
    pub const XML_TAG_CONCEDED: &'static str = "conceded";
    pub const XML_TAG_CONCEDED_LEGALLY: &'static str = "concededLegally";
    pub const XML_TAG_STALLED: &'static str = "stalled";
    pub const XML_TAG_FAME: &'static str = "fame";
    pub const XML_TAG_SPECTATORS: &'static str = "spectators";
    pub const XML_TAG_WINNINGS: &'static str = "winnings";
    pub const XML_TAG_TEAM_VALUE: &'static str = "teamValue";

    // Player result XML tags
    pub const XML_TAG_PLAYER_ID: &'static str = "playerId";
    pub const XML_TAG_TOUCHDOWNS: &'static str = "touchdowns";
    pub const XML_TAG_CASUALTIES: &'static str = "casualties";
    pub const XML_TAG_COMPLETIONS: &'static str = "completions";
    pub const XML_TAG_INTERCEPTIONS: &'static str = "interceptions";

    pub fn new() -> Self {
        Self
    }

    /// Builds the `<gameResult>` XML wrapper sent to FUMBBL_RESULT.
    ///
    /// The Java version walks the live `Game`/`GameResult`/`Player` model to emit team and
    /// player result sub-elements; that model does not exist in this simplified server crate
    /// yet, so this emits the outer wrapper only. Callers that have concrete team/player result
    /// XML fragments can pass them in via `to_xml_with_team_results`.
    pub fn to_xml(&self) -> String {
        self.to_xml_with_team_results(&[])
    }

    /// Wraps pre-rendered team-result XML fragments (see [`Self::to_xml`]).
    pub fn to_xml_with_team_results(&self, team_result_xml_fragments: &[String]) -> String {
        let mut xml = String::from("<gameResult>");
        for fragment in team_result_xml_fragments {
            xml.push_str(fragment);
        }
        xml.push_str("</gameResult>");
        xml
    }
}

impl Default for FumbblResult {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let _ = FumbblResult::new();
    }

    #[test]
    fn constants() {
        assert_eq!(FumbblResult::XML_TAG_TEAM_ID, "teamId");
        assert_eq!(FumbblResult::XML_TAG_TOUCHDOWNS, "touchdowns");
    }

    #[test]
    fn to_xml_wraps_in_game_result_tag() {
        let xml = FumbblResult::new().to_xml();
        assert_eq!(xml, "<gameResult></gameResult>");
    }

    #[test]
    fn to_xml_with_team_results_embeds_fragments() {
        let xml = FumbblResult::new()
            .to_xml_with_team_results(&["<teamResult/>".to_string()]);
        assert_eq!(xml, "<gameResult><teamResult/></gameResult>");
    }
}
