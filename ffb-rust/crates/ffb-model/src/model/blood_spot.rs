use serde::{Deserialize, Serialize};
use crate::enums::PlayerState;
use crate::types::FieldCoordinate;

/// 1:1 translation of com.fumbbl.ffb.BloodSpot.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BloodSpot {
    pub injury: Option<PlayerState>,
    pub coordinate: Option<FieldCoordinate>,
}

impl BloodSpot {
    pub fn new(coordinate: FieldCoordinate, injury: PlayerState) -> Self {
        BloodSpot { coordinate: Some(coordinate), injury: Some(injury) }
    }

    pub fn get_injury(&self) -> Option<PlayerState> { self.injury }
    pub fn get_coordinate(&self) -> Option<FieldCoordinate> { self.coordinate }

    /// Java: `BloodSpot.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        use crate::util::java_json as j;
        let mut o = j::JsonObject::new();
        o.insert("injury".into(), j::player_state_or_null(self.injury.map(|s| s.0 as i32)));
        o.insert("coordinate".into(), j::field_coordinate_or_null(self.coordinate));
        serde_json::Value::Object(o)
    }

    /// Java: `new BloodSpot().initFrom(source, jsonValue)`.
    pub fn init_from(json: &serde_json::Value) -> Result<Self, String> {
        use crate::util::java_json as j;
        let o = j::to_json_object(json)?;
        Ok(BloodSpot { injury: j::get_player_state(o, "injury")?, coordinate: j::get_field_coordinate(o, "coordinate")? })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn serde_round_trip_default() {
        let b = BloodSpot::default();
        let s = serde_json::to_string(&b).unwrap();
        let back: BloodSpot = serde_json::from_str(&s).unwrap();
        assert!(back.injury.is_none());
        assert!(back.coordinate.is_none());
    }

}
