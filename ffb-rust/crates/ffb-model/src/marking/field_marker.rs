use crate::types::FieldCoordinate;

/// 1:1 translation of `com.fumbbl.ffb.marking.FieldMarker`.
#[derive(Debug, Clone, Default)]
pub struct FieldMarker {
    /// Java: coordinate
    pub coordinate: Option<FieldCoordinate>,
    /// Java: homeText
    pub home_text: Option<String>,
    /// Java: awayText
    pub away_text: Option<String>,
}

impl FieldMarker {
    pub fn new() -> Self { Self::default() }

    pub fn with_coordinate(coordinate: FieldCoordinate) -> Self {
        Self { coordinate: Some(coordinate), ..Default::default() }
    }

    pub fn with_all(coordinate: FieldCoordinate, home_text: impl Into<String>, away_text: impl Into<String>) -> Self {
        Self {
            coordinate: Some(coordinate),
            home_text: Some(home_text.into()),
            away_text: Some(away_text.into()),
        }
    }

    pub fn get_coordinate(&self) -> Option<&FieldCoordinate> { self.coordinate.as_ref() }
    pub fn set_home_text(&mut self, text: impl Into<String>) { self.home_text = Some(text.into()); }
    pub fn get_home_text(&self) -> Option<&str> { self.home_text.as_deref() }
    pub fn set_away_text(&mut self, text: impl Into<String>) { self.away_text = Some(text.into()); }
    pub fn get_away_text(&self) -> Option<&str> { self.away_text.as_deref() }

    /// Java: `transform()` — swap home/away texts and transform coordinate.
    pub fn transform(&self) -> FieldMarker {
        FieldMarker {
            coordinate: self.coordinate.as_ref().map(|c| c.transform()),
            home_text: self.away_text.clone(),
            away_text: self.home_text.clone(),
        }
    }

    /// Java: `static transform(FieldMarker)`.
    pub fn transform_opt(marker: Option<&FieldMarker>) -> Option<FieldMarker> {
        marker.map(|m| m.transform())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transform_swaps_texts() {
        let m = FieldMarker::with_all(FieldCoordinate::new(3, 4), "Home", "Away");
        let t = m.transform();
        assert_eq!(t.get_home_text(), Some("Away"));
        assert_eq!(t.get_away_text(), Some("Home"));
    }

    #[test]
    fn transform_opt_none_returns_none() {
        assert!(FieldMarker::transform_opt(None).is_none());
    }
}
