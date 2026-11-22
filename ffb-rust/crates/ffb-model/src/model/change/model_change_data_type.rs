// Re-exports the already-implemented type for 1:1 file mapping.
pub use crate::enums::ModelChangeDataType;

use crate::dialog::dialog_parameter_factory::{DialogParameter, DialogParameterFactory};
use crate::enums::{LeaderState, PlayerAction, PlayerState, Rules, SeriousInjuryKind, SkillId, TurnMode, Weather};
use crate::enums::SendToBoxReason;
use crate::inducement::inducement::Inducement;
use crate::marking::field_marker::FieldMarker;
use crate::marking::player_marker::PlayerMarker;
use crate::model::blood_spot::BloodSpot;
use crate::model::dice_decoration::DiceDecoration;
use crate::model::game_options::GameOptions;
use crate::model::target_selection_state::TargetSelectionState;
use crate::model::track_number::TrackNumber;
use crate::types::{FieldCoordinate, MoveSquare, PushbackSquare, RangeRuler};
use crate::util::java_json as j;
use serde_json::Value;

/// The Java `Object` a `ModelChange` carries, as `ModelChangeDataType.fromJsonValue` builds it.
/// `Null` is every Java null: a JSON null value, and an enum name the factory does not know
/// (`UtilJson.toEnumWithName` -> `forName` null).
#[derive(Debug, Clone)]
pub enum ModelChangeValue {
    Null,
    Boolean(bool),
    String(String),
    PlayerAction(PlayerAction),
    Skill(SkillId),
    Long(i64),
    /// The date's string, as the Rust `Game` keeps it (see `java_json::get_date`).
    Date(String),
    TurnMode(TurnMode),
    FieldCoordinate(FieldCoordinate),
    DialogParameter(Box<DialogParameter>),
    Integer(i32),
    PlayerState(PlayerState),
    SeriousInjury(SeriousInjuryKind),
    SendToBoxReason(SendToBoxReason),
    BloodSpot(BloodSpot),
    TrackNumber(TrackNumber),
    PushbackSquare(PushbackSquare),
    MoveSquare(MoveSquare),
    Weather(Weather),
    RangeRuler(RangeRuler),
    DiceDecoration(DiceDecoration),
    Inducement(Inducement),
    FieldMarker(FieldMarker),
    PlayerMarker(PlayerMarker),
    LeaderState(LeaderState),
    TargetSelectionState(Box<TargetSelectionState>),
    /// The prayer's ENUM-constant name (the form the Rust `InducementSet` keeps).
    Prayer(&'static str),
    /// A trap door = its coordinate (the Rust `FieldModel` keeps trap doors as coordinates).
    TrapDoor(FieldCoordinate),
}

impl ModelChangeValue {
    pub fn is_null(&self) -> bool {
        matches!(self, ModelChangeValue::Null)
    }
}

fn as_int(value: &Value) -> Result<i32, String> {
    value
        .as_i64()
        .and_then(|v| i32::try_from(v).ok())
        .ok_or_else(|| format!("asInt on {value}"))
}

fn as_str(value: &Value) -> Result<&str, String> {
    value.as_str().ok_or_else(|| format!("asString on {value}"))
}

impl ModelChangeDataType {
    /// Java: `ModelChangeDataType.fromJsonValue(source, jsonValue)` for a game of `rules` with
    /// `options` (the factory source). A JSON null (or absent value) is `Null` for every type.
    /// `Err` where Java throws (`asBoolean`/`asInt`/`asString` on the wrong type, a broken nested
    /// object) and for the types the Rust model cannot hold yet (cards, card effects, card
    /// choices, game options, dialog ids, sketch states -- none occur in the replay corpus).
    pub fn from_json_value(self, rules: Rules, options: &GameOptions, json: &Value) -> Result<ModelChangeValue, String> {
        use ModelChangeDataType as T;
        use ModelChangeValue as V;
        if json.is_null() {
            return Ok(V::Null);
        }
        let enum_name = || as_str(json);
        Ok(match self {
            T::BlitzState | T::TargetSelectionState => {
                V::TargetSelectionState(Box::new(TargetSelectionState::init_from(rules, json)?))
            }
            T::BloodSpot => V::BloodSpot(BloodSpot::init_from(json)?),
            T::Boolean => V::Boolean(json.as_bool().ok_or_else(|| format!("asBoolean on {json}"))?),
            T::DialogParameter => match DialogParameterFactory.for_json_value(rules, json)? {
                Some(parameter) => V::DialogParameter(Box::new(parameter)),
                None => V::Null,
            },
            T::Date => {
                let mut o = j::JsonObject::new();
                o.insert("date".into(), json.clone());
                j::get_date(&o, "date")?.map_or(V::Null, V::Date)
            }
            T::DiceDecoration => V::DiceDecoration(DiceDecoration::init_from(json)?),
            T::FieldCoordinate => {
                let mut o = j::JsonObject::new();
                o.insert("c".into(), json.clone());
                j::get_field_coordinate(&o, "c")?.map_or(V::Null, V::FieldCoordinate)
            }
            T::FieldMarker => V::FieldMarker(FieldMarker::init_from(json)?),
            T::Inducement => {
                let mut inducement = Inducement::new("", 0, Vec::new());
                inducement.init_from(rules, json)?;
                V::Inducement(inducement)
            }
            T::Integer => V::Integer(as_int(json)?),
            // LeaderStateFactory.forName: equalsIgnoreCase over the names.
            T::LeaderState => {
                let name = enum_name()?;
                [LeaderState::None, LeaderState::Available, LeaderState::Used]
                    .into_iter()
                    .find(|s| j::java_equals_ignore_case(s.name(), name))
                    .map_or(V::Null, V::LeaderState)
            }
            T::Long => V::Long(json.as_i64().ok_or_else(|| format!("asLong on {json}"))?),
            T::MoveSquare => V::MoveSquare(MoveSquare::init_from(json)?),
            T::Null => V::Null,
            T::PlayerAction => PlayerAction::from_name(enum_name()?).map_or(V::Null, V::PlayerAction),
            T::PlayerMarker => V::PlayerMarker(PlayerMarker::init_from(json)?),
            T::PlayerState => {
                let id = as_int(json)?;
                V::PlayerState(PlayerState(u32::try_from(id).map_err(|_| format!("player state {id}"))?))
            }
            T::Prayer => crate::factory::prayer_factory::for_name_in(rules, options, enum_name()?)?.map_or(V::Null, V::Prayer),
            T::PushbackSquare => V::PushbackSquare(PushbackSquare::init_from(json)?),
            T::RangeRuler => V::RangeRuler(RangeRuler::init_from(json)?),
            T::SendToBoxReason => SendToBoxReason::from_name(enum_name()?).map_or(V::Null, V::SendToBoxReason),
            T::SeriousInjury => crate::factory::serious_injury_factory::for_name_in(rules, enum_name()?)
                .map_or(V::Null, |si| V::SeriousInjury(si.to_kind())),
            T::Skill => crate::factory::skill_factory::SkillFactory::for_name_in(rules, enum_name()?).map_or(V::Null, V::Skill),
            T::String => V::String(as_str(json)?.to_string()),
            T::TrackNumber => V::TrackNumber(TrackNumber::init_from(json)?),
            T::TrapDoor => {
                let o = j::to_json_object(json)?;
                j::get_field_coordinate(o, "coordinate")?.map_or_else(
                    || Err("TrapDoor: null coordinate (not representable)".to_string()),
                    |c| Ok(V::TrapDoor(c)),
                )?
            }
            // TurnModeFactory.forName: equalsIgnoreCase over values().
            T::TurnMode => {
                let name = enum_name()?;
                TurnMode::all()
                    .iter()
                    .copied()
                    .find(|m| j::java_equals_ignore_case(m.name(), name))
                    .map_or(V::Null, V::TurnMode)
            }
            T::Weather => Weather::for_name_ignore_case(enum_name()?).map_or(V::Null, V::Weather),
            T::Card | T::CardEffect | T::CardChoices | T::GameOption | T::DialogId | T::SketchState => {
                return Err(format!("ModelChangeDataType {}: decoding not ported yet ({json})", self.name()));
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn decode(t: ModelChangeDataType, v: Value) -> Result<ModelChangeValue, String> {
        t.from_json_value(Rules::Bb2020, &GameOptions::default(), &v)
    }

    #[test]
    fn null_is_null_for_every_type() {
        assert!(matches!(decode(ModelChangeDataType::Boolean, Value::Null).unwrap(), ModelChangeValue::Null));
        assert!(matches!(decode(ModelChangeDataType::Card, Value::Null).unwrap(), ModelChangeValue::Null));
    }

    #[test]
    fn scalars_and_enum_names() {
        assert!(matches!(decode(ModelChangeDataType::Boolean, json!(true)).unwrap(), ModelChangeValue::Boolean(true)));
        assert!(decode(ModelChangeDataType::Boolean, json!(1)).is_err());
        assert!(matches!(decode(ModelChangeDataType::Integer, json!(7)).unwrap(), ModelChangeValue::Integer(7)));
        assert!(matches!(decode(ModelChangeDataType::Long, json!(1965890)).unwrap(), ModelChangeValue::Long(1965890)));
        assert!(matches!(decode(ModelChangeDataType::FieldCoordinate, json!([3, 4])).unwrap(), ModelChangeValue::FieldCoordinate(c) if c == FieldCoordinate::new(3, 4)));
        assert!(matches!(decode(ModelChangeDataType::PlayerState, json!(257)).unwrap(), ModelChangeValue::PlayerState(PlayerState(257))));
        assert!(matches!(decode(ModelChangeDataType::TurnMode, json!("REGULAR")).unwrap(), ModelChangeValue::TurnMode(TurnMode::Regular)));
        // a newer-server turn mode: forName null
        assert!(matches!(decode(ModelChangeDataType::TurnMode, json!("dwarfenWisdom")).unwrap(), ModelChangeValue::Null));
        assert!(matches!(decode(ModelChangeDataType::LeaderState, json!("used")).unwrap(), ModelChangeValue::LeaderState(LeaderState::Used)));
        assert!(matches!(decode(ModelChangeDataType::Skill, json!("Block")).unwrap(), ModelChangeValue::Skill(SkillId::Block)));
        assert!(matches!(decode(ModelChangeDataType::Skill, json!("No Such Skill")).unwrap(), ModelChangeValue::Null));
        assert!(matches!(decode(ModelChangeDataType::Date, json!("2026-11-01T10:11:12.345")).unwrap(), ModelChangeValue::Date(d) if d == "2026-11-01T10:11:12.345"));
        assert!(decode(ModelChangeDataType::GameOption, json!({})).is_err());
    }
}
