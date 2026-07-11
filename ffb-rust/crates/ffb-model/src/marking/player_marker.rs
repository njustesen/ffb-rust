/// 1:1 translation of `com.fumbbl.ffb.marking.PlayerMarker`.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PlayerMarker {
    /// Java: fPlayerId
    pub player_id: Option<String>,
    /// Java: fHomeText
    pub home_text: Option<String>,
    /// Java: fAwayText
    pub away_text: Option<String>,
}

impl PlayerMarker {
    pub fn new() -> Self { Self::default() }

    pub fn with_player_id(player_id: impl Into<String>) -> Self {
        Self { player_id: Some(player_id.into()), ..Default::default() }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn set_home_text(&mut self, text: impl Into<String>) { self.home_text = Some(text.into()); }
    pub fn get_home_text(&self) -> Option<&str> { self.home_text.as_deref() }
    pub fn set_away_text(&mut self, text: impl Into<String>) { self.away_text = Some(text.into()); }
    pub fn get_away_text(&self) -> Option<&str> { self.away_text.as_deref() }

    /// Java: `transform()` — swap home/away texts.
    pub fn transform(&self) -> PlayerMarker {
        PlayerMarker {
            player_id: self.player_id.clone(),
            home_text: self.away_text.clone(),
            away_text: self.home_text.clone(),
        }
    }

    /// Java: `PlayerMarker.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "playerId": self.player_id,
            "homeText": self.home_text,
            "awayText": self.away_text,
        })
    }

    /// Java: `PlayerMarker.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            player_id: json.get("playerId").and_then(|v| v.as_str()).map(str::to_string),
            home_text: json.get("homeText").and_then(|v| v.as_str()).map(str::to_string),
            away_text: json.get("awayText").and_then(|v| v.as_str()).map(str::to_string),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_player_id_sets_id() {
        let m = PlayerMarker::with_player_id("p1");
        assert_eq!(m.get_player_id(), Some("p1"));
    }

    #[test]
    fn transform_swaps_texts() {
        let mut m = PlayerMarker::with_player_id("p1");
        m.set_home_text("Home");
        m.set_away_text("Away");
        let t = m.transform();
        assert_eq!(t.get_home_text(), Some("Away"));
        assert_eq!(t.get_away_text(), Some("Home"));
    }
}

