use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;
use crate::model::SoundId;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogPenaltyShootoutParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogPenaltyShootoutParameter {
    pub home_rolls: Vec<i32>,
    pub away_rolls: Vec<i32>,
    pub home_won: Vec<bool>,
    /// Java `List<String>`: a `None` is a null element (decoded from JSON null).
    pub descriptions: Vec<Option<String>>,
    pub home_score: i32,
    pub away_score: i32,
    pub home_team_wins: bool,
    pub winning_sound: Option<SoundId>,
    pub losing_sound: Option<SoundId>,
}

impl DialogPenaltyShootoutParameter {
    pub fn add_shootout(&mut self, home: i32, away: i32, home_win: bool, round: String) {
        self.home_rolls.push(home);
        self.away_rolls.push(away);
        self.home_won.push(home_win);
        self.descriptions.push(Some(round));
    }

    pub fn home_team_wins(&self) -> bool { self.home_team_wins }
    pub fn get_home_rolls(&self) -> &[i32] { &self.home_rolls }
    pub fn get_away_rolls(&self) -> &[i32] { &self.away_rolls }
    pub fn get_home_won(&self) -> &[bool] { &self.home_won }
    pub fn get_descriptions(&self) -> &[Option<String>] { &self.descriptions }
    pub fn get_home_score(&self) -> i32 { self.home_score }
    pub fn get_away_score(&self) -> i32 { self.away_score }
    pub fn get_winning_sound(&self) -> Option<SoundId> { self.winning_sound }
    pub fn get_losing_sound(&self) -> Option<SoundId> { self.losing_sound }
}

impl IDialogParameter for DialogPenaltyShootoutParameter {
    fn get_id(&self) -> DialogId { DialogId::PENALTY_SHOOTOUT }
    fn transform(&self) -> Box<dyn IDialogParameter> {
        Box::new(DialogPenaltyShootoutParameter {
            home_rolls: self.away_rolls.clone(),
            away_rolls: self.home_rolls.clone(),
            home_won: self.home_won.iter().map(|w| !w).collect(),
            descriptions: self.descriptions.clone(),
            home_score: self.away_score,
            away_score: self.home_score,
            home_team_wins: !self.home_team_wins,
            winning_sound: self.winning_sound,
            losing_sound: self.losing_sound,
        })
    }
}

