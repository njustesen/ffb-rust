use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogUseInducementParameter.
/// `InducementType[]` / `Card[]` are kept as canonical names; `None` = a Java null element (an
/// unknown name decodes to null, and toJsonValue then throws).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogUseInducementParameter {
    pub team_id: Option<String>,
    /// InducementType[] as canonical names (`None` = null element).
    pub inducement_types: Vec<Option<String>>,
    /// Card[] as canonical names (`None` = null element).
    pub cards: Vec<Option<String>>,
    pub player_id: Option<String>,
}

impl DialogUseInducementParameter {
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_inducement_types(&self) -> &[Option<String>] { &self.inducement_types }
    pub fn get_cards(&self) -> &[Option<String>] { &self.cards }
    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
}

impl IDialogParameter for DialogUseInducementParameter {
    fn get_id(&self) -> DialogId { DialogId::USE_INDUCEMENT }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

// JSON serialization (BACKLOG §O).
impl DialogUseInducementParameter {
    /// Java `initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, rules: crate::enums::Rules, json: &serde_json::Value) -> Result<(), String> {
        use crate::util::java_json as jj;
        let obj = jj::to_json_object(json)?;
        super::util_dialog_parameter::validate_dialog_id_json(DialogId::USE_INDUCEMENT, obj)?;
        self.team_id = jj::get_string(obj, "teamId")?;
        // `inducementTypeNames.length` with no null check: an absent array -> NullPointerException.
        let inducement_type_names = jj::get_string_array(obj, "inducementTypeArray")?
            .ok_or_else(|| "inducementTypeArray: null (NullPointerException)".to_string())?;
        // InducementTypeFactory.forName (per ruleset, equalsIgnoreCase); null or unknown -> null.
        self.inducement_types = inducement_type_names
            .iter()
            .map(|name| name.as_deref().and_then(|n| crate::factory::inducement_type_factory::for_name_in(rules, n)).map(|t| t.id))
            .collect();
        let card_names = jj::get_string_array(obj, "cards")?
            .ok_or_else(|| "cards: null (NullPointerException)".to_string())?;
        // CardFactory.forName: a null name matches nothing, but the lookup still walks the
        // ruleset's cards (BB2025 has none -> NullPointerException); "" emulates that.
        self.cards = card_names
            .iter()
            .map(|name| match name {
                Some(n) => crate::factory::card_factory::for_name_in(rules, n),
                None => crate::factory::card_factory::for_name_in(rules, "").map(|_| None),
            })
            .collect::<Result<Vec<_>, _>>()?;
        if jj::is_defined_in(obj, "playerId") {
            self.player_id = jj::get_string(obj, "playerId")?;
        }
        Ok(())
    }

    /// Java `toJsonValue()`.
    pub fn to_json_value(&self, _rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        use crate::util::java_json as jj;
        use serde_json::Value;
        let mut o = serde_json::Map::new();
        o.insert("dialogId".to_string(), Value::String(DialogId::USE_INDUCEMENT.get_name().to_string()));
        o.insert("teamId".to_string(), jj::string_or_null(self.team_id.as_deref()));
        // `inducementType.getName()` / `card.getName()`: a null element -> NullPointerException.
        let inducement_type_names = self
            .inducement_types
            .iter()
            .map(|t| t.clone().ok_or_else(|| "inducementTypeArray: null type (NullPointerException)".to_string()))
            .collect::<Result<Vec<String>, String>>()?;
        o.insert("inducementTypeArray".to_string(), jj::string_array(&inducement_type_names));
        let card_names = self
            .cards
            .iter()
            .map(|c| c.clone().ok_or_else(|| "cards: null card (NullPointerException)".to_string()))
            .collect::<Result<Vec<String>, String>>()?;
        if jj::is_provided(self.player_id.as_deref()) {
            o.insert("playerId".to_string(), jj::string_or_null(self.player_id.as_deref()));
        }
        o.insert("cards".to_string(), jj::string_array(&card_names));
        Ok(Value::Object(o))
    }
}

#[cfg(test)]
mod json_tests {
    use crate::dialog::dialog_parameter_factory::DialogParameterFactory;
    use crate::enums::Rules;
    use serde_json::{json, Value};

    fn round_trip(rules: Rules, input: Value) -> Result<Value, String> {
        let p = DialogParameterFactory::new().for_json_value(rules, &input)?.expect("parameter");
        p.to_json_value(rules)
    }

    #[test]
    fn json_round_trip_case() {
        assert_eq!(round_trip(Rules::Bb2025, json!({"cards": [], "dialogId": "useInducement", "inducementTypeArray": ["throwARock"], "teamId": "t1"})).unwrap(), json!({"cards": [], "dialogId": "useInducement", "inducementTypeArray": ["throwARock"], "teamId": "t1"}));
    }

    #[test]
    fn names_are_canonical_and_player_id_only_when_provided() {
        assert_eq!(round_trip(Rules::Bb2016, json!({"cards": ["pit trap"], "dialogId": "useInducement", "inducementTypeArray": ["WIZARD"], "teamId": "t1", "playerId": ""})).unwrap(), json!({"cards": ["Pit Trap"], "dialogId": "useInducement", "inducementTypeArray": ["wizard"], "teamId": "t1"}));
    }

    #[test]
    fn player_id_written_when_provided() {
        assert_eq!(round_trip(Rules::Bb2025, json!({"cards": [], "dialogId": "useInducement", "inducementTypeArray": ["wizard"], "playerId": "p1"})).unwrap(), json!({"cards": [], "dialogId": "useInducement", "inducementTypeArray": ["wizard"], "teamId": null, "playerId": "p1"}));
    }

    #[test]
    fn unknown_type_throws_on_encode() {
        assert!(round_trip(Rules::Bb2025, json!({"cards": [], "dialogId": "useInducement", "inducementTypeArray": ["dwarfenWisdom"]})).is_err());
    }

    #[test]
    fn bb2025_card_lookup_throws() {
        assert!(round_trip(Rules::Bb2025, json!({"cards": ["Pit Trap"], "dialogId": "useInducement", "inducementTypeArray": []})).is_err());
    }

    #[test]
    fn missing_cards_throws() {
        assert!(round_trip(Rules::Bb2016, json!({"dialogId": "useInducement", "inducementTypeArray": []})).is_err());
    }
}
