use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogBuyCardsAndInducementsParameter.
/// `fNrOfCardsPerType` (a `HashMap<CardType, Integer>`) is kept as (canonical card type name or
/// null, count) pairs: an unknown type name decodes to a null key, as in Java.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogBuyCardsAndInducementsParameter {
    pub team_id: Option<String>,
    pub available_gold: i32,
    pub card_slots: i32,
    pub card_price: i32,
    pub nr_of_cards_per_type: Vec<(Option<String>, i32)>,
    /// Java `CardChoices` (null until decoded or set).
    pub card_choices: Option<DialogCardChoices>,
    pub can_buy_cards: bool,
    pub uses_treasury: bool,
}

impl DialogBuyCardsAndInducementsParameter {
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_available_gold(&self) -> i32 { self.available_gold }
    pub fn get_card_slots(&self) -> i32 { self.card_slots }
    pub fn get_card_price(&self) -> i32 { self.card_price }
    pub fn can_buy_cards(&self) -> bool { self.can_buy_cards }
    pub fn uses_treasury(&self) -> bool { self.uses_treasury }
}

/// Java `com.fumbbl.ffb.inducement.CardChoice` as this dialog carries it: the card type and the
/// two cards, each as its canonical name or null (Java's factories return null for an unknown name).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DialogCardChoice {
    pub card_type: Option<String>,
    pub choice_one: Option<String>,
    pub choice_two: Option<String>,
}

impl DialogCardChoice {
    /// Java `CardChoice.initFrom` (no validation): CARD_CHOICE_TYPE over the CardTypeFactory,
    /// CARD_CHOICE_ONE/TWO over the CardFactory (which throws for BB2025: no `Cards` there).
    fn init_from(&mut self, rules: crate::enums::Rules, obj: &crate::util::java_json::JsonObject) -> Result<(), String> {
        use crate::util::java_json as jj;
        self.card_type = jj::get_enum_name(obj, "cardChoiceType")?
            .and_then(|name| crate::factory::card_type_factory::for_name_in(rules, name))
            .map(str::to_string);
        self.choice_one = match jj::get_enum_name(obj, "cardChoice1")? {
            Some(name) => crate::factory::card_factory::for_name_in(rules, name)?,
            None => None,
        };
        self.choice_two = match jj::get_enum_name(obj, "cardChoice2")? {
            Some(name) => crate::factory::card_factory::for_name_in(rules, name)?,
            None => None,
        };
        Ok(())
    }

    /// Java `CardChoice.toJsonValue`: all three keys, null as JSON null.
    fn to_json_value(&self) -> serde_json::Value {
        use crate::util::java_json as jj;
        let mut o = serde_json::Map::new();
        o.insert("cardChoiceType".to_string(), jj::string_or_null(self.card_type.as_deref()));
        o.insert("cardChoice1".to_string(), jj::string_or_null(self.choice_one.as_deref()));
        o.insert("cardChoice2".to_string(), jj::string_or_null(self.choice_two.as_deref()));
        serde_json::Value::Object(o)
    }
}

/// Java `com.fumbbl.ffb.inducement.CardChoices` (initial + re-rolled choice).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DialogCardChoices {
    pub initial: Option<DialogCardChoice>,
    pub rerolled: Option<DialogCardChoice>,
}

impl DialogCardChoices {
    /// Java `CardChoices.initFrom`: each JsonObjectOption present -> `new CardChoice().initFrom`.
    fn init_from(&mut self, rules: crate::enums::Rules, obj: &crate::util::java_json::JsonObject) -> Result<(), String> {
        use crate::util::java_json as jj;
        if let Some(choice) = jj::get_object(obj, "cardChoiceFirst")? {
            let mut c = DialogCardChoice::default();
            c.init_from(rules, choice)?;
            self.initial = Some(c);
        }
        if let Some(choice) = jj::get_object(obj, "cardChoiceRerolled")? {
            let mut c = DialogCardChoice::default();
            c.init_from(rules, choice)?;
            self.rerolled = Some(c);
        }
        Ok(())
    }

    /// Java `CardChoices.toJsonValue`: a key only for a non-null choice.
    fn to_json_value(&self) -> serde_json::Value {
        let mut o = serde_json::Map::new();
        if let Some(c) = &self.initial {
            o.insert("cardChoiceFirst".to_string(), c.to_json_value());
        }
        if let Some(c) = &self.rerolled {
            o.insert("cardChoiceRerolled".to_string(), c.to_json_value());
        }
        serde_json::Value::Object(o)
    }
}

