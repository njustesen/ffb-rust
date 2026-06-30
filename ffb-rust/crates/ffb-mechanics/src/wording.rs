/// 1:1 translation of com.fumbbl.ffb.mechanics.Wording.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wording {
    noun: String,
    verb: String,
    inflection: String,
    player_characterization: String,
}

impl Wording {
    pub fn new(noun: impl Into<String>, verb: impl Into<String>, inflection: impl Into<String>, player_characterization: impl Into<String>) -> Self {
        Wording {
            noun: noun.into(),
            verb: verb.into(),
            inflection: inflection.into(),
            player_characterization: player_characterization.into(),
        }
    }

    pub fn get_noun(&self) -> &str { &self.noun }
    pub fn get_verb(&self) -> &str { &self.verb }
    pub fn get_inflection(&self) -> &str { &self.inflection }
    pub fn get_player_characterization(&self) -> &str { &self.player_characterization }
}