// JSON serialization (BACKLOG §O).
impl DialogPenaltyShootoutParameter {
    /// Java `initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, _rules: crate::enums::Rules, json: &serde_json::Value) -> Result<(), String> {
        use crate::util::java_json as jj;
        let obj = jj::to_json_object(json)?;
        super::util_dialog_parameter::validate_dialog_id_json(DialogId::PENALTY_SHOOTOUT, obj)?;
        // `Arrays.stream(int[])` / `Arrays.asList(String[])` / `for (boolean win : boolean[])` with no
        // null checks: an absent (or null) array throws.
        let rolls_home = jj::get_int_array(obj, "rollsHome")?.ok_or_else(|| "rollsHome: null (NullPointerException)".to_string())?;
        self.home_rolls.extend(rolls_home);
        let rolls_away = jj::get_int_array(obj, "rollsAway")?.ok_or_else(|| "rollsAway: null (NullPointerException)".to_string())?;
        self.away_rolls.extend(rolls_away);
        self.home_won.extend(jj::get_boolean_array(obj, "penaltyWins")?);
        self.home_score = jj::get_int(obj, "penaltyScoreHome")?;
        self.away_score = jj::get_int(obj, "penaltyScoreAway")?;
        self.home_team_wins = jj::get_boolean(obj, "homeTeam")?;
        let descriptions = jj::get_string_array(obj, "descriptions")?
            .ok_or_else(|| "descriptions: null (NullPointerException)".to_string())?;
        self.descriptions.extend(descriptions);
        // JsonEnumWithNameOption over SoundIdFactory (equalsIgnoreCase; unknown -> null).
        self.winning_sound = jj::get_enum_name(obj, "sound")?.and_then(SoundId::for_name_ignore_case);
        self.losing_sound = jj::get_enum_name(obj, "soundAlternative")?.and_then(SoundId::for_name_ignore_case);
        Ok(())
    }

    /// Java `toJsonValue()`: `super.toJsonValue()` (`DialogWithoutParameter`: the dialog id) plus the
    /// shootout fields.
    pub fn to_json_value(&self, _rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        use crate::util::java_json as jj;
        use serde_json::Value;
        let mut o = match super::dialog_without_parameter::DialogWithoutParameter::to_json_value(DialogId::PENALTY_SHOOTOUT) {
            Value::Object(o) => o,
            other => return Err(format!("DialogWithoutParameter.toJsonValue: not an object: {other}")),
        };
        o.insert("rollsHome".to_string(), jj::int_array_or_null(Some(self.home_rolls.as_slice())));
        o.insert("rollsAway".to_string(), jj::int_array_or_null(Some(self.away_rolls.as_slice())));
        o.insert("penaltyScoreHome".to_string(), Value::from(self.home_score));
        o.insert("penaltyScoreAway".to_string(), Value::from(self.away_score));
        o.insert("penaltyWins".to_string(), jj::boolean_array_or_null(Some(self.home_won.as_slice())));
        o.insert("homeTeam".to_string(), Value::Bool(self.home_team_wins));
        o.insert("descriptions".to_string(), jj::string_array_or_null(Some(self.descriptions.as_slice())));
        o.insert("sound".to_string(), jj::string_or_null(self.winning_sound.map(SoundId::get_name)));
        o.insert("soundAlternative".to_string(), jj::string_or_null(self.losing_sound.map(SoundId::get_name)));
        Ok(Value::Object(o))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_shootout_appends_to_all_vecs() {
        let mut p = DialogPenaltyShootoutParameter::default();
        p.add_shootout(5, 3, true, "Round 1".into());
        assert_eq!(p.get_home_rolls(), &[5]);
        assert_eq!(p.get_away_rolls(), &[3]);
        assert_eq!(p.get_home_won(), &[true]);
        assert_eq!(p.get_descriptions(), &[Some("Round 1".to_string())]);
    }

    #[test]
    fn add_shootout_multiple_rounds_accumulate() {
        let mut p = DialogPenaltyShootoutParameter::default();
        p.add_shootout(3, 5, false, "Round 1".into());
        p.add_shootout(6, 2, true, "Round 2".into());
        assert_eq!(p.get_home_rolls().len(), 2);
        assert_eq!(p.get_away_rolls()[0], 5);
        assert_eq!(p.get_home_won()[1], true);
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
        assert_eq!(round_trip(Rules::Bb2025, json!({"descriptions": ["1st", "2nd"], "dialogId": "penaltyShootout", "homeTeam": false, "penaltyScoreAway": 3, "penaltyScoreHome": 2, "penaltyWins": [false, true], "rollsAway": [4, 4], "rollsHome": [1, 6], "sound": "specClap", "soundAlternative": "specAah"})).unwrap(), json!({"descriptions": ["1st", "2nd"], "dialogId": "penaltyShootout", "homeTeam": false, "penaltyScoreAway": 3, "penaltyScoreHome": 2, "penaltyWins": [false, true], "rollsAway": [4, 4], "rollsHome": [1, 6], "sound": "specClap", "soundAlternative": "specAah"}));
    }

    #[test]
    fn sounds_are_case_insensitive_or_null() {
        assert_eq!(round_trip(Rules::Bb2025, json!({"descriptions": [null], "dialogId": "penaltyShootout", "homeTeam": true, "penaltyWins": [], "rollsAway": [], "rollsHome": [], "sound": "SPECCLAP", "soundAlternative": "nope"})).unwrap(), json!({"descriptions": [null], "dialogId": "penaltyShootout", "homeTeam": true, "penaltyScoreAway": 0, "penaltyScoreHome": 0, "penaltyWins": [], "rollsAway": [], "rollsHome": [], "sound": "specClap", "soundAlternative": null}));
    }

    #[test]
    fn missing_rolls_throws() {
        assert!(round_trip(Rules::Bb2025, json!({"descriptions": [], "dialogId": "penaltyShootout", "homeTeam": true, "penaltyWins": [], "rollsAway": []})).is_err());
    }

    #[test]
    fn missing_home_team_throws() {
        assert!(round_trip(Rules::Bb2025, json!({"descriptions": [], "dialogId": "penaltyShootout", "penaltyWins": [], "rollsAway": [], "rollsHome": []})).is_err());
    }
}