impl IDialogParameter for DialogBuyCardsAndInducementsParameter {
    fn get_id(&self) -> DialogId { DialogId::BUY_CARDS_AND_INDUCEMENTS }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

// JSON serialization (BACKLOG §O).
impl DialogBuyCardsAndInducementsParameter {
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
        super::util_dialog_parameter::validate_dialog_id_json(DialogId::BUY_CARDS_AND_INDUCEMENTS, obj)?;
        self.team_id = jj::get_string(obj, "teamId")?;
        self.card_slots = jj::get_int(obj, "availableCards")?;
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
        if let Some(choice_object) = jj::get_object(obj, "cardChoices")? {
            let mut card_choices = DialogCardChoices::default();
            card_choices.init_from(rules, choice_object)?;
            self.card_choices = Some(card_choices);
        }
        self.card_price = jj::get_int(obj, "cardsPrice")?;
        self.can_buy_cards = jj::get_boolean(obj, "canBuyCards")?;
        if jj::is_defined_in(obj, "usesTreasury") {
            self.uses_treasury = jj::get_boolean(obj, "usesTreasury")?;
        }
        Ok(())
    }

    /// Java `toJsonValue()`.
    pub fn to_json_value(&self, _rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        use crate::util::java_json as jj;
        use serde_json::Value;
        let mut o = serde_json::Map::new();
        o.insert("dialogId".to_string(), Value::String(DialogId::BUY_CARDS_AND_INDUCEMENTS.get_name().to_string()));
        o.insert("teamId".to_string(), jj::string_or_null(self.team_id.as_deref()));
        o.insert("availableCards".to_string(), Value::from(self.card_slots));
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
        // `cardChoices.toJsonValue()` with no null check: no cardChoices -> NullPointerException.
        let card_choices = self.card_choices.as_ref().ok_or_else(|| {
            "DialogBuyCardsAndInducementsParameter.toJsonValue: cardChoices is null (NullPointerException)".to_string()
        })?;
        o.insert("cardChoices".to_string(), card_choices.to_json_value());
        o.insert("cardsPrice".to_string(), Value::from(self.card_price));
        o.insert("canBuyCards".to_string(), Value::Bool(self.can_buy_cards));
        o.insert("usesTreasury".to_string(), Value::Bool(self.uses_treasury));
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
    fn round_trip_bb2020() {
        assert_eq!(round_trip(Rules::Bb2020, json!({"availableCards": 0, "availableGold": 40000, "canBuyCards": false, "cardChoices": {}, "cardsPrice": 100000, "dialogId": "buyCardsAndInducements", "nrOfCardsPerType": [{"cardType": "dirtyTrick", "nrOfCards": 0}, {"cardType": "magicItem", "nrOfCards": 0}], "teamId": "t1", "usesTreasury": true})).unwrap(), json!({"availableCards": 0, "availableGold": 40000, "canBuyCards": false, "cardChoices": {}, "cardsPrice": 100000, "dialogId": "buyCardsAndInducements", "nrOfCardsPerType": [{"cardType": "dirtyTrick", "nrOfCards": 0}, {"cardType": "magicItem", "nrOfCards": 0}], "teamId": "t1", "usesTreasury": true}));
    }

    #[test]
    fn card_choices_with_unknown_cards_bb2020() {
        assert_eq!(round_trip(Rules::Bb2020, json!({"dialogId": "buyCardsAndInducements", "canBuyCards": true, "cardChoices": {"cardChoiceFirst": {"cardChoiceType": "MagicItem", "cardChoice1": "Force Shield"}}})).unwrap(), json!({"dialogId": "buyCardsAndInducements", "teamId": null, "availableCards": 0, "availableGold": 0, "nrOfCardsPerType": [], "canBuyCards": true, "cardChoices": {"cardChoiceFirst": {"cardChoiceType": "magicItem", "cardChoice1": null, "cardChoice2": null}}, "cardsPrice": 0, "usesTreasury": false}));
    }

    #[test]
    fn card_choices_bb2016_resolve_cards() {
        assert_eq!(round_trip(Rules::Bb2016, json!({"dialogId": "buyCardsAndInducements", "canBuyCards": true, "cardChoices": {"cardChoiceRerolled": {"cardChoiceType": "dirtyTrick", "cardChoice1": "pit trap", "cardChoice2": "Nope"}}})).unwrap(), json!({"dialogId": "buyCardsAndInducements", "teamId": null, "availableCards": 0, "availableGold": 0, "nrOfCardsPerType": [], "canBuyCards": true, "cardChoices": {"cardChoiceRerolled": {"cardChoiceType": "dirtyTrick", "cardChoice1": "Pit Trap", "cardChoice2": null}}, "cardsPrice": 0, "usesTreasury": false}));
    }

    #[test]
    fn missing_card_choices_throws_on_encode() {
        assert!(round_trip(Rules::Bb2020, json!({"dialogId": "buyCardsAndInducements", "canBuyCards": true})).is_err());
    }

    #[test]
    fn missing_can_buy_cards_throws() {
        assert!(round_trip(Rules::Bb2020, json!({"dialogId": "buyCardsAndInducements", "cardChoices": {}})).is_err());
    }
}
