//! The reports of one `serverModelSync` as `FumbblReplayParser.parseFile` sees them (BACKLOG §O O3).
//!
//! Java decodes the whole `reportList` (`ReportList.initFrom`: `ReportFactory.forId(reportId)` of the
//! game's ruleset, then the class's `initFrom`) and the record builders read a handful of getters.
//! This view keeps each report's JSON plus its Java CLASS (the per-ruleset table dumped from the
//! engine, `java_names::REPORT_CLASSES`) and implements exactly those getters with the `IJsonOption`
//! semantics of the class's `initFrom` (`JsonIntOption` absent -> 0, a primitive `boolean` field
//! with an absent key throws, `JsonEnumWithNameOption` through the ruleset's factory ...).
//!
//! Scope note: Java decodes EVERY field of every report and a throw anywhere skips the whole sync;
//! the corpus has 0 skipped syncs on the Java side (run5), so this view decodes only what the
//! records read. A report id whose ruleset has no class (`NONE`) is an error here as in Java.

use super::java_names;
use ffb_model::enums::{PlayerState, Rules};
use ffb_model::factory::skill_factory::SkillFactory;
use ffb_model::factory::serious_injury_factory;
use ffb_model::model::serious_injury::SeriousInjury as _;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::java_json::{self, JsonObject};
use serde_json::Value;

/// The Java `ReportId` constant of a `getName()` (`ReportIdFactory.forName`: equalsIgnoreCase).
pub fn report_id_constant(name: &str) -> Option<&'static str> {
    constant_for_name(java_names::REPORT_ID, name)
}

