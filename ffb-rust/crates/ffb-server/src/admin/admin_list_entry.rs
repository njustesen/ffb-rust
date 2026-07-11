/// 1:1 translation of com.fumbbl.ffb.server.admin.AdminListEntry.
pub struct AdminListEntry {
    pub game_id: i64,
    pub started: Option<i64>,
    pub finished: Option<i64>,
    pub last_updated: Option<i64>,
    pub half: i32,
    pub turn: i32,
    pub status: String,
    pub team_home_id: String,
    pub team_home_name: String,
    pub team_home_coach: String,
    pub team_away_id: String,
    pub team_away_name: String,
    pub team_away_coach: String,
    pub swapped_out: bool,
    pub test_mode: bool,
}

impl AdminListEntry {
    pub const XML_TAG: &'static str = "game";

    pub fn new(game_id: i64) -> Self {
        Self {
            game_id,
            started: None,
            finished: None,
            last_updated: None,
            half: 0,
            turn: 0,
            status: String::new(),
            team_home_id: String::new(),
            team_home_name: String::new(),
            team_home_coach: String::new(),
            team_away_id: String::new(),
            team_away_name: String::new(),
            team_away_coach: String::new(),
            swapped_out: false,
            test_mode: false,
        }
    }

    pub fn get_game_id(&self) -> i64 {
        self.game_id
    }

    /// Port of `AdminListEntry.addToXml`. Java formats `started`/`finished`/`lastUpdated` as
    /// `yyyy-MM-dd'T'HH:mm:ss.SSS` timestamps; since this struct stores them as raw epoch millis
    /// (no `Date`/timestamp-formatting infra exists here yet), the millis are emitted verbatim.
    pub fn to_xml(&self) -> String {
        fn attr(name: &str, value: &str) -> String {
            if value.is_empty() {
                String::new()
            } else {
                format!(" {}=\"{}\"", name, value)
            }
        }
        fn opt_attr(name: &str, value: Option<i64>) -> String {
            match value {
                Some(v) => format!(" {}=\"{}\"", name, v),
                None => String::new(),
            }
        }

        let mut xml = format!(
            "<{tag}{id}{started}{finished}{last_updated}{half}{turn}{status}{swapped}{test}>",
            tag = Self::XML_TAG,
            id = attr("id", &self.game_id.to_string()),
            started = opt_attr("started", self.started),
            finished = opt_attr("finished", self.finished),
            last_updated = opt_attr("lastUpdated", self.last_updated),
            half = attr("half", &self.half.to_string()),
            turn = attr("turn", &self.turn.to_string()),
            status = attr("status", &self.status),
            swapped = attr("swappedOut", &self.swapped_out.to_string()),
            test = attr("testMode", &self.test_mode.to_string()),
        );

        xml.push_str(&format!(
            "<team{id}{home}{name}{coach}/>",
            id = attr("id", &self.team_home_id),
            home = attr("home", "true"),
            name = attr("name", &self.team_home_name),
            coach = attr("coach", &self.team_home_coach),
        ));
        xml.push_str(&format!(
            "<team{id}{home}{name}{coach}/>",
            id = attr("id", &self.team_away_id),
            home = attr("home", "false"),
            name = attr("name", &self.team_away_name),
            coach = attr("coach", &self.team_away_coach),
        ));

        xml.push_str(&format!("</{}>", Self::XML_TAG));
        xml
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let e = AdminListEntry::new(1);
        assert_eq!(e.get_game_id(), 1);
        assert_eq!(AdminListEntry::XML_TAG, "game");
    }

    #[test]
    fn to_xml_contains_game_and_team_attributes() {
        let mut e = AdminListEntry::new(7);
        e.half = 2;
        e.turn = 5;
        e.status = "ACTIVE".to_string();
        e.team_home_name = "Reavers".to_string();
        e.team_away_name = "Bombers".to_string();
        let xml = e.to_xml();
        assert!(xml.starts_with("<game"));
        assert!(xml.ends_with("</game>"));
        assert!(xml.contains("id=\"7\""));
        assert!(xml.contains("half=\"2\""));
        assert!(xml.contains("status=\"ACTIVE\""));
        assert!(xml.contains("home=\"true\""));
        assert!(xml.contains("name=\"Reavers\""));
        assert!(xml.contains("name=\"Bombers\""));
    }

    #[test]
    fn to_xml_omits_absent_timestamps() {
        let e = AdminListEntry::new(1);
        let xml = e.to_xml();
        assert!(!xml.contains("started="));
        assert!(!xml.contains("finished="));
    }
}
