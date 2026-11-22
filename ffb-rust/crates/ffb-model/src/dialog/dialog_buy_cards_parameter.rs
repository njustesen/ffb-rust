use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogBuyCardsParameter.
/// `fNrOfCardsPerType` (a `HashMap<CardType, Integer>`) is kept as (canonical card type name or
/// null, count) pairs: an unknown type name decodes to a null key, as in Java.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogBuyCardsParameter {
    pub team_id: Option<String>,
    pub available_gold: i32,
    pub available_cards: i32,
    pub nr_of_cards_per_type: Vec<(Option<String>, i32)>,
}

impl DialogBuyCardsParameter {
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_available_gold(&self) -> i32 { self.available_gold }
    pub fn get_available_cards(&self) -> i32 { self.available_cards }
    pub fn get_nr_of_cards_per_type(&self) -> &[(Option<String>, i32)] { &self.nr_of_cards_per_type }
}

impl IDialogParameter for DialogBuyCardsParameter {
    fn get_id(&self) -> DialogId { DialogId::BUY_CARDS }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

// JSON serialization (BACKLOG §O).
impl DialogBuyCardsParameter {
    /// Java `put(CardType, int)`: `HashMap.put` -- an existing key (incl. the null type) is
    /// overwritten in place.
    pub fn put(&mut self, card_type: Option<String>, nr_of_cards: i32) {
        match self.nr_of_cards_per_type.iter_mut().find(|(t, _)| *t == card_type) {
            Some(entry) => entry.1 = nr_of_cards,
            None => self.nr_of_cards_per_type.push((card_type, nr_of_cards)),
        }
    }

    /// Java `initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, rules: crate::enums::Rules, json: &serde_json::Value) -> Result<(), String> {
        use crate::util::java_json as jj;
        let obj = jj::to_json_object(json)?;
        super::util_dialog_parameter::validate_dialog_id_json(DialogId::BUY_CARDS, obj)?;
        self.team_id = jj::get_string(obj, "teamId")?;
        self.available_cards = jj::get_int(obj, "availableCards")?;
        self.available_gold = jj::get_int(obj, "availableGold")?;
        // JsonArrayOption: absent/null -> an empty array; every element `asObject()` (throws on a
        // non-object); CARD_TYPE is a JsonEnumWithNameOption over the ruleset's CardTypeFactory.
        for element in jj::get_json_array_option(obj, "nrOfCardsPerType")? {
            let inner = element.as_object().ok_or_else(|| format!("nrOfCardsPerType: asObject on {element}"))?;
            let card_type = jj::get_enum_name(inner, "cardType")?
                .and_then(|name| crate::factory::card_type_factory::for_name_in(rules, name))
                .map(str::to_string);
            let nr_of_cards = jj::get_int(inner, "nrOfCards")?;
            self.put(card_type, nr_of_cards);
        }
        Ok(())
    }

    /// Java `toJsonValue()`.
    pub fn to_json_value(&self, _rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        use crate::util::java_json as jj;
        use serde_json::Value;
        let mut o = serde_json::Map::new();
        o.insert("dialogId".to_string(), Value::String(DialogId::BUY_CARDS.get_name().to_string()));
        o.insert("teamId".to_string(), jj::string_or_null(self.team_id.as_deref()));
        o.insert("availableCards".to_string(), Value::from(self.available_cards));
        o.insert("availableGold".to_string(), Value::from(self.available_gold));
        // the map's entries as {cardType, nrOfCards} objects (HashMap order -- the canonical form
        // sorts this array); a null type is written as JSON null.
        let nr_of_cards_per_type: Vec<Value> = self
            .nr_of_cards_per_type
            .iter()
            .map(|(card_type, nr_of_cards)| {
                let mut entry = serde_json::Map::new();
                entry.insert("cardType".to_string(), jj::string_or_null(card_type.as_deref()));
                entry.insert("nrOfCards".to_string(), Value::from(*nr_of_cards));
                Value::Object(entry)
            })
            .collect();
        o.insert("nrOfCardsPerType".to_string(), Value::Array(nr_of_cards_per_type));
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
    fn round_trip_bb2016() {
        assert_eq!(round_trip(Rules::Bb2016, json!({"dialogId": "buyCards", "teamId": "t1", "availableCards": 2, "availableGold": 520050, "nrOfCardsPerType": [{"cardType": "dirtyTrick", "nrOfCards": 12}, {"cardType": "magicItem", "nrOfCards": 12}]})).unwrap(), json!({"dialogId": "buyCards", "teamId": "t1", "availableCards": 2, "availableGold": 520050, "nrOfCardsPerType": [{"cardType": "dirtyTrick", "nrOfCards": 12}, {"cardType": "magicItem", "nrOfCards": 12}]}));
    }

    #[test]
    fn unknown_type_is_a_null_key_and_bb2025_has_no_types() {
        assert_eq!(round_trip(Rules::Bb2025, json!({"dialogId": "buyCards", "nrOfCardsPerType": [{"cardType": "magicItem", "nrOfCards": 1}, {"cardType": "dirtyTrick", "nrOfCards": 2}]})).unwrap(), json!({"dialogId": "buyCards", "teamId": null, "availableCards": 0, "availableGold": 0, "nrOfCardsPerType": [{"cardType": null, "nrOfCards": 2}]}));
    }

    #[test]
    fn missing_array_is_empty_and_type_is_case_insensitive() {
        assert_eq!(round_trip(Rules::Bb2020, json!({"dialogId": "buyCards", "nrOfCardsPerType": [{"cardType": "MAGICITEM", "nrOfCards": 3}]})).unwrap(), json!({"dialogId": "buyCards", "teamId": null, "availableCards": 0, "availableGold": 0, "nrOfCardsPerType": [{"cardType": "magicItem", "nrOfCards": 3}]}));
    }

    #[test]
    fn non_object_element_throws() {
        assert!(round_trip(Rules::Bb2016, json!({"dialogId": "buyCards", "nrOfCardsPerType": [null]})).is_err());
    }
}