/// The Java constant of a `getName()` in one of the generated tables, Java `equalsIgnoreCase`.
pub fn constant_for_name(table: &'static [(&'static str, &'static str)], name: &str) -> Option<&'static str> {
    table.iter().find(|(_, n)| java_json::java_equals_ignore_case(n, name)).map(|(c, _)| *c)
}

/// The Java constant of an exact `getName()` (the Rust model's `name()` is Java's `getName()`).
pub fn constant_of(table: &'static [(&'static str, &'static str)], name: &str) -> Result<&'static str, String> {
    table.iter().find(|(_, n)| *n == name).map(|(c, _)| *c).ok_or_else(|| format!("no Java constant for {name:?}"))
}

fn ruleset_key(rules: Rules) -> &'static str {
    match rules {
        Rules::Bb2016 | Rules::Common => "BB2016",
        Rules::Bb2020 => "BB2020",
        Rules::Bb2025 => "BB2025",
    }
}

/// `ReportSkillRoll` fields (every subclass inherits them unchanged).
#[derive(Debug, Clone)]
pub struct SkillRoll {
    pub player_id: Option<String>,
    pub successful: bool,
    pub minimum_roll: i32,
    pub re_rolled: bool,
}

/// One decoded report.
#[derive(Debug, Clone)]
pub struct ReplayReport {
    pub rules: Rules,
    /// Java `ReportId` constant (`report.getId().name()`).
    pub id: &'static str,
    /// Java class canonical name (`com.fumbbl.ffb.report.mixed.ReportInjury` ...).
    pub class: &'static str,
    /// `Some` iff the class is a `ReportSkillRoll` (`report instanceof ReportSkillRoll`).
    pub skill_roll: Option<SkillRoll>,
    pub json: JsonObject,
}

impl ReplayReport {
    /// `factory.forId(getId(element)).initFrom(source, element)` for one array element.
    pub fn decode(rules: Rules, element: &Value) -> Result<ReplayReport, String> {
        let obj = java_json::to_json_object(element)?.clone();
        let name = java_json::get_enum_name(&obj, "reportId")?.ok_or("report without reportId (forId(null))")?;
        let id = report_id_constant(name).ok_or_else(|| format!("unknown reportId {name}"))?;
        let key = ruleset_key(rules);
        let (_, _, class, chain) = java_names::REPORT_CLASSES
            .iter()
            .find(|(r, i, _, _)| *r == key && *i == id)
            .ok_or_else(|| format!("report class table has no {key} {id}"))?;
        if *class == "NONE" {
            return Err(format!("No constructor registered for reportId {name} ({key})"));
        }
        let skill_roll = if chain.split(',').any(|c| c == "com.fumbbl.ffb.report.ReportSkillRoll") {
            Some(SkillRoll {
                player_id: java_json::get_string(&obj, "playerId")?,
                successful: java_json::get_boolean(&obj, "successful")?,
                minimum_roll: java_json::get_int(&obj, "minimumRoll")?,
                re_rolled: java_json::get_boolean(&obj, "reRolled")?,
            })
        } else {
            None
        };
        Ok(ReplayReport { rules, id, class, skill_roll, json: obj })
    }

    pub fn is(&self, id: &str) -> bool {
        self.id == id
    }

    pub fn is_class(&self, class: &str) -> bool {
        self.class == class
    }

    // ── JsonOption getters ────────────────────────────────────────────────────

    pub fn string(&self, key: &str) -> Result<Option<String>, String> {
        java_json::get_string(&self.json, key)
    }

    /// A primitive `boolean` field (absent key -> NPE on unboxing).
    pub fn boolean(&self, key: &str) -> Result<bool, String> {
        java_json::get_boolean(&self.json, key)
    }

    pub fn int(&self, key: &str) -> Result<i32, String> {
        java_json::get_int(&self.json, key)
    }

    pub fn int_array(&self, key: &str) -> Result<Option<Vec<i32>>, String> {
        java_json::get_int_array(&self.json, key)
    }

    pub fn player_state(&self, key: &str) -> Result<Option<PlayerState>, String> {
        java_json::get_player_state(&self.json, key)
    }

    pub fn field_coordinate(&self, key: &str) -> Result<Option<FieldCoordinate>, String> {
        java_json::get_field_coordinate(&self.json, key)
    }

    /// `(SeriousInjury) JsonEnumWithNameOption(key, SERIOUS_INJURY)` -> `getName()` of the result.
    pub fn serious_injury_name(&self, key: &str) -> Result<Option<String>, String> {
        Ok(java_json::get_enum_name(&self.json, key)?
            .and_then(|n| serious_injury_factory::for_name_in(self.rules, n))
            .map(|si| si.get_name().to_string()))
    }

    /// `(Skill) JsonEnumWithNameOption(key, SKILL)` -> `getName()` of the result.
    pub fn skill_name(&self, key: &str) -> Result<Option<&'static str>, String> {
        Ok(SkillFactory::from_json_value_in(self.rules, self.json.get(key))?.and_then(|id| SkillFactory::name_in(self.rules, id)))
    }

    /// A `JsonEnumWithNameOption` of an enum factory (`forName` = equalsIgnoreCase over getName)
    /// -> the Java constant.
    pub fn enum_constant(&self, key: &str, table: &'static [(&'static str, &'static str)]) -> Result<Option<&'static str>, String> {
        Ok(java_json::get_enum_name(&self.json, key)?.and_then(|n| constant_for_name(table, n)))
    }

    // ── per-class getters the records read ───────────────────────────────────

    /// `getPlayerId()` of a ReportSkillRoll or of the classes that declare their own.
    pub fn player_id(&self) -> Result<Option<String>, String> {
        match &self.skill_roll {
            Some(sr) => Ok(sr.player_id.clone()),
            None => self.string("playerId"),
        }
    }

    pub fn defender_id(&self) -> Result<Option<String>, String> {
        self.string("defenderId")
    }

    /// `getBallCoordinateEnd` / `getScatterDirection` (constant) / `getRollScatterDistance` of
    /// ReportPassDeviate and ReportKickoffScatter.
    pub fn scatter(&self) -> Result<(Option<FieldCoordinate>, Option<&'static str>, i32), String> {
        Ok((
            self.field_coordinate("ballCoordinateEnd")?,
            self.enum_constant("scatterDirection", java_names::DIRECTION)?,
            self.int("rollScatterDistance")?,
        ))
    }
}

/// `ReportList.initFrom` of a sync's `reportList` object (absent -> empty).
pub fn decode_report_list(rules: Rules, report_list: Option<&Value>) -> Result<Vec<ReplayReport>, String> {
    let Some(rl) = report_list else { return Ok(Vec::new()) };
    if rl.is_null() {
        return Ok(Vec::new());
    }
    let obj = java_json::to_json_object(rl)?;
    let Some(reports) = java_json::get_array(obj, "reports")? else { return Ok(Vec::new()) };
    reports.iter().map(|r| ReplayReport::decode(rules, r)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn report_ids_resolve_ignoring_case_to_java_constants() {
        assert_eq!(report_id_constant("leapRoll"), Some("JUMP_ROLL"));
        assert_eq!(report_id_constant("LEAPROLL"), Some("JUMP_ROLL"));
        assert_eq!(report_id_constant("kickTeamMateRoll"), Some("KICK_TEAM_MATE_ROLL"));
        assert_eq!(report_id_constant("dwarfenWisdomRoll"), None);
    }

    #[test]
    fn skill_roll_classes_follow_the_ruleset_factory() {
        // GETTING_EVEN_ROLL is a ReportSkillRoll only where bb2025 registers it.
        let r = ReplayReport::decode(
            Rules::Bb2025,
            &json!({"reportId": "dodgeRoll", "playerId": "p1", "successful": false, "roll": 1, "minimumRoll": 3, "reRolled": false}),
        )
        .unwrap();
        assert_eq!(r.id, "DODGE_ROLL");
        assert_eq!(r.class, "com.fumbbl.ffb.report.mixed.ReportDodgeRoll");
        let sr = r.skill_roll.unwrap();
        assert_eq!((sr.player_id.as_deref(), sr.successful, sr.minimum_roll, sr.re_rolled), (Some("p1"), false, 3, false));
        let injury = ReplayReport::decode(Rules::Bb2016, &json!({"reportId": "injury", "defenderId": "d"})).unwrap();
        assert_eq!(injury.class, "com.fumbbl.ffb.report.bb2016.ReportInjury");
        assert!(injury.skill_roll.is_none());
        // BB2016 has no SELECT_GAZE_TARGET class: Java's forId throws.
        assert!(ReplayReport::decode(Rules::Bb2016, &json!({"reportId": "selectGazeTarget"})).is_err());
        // A skill roll without its primitive boolean throws on unboxing.
        assert!(ReplayReport::decode(Rules::Bb2020, &json!({"reportId": "dodgeRoll", "reRolled": false})).is_err());
    }

    #[test]
    fn enum_constants_come_from_the_java_declarations() {
        assert_eq!(constant_for_name(java_names::DIRECTION, "northeast"), Some("NORTHEAST"));
        assert_eq!(constant_for_name(java_names::PLAYER_ACTION, "handOverMove"), Some("HAND_OVER_MOVE"));
        assert_eq!(constant_of(java_names::WEATHER, "Nice Weather").unwrap(), "NICE");
        assert!(constant_of(java_names::TURN_MODE, "nope").is_err());
    }
}
