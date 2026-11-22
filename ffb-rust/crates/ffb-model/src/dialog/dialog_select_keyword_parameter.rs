use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogSelectKeywordParameter.
/// Note: Keyword/KeywordChoiceMode serialized as String (stubs not yet translated).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogSelectKeywordParameter {
    pub keywords: Vec<String>,
    pub player_id: Option<String>,
    pub keyword_choice_mode: Option<String>,
    pub min_select: i32,
    pub max_select: i32,
}

impl DialogSelectKeywordParameter {
    pub fn get_keywords(&self) -> &[String] { &self.keywords }
    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_keyword_choice_mode(&self) -> Option<&str> { self.keyword_choice_mode.as_deref() }
    pub fn get_min_select(&self) -> i32 { self.min_select }
    pub fn get_max_select(&self) -> i32 { self.max_select }

    /// Java: `transform()` returns `new DialogSelectKeywordParameter(playerId, keywords,
    /// keywordChoiceMode, 1, 1)` — min/max select are hardcoded to 1, not carried over
    /// from the original instance.
    pub fn transform_typed(&self) -> Self {
        DialogSelectKeywordParameter {
            keywords: self.keywords.clone(),
            player_id: self.player_id.clone(),
            keyword_choice_mode: self.keyword_choice_mode.clone(),
            min_select: 1,
            max_select: 1,
        }
    }
}

impl IDialogParameter for DialogSelectKeywordParameter {
    fn get_id(&self) -> DialogId { DialogId::SELECT_KEYWORD }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.transform_typed()) }
}

// JSON serialization (BACKLOG §O). `keywords` holds `Keyword.getName()` values,
// `keyword_choice_mode` the `KeywordChoiceMode` constant name (what Java writes).
impl DialogSelectKeywordParameter {
    /// Java `initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, _rules: crate::enums::Rules, json: &serde_json::Value) -> Result<(), String> {
        use crate::util::java_json as jj;
        let obj = jj::to_json_object(json)?;
        super::util_dialog_parameter::validate_dialog_id_json(DialogId::SELECT_KEYWORD, obj)?;
        self.player_id = jj::get_string(obj, "playerId")?;
        // `for (String s : keyWordArray)` with no null check: absent -> NullPointerException.
        let keyword_array = jj::get_string_array(obj, "keywords")?
            .ok_or_else(|| "keywords: null (NullPointerException)".to_string())?;
        for name in keyword_array {
            // Keyword.forName: equalsIgnoreCase, fallback UNKNOWN (also for a null element).
            let keyword = match name {
                Some(n) => crate::model::keyword::Keyword::for_name(&n),
                None => crate::model::keyword::Keyword::UNKNOWN,
            };
            self.keywords.push(keyword.get_name().to_string());
        }
        // KeywordChoiceMode.valueOf: the exact constant name (null / unknown throw).
        let mode = crate::model::keyword_choice_mode::KeywordChoiceMode::value_of(
            jj::get_string(obj, "keywordChoiceMode")?.as_deref(),
        )?;
        self.keyword_choice_mode = Some(mode.name().to_string());
        self.min_select = jj::get_int(obj, "minSelects")?;
        self.max_select = jj::get_int(obj, "maxSelects")?;
        Ok(())
    }

    /// Java `toJsonValue()`.
    pub fn to_json_value(&self, _rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        use crate::util::java_json as jj;
        use serde_json::Value;
        let mut o = serde_json::Map::new();
        o.insert("dialogId".to_string(), Value::String(DialogId::SELECT_KEYWORD.get_name().to_string()));
        o.insert("playerId".to_string(), jj::string_or_null(self.player_id.as_deref()));
        o.insert("keywords".to_string(), jj::string_array(&self.keywords));
        // `keywordChoiceMode.name()`: a null mode -> NullPointerException.
        let mode = self.keyword_choice_mode.as_deref().ok_or_else(|| {
            "DialogSelectKeywordParameter.toJsonValue: keywordChoiceMode is null (NullPointerException)".to_string()
        })?;
        o.insert("keywordChoiceMode".to_string(), Value::String(mode.to_string()));
        o.insert("minSelects".to_string(), Value::from(self.min_select));
        o.insert("maxSelects".to_string(), Value::from(self.max_select));
        Ok(Value::Object(o))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn transform_resets_min_max_select_to_one() {
        // Java's transform() always hardcodes minSelect/maxSelect to 1, regardless of
        // the original values — it does NOT just clone the instance.
        let p = DialogSelectKeywordParameter {
            player_id: Some("p1".into()),
            keywords: vec!["PASS".into()],
            keyword_choice_mode: Some("MULTIPLE".into()),
            min_select: 2,
            max_select: 5,
        };
        let transformed = p.transform_typed();
        assert_eq!(transformed.get_min_select(), 1);
        assert_eq!(transformed.get_max_select(), 1);
        assert_eq!(transformed.get_player_id(), Some("p1"));
        assert_eq!(transformed.get_keywords(), &["PASS".to_string()]);
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
        assert_eq!(round_trip(Rules::Bb2025, json!({"dialogId": "selectKeyword", "keywordChoiceMode": "GETTING_EVEN", "keywords": ["Undead", "Werewolf"], "maxSelects": 1, "minSelects": 1, "playerId": "p1"})).unwrap(), json!({"dialogId": "selectKeyword", "keywordChoiceMode": "GETTING_EVEN", "keywords": ["Undead", "Werewolf"], "maxSelects": 1, "minSelects": 1, "playerId": "p1"}));
    }

    #[test]
    fn keywords_are_canonical_or_unknown() {
        assert_eq!(round_trip(Rules::Bb2025, json!({"dialogId": "selectKeyword", "keywordChoiceMode": "GETTING_EVEN", "keywords": ["undead", "Gargoyle", null]})).unwrap(), json!({"dialogId": "selectKeyword", "keywordChoiceMode": "GETTING_EVEN", "keywords": ["Undead", "Unknown", "Unknown"], "maxSelects": 0, "minSelects": 0, "playerId": null}));
    }

    #[test]
    fn mode_name_is_exact() {
        assert!(round_trip(Rules::Bb2025, json!({"dialogId": "selectKeyword", "keywordChoiceMode": "gettingEven", "keywords": []})).is_err());
    }

    #[test]
    fn missing_keywords_throws() {
        assert!(round_trip(Rules::Bb2025, json!({"dialogId": "selectKeyword", "keywordChoiceMode": "GETTING_EVEN"})).is_err());
    }
}
