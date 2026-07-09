/// 1:1 translation of `com.fumbbl.ffb.bb2020.InjuryDescription`.
#[derive(Debug, Clone, Default)]
pub struct InjuryDescription {
    /// Java: playerId
    pub player_id: Option<String>,
    /// Java: playerState (encoded as an integer key in Java)
    pub player_state: Option<i32>,
    /// Java: seriousInjury (encoded as a String key in Java)
    pub serious_injury: Option<String>,
    /// Java: apothecaryTypes (list of ApothecaryType names)
    pub apothecary_types: Vec<String>,
}

impl InjuryDescription {
    pub fn new() -> Self { Self::default() }

    pub fn with_all(
        player_id: impl Into<String>,
        player_state: i32,
        serious_injury: Option<String>,
        apothecary_types: Vec<String>,
    ) -> Self {
        Self {
            player_id: Some(player_id.into()),
            player_state: Some(player_state),
            serious_injury,
            apothecary_types,
        }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_player_state(&self) -> Option<i32> { self.player_state }
    pub fn get_serious_injury(&self) -> Option<&str> { self.serious_injury.as_deref() }
    pub fn get_apothecary_types(&self) -> &[String] { &self.apothecary_types }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_has_empty_fields() {
        let d = InjuryDescription::new();
        assert!(d.get_player_id().is_none());
        assert!(d.get_apothecary_types().is_empty());
    }

    #[test]
    fn with_all_sets_all_fields() {
        let d = InjuryDescription::with_all("p1", 2, Some("BADLY_HURT".into()), vec!["NORMAL".into()]);
        assert_eq!(d.get_player_id(), Some("p1"));
        assert_eq!(d.get_player_state(), Some(2));
        assert_eq!(d.get_serious_injury(), Some("BADLY_HURT"));
        assert_eq!(d.get_apothecary_types().len(), 1);
    }
}
