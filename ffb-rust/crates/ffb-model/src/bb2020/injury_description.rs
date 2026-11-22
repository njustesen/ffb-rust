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

    /// Java `initFrom(IFactorySource, JsonValue)`. `serious_injury` keeps the `getName()` of the
    /// ruleset's injury (`SeriousInjuryFactory.forName`: unknown -> null); `apothecary_types` the
    /// `ApothecaryType` constant names (`valueOf`: unknown or null -> throws).
    pub fn init_from(&mut self, rules: crate::enums::Rules, json: &serde_json::Value) -> Result<(), String> {
        use crate::util::java_json as jj;
        let obj = jj::to_json_object(json)?;
        self.player_id = jj::get_string(obj, "playerId")?;
        self.player_state = jj::get_player_state_id(obj, "playerState")?;
        self.serious_injury = read_serious_injury(rules, obj, "seriousInjury")?;
        if jj::is_defined_in(obj, "apothecaryTypes") {
            self.apothecary_types.extend(read_apothecary_types(obj)?);
        }
        Ok(())
    }

    /// Java `toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        use crate::util::java_json as jj;
        let mut o = serde_json::Map::new();
        o.insert("playerId".into(), jj::string_or_null(self.player_id.as_deref()));
        o.insert("playerState".into(), jj::player_state_or_null(self.player_state));
        o.insert("seriousInjury".into(), jj::string_or_null(self.serious_injury.as_deref()));
        o.insert("apothecaryTypes".into(), jj::string_array(&self.apothecary_types));
        serde_json::Value::Object(o)
    }
}

/// `(SeriousInjury) JsonEnumWithNameOption(SERIOUS_INJURY..).getFrom`: absent/null -> null, else
/// the ruleset factory's `forName` (exact; unknown -> null), kept as its `getName()`.
pub fn read_serious_injury(
    rules: crate::enums::Rules,
    obj: &crate::util::java_json::JsonObject,
    key: &str,
) -> Result<Option<String>, String> {
    use crate::factory::serious_injury_factory as sif;
    use crate::model::serious_injury::SeriousInjury;
    Ok(crate::util::java_json::get_enum_name(obj, key)?
        .and_then(|name| sif::for_name_in(rules, name))
        .map(|si| si.get_name().to_string()))
}

/// `Arrays.stream(APOTHECARY_TYPES.getFrom(..)).map(ApothecaryType::valueOf)` (the caller checks
/// `isDefinedIn`): a JSON-null array -> `Arrays.stream(null)` NPE, a null element or an unknown
/// name -> `valueOf` throws. Returns the constant names.
pub fn read_apothecary_types(obj: &crate::util::java_json::JsonObject) -> Result<Vec<String>, String> {
    use crate::enums::ApothecaryType;
    crate::util::java_json::get_string_array(obj, "apothecaryTypes")?
        .ok_or("apothecaryTypes: NullPointerException (Arrays.stream(null))")?
        .into_iter()
        .map(|name| {
            let name = name.ok_or("ApothecaryType.valueOf: Name is null")?;
            ApothecaryType::from_name(&name)
                .map(|t| t.name().to_string())
                .ok_or(format!("No enum constant ApothecaryType.{name}"))
        })
        .collect()
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
