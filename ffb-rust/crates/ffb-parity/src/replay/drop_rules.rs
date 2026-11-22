//! The harness's newer-server drop rules (`FumbblReplayParser`, BACKLOG §O iteration 2), ported
//! 1:1 so the Rust loader decodes exactly the JSON the Java engine decodes. Each returns the
//! `countUnknown` keys it would count, in order.

use ffb_model::enums::Rules;
use ffb_model::model::game_options::GameOptions;
use serde_json::Value;

/// `FumbblReplayParser.dropUnknownInducements(game, gameJson)`: in
/// `turnData{Home,Away}.inducementSet`, remove every `inducementArray` element whose
/// `inducementType` string the game's `InducementTypeFactory` does not know (key
/// `inducementArray=<name>`), and -- only when the game HAS a prayer factory (BB2020/BB2025) --
/// every string `prayers` element its `forName` does not know (key `prayers=<name>`).
/// `Err` where minimal-json's `getString(name, null)` would throw (a non-string type value).
pub fn drop_unknown_inducements(rules: Rules, options: &GameOptions, game_json: &mut Value) -> Result<Vec<String>, String> {
    let mut dropped_keys = Vec::new();
    for turn_data in ["turnDataHome", "turnDataAway"] {
        let Some(set) = game_json
            .get_mut(turn_data)
            .filter(|td| td.is_object())
            .and_then(|td| td.get_mut("inducementSet"))
            .and_then(Value::as_object_mut)
        else {
            continue;
        };
        if let Some(Value::Array(inducements)) = set.get_mut("inducementArray") {
            let mut kept = Vec::with_capacity(inducements.len());
            for ind in inducements.drain(..) {
                let type_name = match ind.as_object().and_then(|o| o.get("inducementType")) {
                    None => None,
                    Some(Value::String(s)) => Some(s.clone()),
                    Some(other) => return Err(format!("{turn_data}: inducementType getString on {other}")),
                };
                match type_name {
                    Some(name) if ffb_model::factory::inducement_type_factory::for_name_in(rules, &name).is_none() => {
                        dropped_keys.push(format!("inducementArray={name}"));
                    }
                    _ => kept.push(ind),
                }
            }
            *inducements = kept;
        }
        let has_prayer_factory = matches!(rules, Rules::Bb2020 | Rules::Bb2025);
        if let (true, Some(Value::Array(prayers))) = (has_prayer_factory, set.get_mut("prayers")) {
            let mut kept = Vec::with_capacity(prayers.len());
            for p in prayers.drain(..) {
                if let Value::String(name) = &p {
                    if ffb_model::factory::prayer_factory::for_name_in(rules, options, name)?.is_none() {
                        dropped_keys.push(format!("prayers={name}"));
                        continue;
                    }
                }
                kept.push(p);
            }
            *prayers = kept;
        }
    }
    Ok(dropped_keys)
}

/// `FumbblReplayParser.knownInducementType(game, name)`: the game's `InducementTypeFactory.forName`.
fn known_inducement_type(rules: Rules, name: &str) -> bool {
    ffb_model::factory::inducement_type_factory::for_name_in(rules, name).is_some()
}

