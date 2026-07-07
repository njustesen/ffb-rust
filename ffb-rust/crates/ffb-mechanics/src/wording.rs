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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn getters_return_constructed_values() {
        let w = Wording::new("Pass", "passes", "es", "thrower");
        assert_eq!(w.get_noun(), "Pass");
        assert_eq!(w.get_verb(), "passes");
        assert_eq!(w.get_inflection(), "es");
        assert_eq!(w.get_player_characterization(), "thrower");
    }

    #[test]
    fn equal_wordings_compare_equal() {
        let a = Wording::new("Pass", "passes", "es", "thrower");
        let b = Wording::new("Pass", "passes", "es", "thrower");
        assert_eq!(a, b);
    }

    #[test]
    fn different_noun_is_not_equal() {
        let a = Wording::new("Pass", "passes", "es", "thrower");
        let b = Wording::new("Kick", "passes", "es", "thrower");
        assert_ne!(a, b);
    }

    #[test]
    fn different_verb_is_not_equal() {
        let a = Wording::new("Pass", "passes", "es", "thrower");
        let b = Wording::new("Pass", "kicks", "es", "thrower");
        assert_ne!(a, b);
    }

    #[test]
    fn clone_produces_equal_wording() {
        let a = Wording::new("Catch", "catches", "es", "receiver");
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn empty_strings_accepted() {
        let w = Wording::new("", "", "", "");
        assert_eq!(w.get_noun(), "");
        assert_eq!(w.get_verb(), "");
        assert_eq!(w.get_inflection(), "");
        assert_eq!(w.get_player_characterization(), "");
    }
}