/// `FumbblReplayParser.dropUnknownValues(game, mc)` on one model change of a known id: values
/// inside it that the in-repo engine has no constant for. Returns `(drop the whole change,
/// countUnknown keys)`; otherwise removes the unknown elements in place. The game has a prayer
/// factory only in BB2020/BB2025 (`prayers != null`).
/// `Err` where minimal-json would throw (`getString` on a non-string `inducementType`).
pub fn drop_unknown_values(rules: Rules, options: &GameOptions, mc: &mut Value) -> Result<(bool, Vec<String>), String> {
    let mut keys = Vec::new();
    let id = match mc.get("modelChangeId") {
        None => String::new(),
        Some(Value::String(s)) => s.clone(),
        Some(other) => return Err(format!("modelChangeId getString on {other}")),
    };
    let has_prayer_factory = matches!(rules, Rules::Bb2020 | Rules::Bb2025);
    let value = mc.get_mut("modelChangeValue");
    match id.as_str() {
        "fieldModelAddPrayer" | "fieldModelRemovePrayer" => {
            // prayers.valueOf(name): the ruleset's Prayer enum constant, IllegalArgumentException if none
            if let (true, Some(Value::String(name))) = (has_prayer_factory, value.as_deref()) {
                let known = match rules {
                    Rules::Bb2020 => ffb_model::inducement::bb2020::prayer::Prayer::for_enum_name(name).is_some(),
                    _ => ffb_model::inducement::bb2025::prayer::Prayer::for_enum_name(name).is_some(),
                };
                if !known {
                    keys.push(format!("{id}={name}"));
                    return Ok((true, keys));
                }
            }
            return Ok((false, keys));
        }
        "inducementSetAddPrayer" | "inducementSetRemovePrayer" => {
            if let (true, Some(Value::String(name))) = (has_prayer_factory, value.as_deref()) {
                if ffb_model::factory::prayer_factory::for_name_in(rules, options, name)?.is_none() {
                    keys.push(format!("{id}={name}"));
                    return Ok((true, keys));
                }
            }
            return Ok((false, keys));
        }
        "inducementSetAddInducement" | "inducementSetRemoveInducement" => {
            if let Some(Value::Object(obj)) = value.as_deref() {
                let type_name = match obj.get("inducementType") {
                    None => None,
                    Some(Value::String(s)) => Some(s.as_str()),
                    Some(other) => return Err(format!("{id}: inducementType getString on {other}")),
                };
                if let Some(name) = type_name {
                    if !known_inducement_type(rules, name) {
                        keys.push(format!("{id}={name}"));
                        return Ok((true, keys));
                    }
                }
            }
            return Ok((false, keys));
        }
        _ => {}
    }
    if let Some(Value::Object(obj)) = value {
        // DialogUseInducementParameter: an inducement type the engine has no constant for.
        if let Some(Value::Array(types)) = obj.get_mut("inducementTypeArray") {
            types.retain(|t| match t {
                Value::String(name) if !known_inducement_type(rules, name) => {
                    keys.push(format!("inducementType={name}"));
                    false
                }
                _ => true,
            });
        }
        if let Some(Value::Array(props)) = obj.get_mut("reRollProperties") {
            props.retain(|p| match p {
                Value::String(name) if ffb_model::enums::ReRollProperty::for_name_ignore_case(name).is_none() => {
                    keys.push(format!("reRollProperty={name}"));
                    false
                }
                _ => true,
            });
        }
    }
    Ok((false, keys))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn drop_unknown_values_filters_dialog_arrays_and_drops_unknown_set_changes() {
        let options = GameOptions::new();
        let mut mc = json!({"modelChangeId": "gameSetDialogParameter", "modelChangeValue": {
            "dialogId": "blockRollProperties",
            "reRollProperties": ["TRR", "DRIVE_BONUS", "pro", 3]
        }});
        let (drop, keys) = drop_unknown_values(Rules::Bb2025, &options, &mut mc).unwrap();
        assert!(!drop);
        assert_eq!(keys, vec!["reRollProperty=DRIVE_BONUS"]);
        assert_eq!(mc["modelChangeValue"]["reRollProperties"], json!(["TRR", "pro", 3]));

        let mut mc = json!({"modelChangeId": "gameSetDialogParameter", "modelChangeValue": {
            "dialogId": "useInducement", "inducementTypeArray": ["dwarfenWisdom", "bribes"]
        }});
        let (drop, keys) = drop_unknown_values(Rules::Bb2025, &options, &mut mc).unwrap();
        assert!(!drop);
        assert_eq!(keys, vec!["inducementType=dwarfenWisdom"]);
        assert_eq!(mc["modelChangeValue"]["inducementTypeArray"], json!(["bribes"]));

        let mut mc = json!({"modelChangeId": "inducementSetAddInducement", "modelChangeValue": {"inducementType": "dwarfenWisdom"}});
        assert_eq!(drop_unknown_values(Rules::Bb2025, &options, &mut mc).unwrap(), (true, vec!["inducementSetAddInducement=dwarfenWisdom".to_string()]));

        let mut mc = json!({"modelChangeId": "inducementSetAddPrayer", "modelChangeValue": "Blessing of Nuffle"});
        assert!(drop_unknown_values(Rules::Bb2025, &options, &mut mc).unwrap().0);
        // BB2016 has no prayer factory: kept
        assert!(!drop_unknown_values(Rules::Bb2016, &options, &mut mc).unwrap().0);
    }

    #[test]
    fn drops_newer_server_inducements_and_prayers_like_the_harness() {
        let mut game = json!({
            "turnDataHome": {"inducementSet": {
                "inducementArray": [
                    {"inducementType": "dwarfenWisdom", "value": 1},
                    {"inducementType": "bribes", "value": 1},
                    {"value": 1}
                ],
                "prayers": ["Blessing of Nuffle", "Iron Man", 7]
            }},
            "turnDataAway": {"inducementSet": {"inducementArray": [{"inducementType": "josefBugman"}]}}
        });
        let keys = drop_unknown_inducements(Rules::Bb2025, &GameOptions::new(), &mut game).unwrap();
        assert_eq!(keys, vec!["inducementArray=dwarfenWisdom", "prayers=Blessing of Nuffle", "inducementArray=josefBugman"]);
        assert_eq!(game["turnDataHome"]["inducementSet"]["inducementArray"].as_array().unwrap().len(), 2);
        assert_eq!(game["turnDataHome"]["inducementSet"]["prayers"], json!(["Iron Man", 7]));
        assert_eq!(game["turnDataAway"]["inducementSet"]["inducementArray"], json!([]));
    }

    #[test]
    fn bb2016_has_no_prayer_factory_so_prayers_are_kept() {
        let mut game = json!({"turnDataHome": {"inducementSet": {"prayers": ["Blessing of Nuffle"]}}});
        let keys = drop_unknown_inducements(Rules::Bb2016, &GameOptions::new(), &mut game).unwrap();
        assert!(keys.is_empty());
    }
}
