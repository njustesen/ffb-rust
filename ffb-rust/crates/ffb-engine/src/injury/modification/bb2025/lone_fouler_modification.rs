/// Translation of com.fumbbl.ffb.server.injury.modification.bb2025.LoneFoulerModification.
///
/// Extends RerollArmourModification for foul injury types. Extra gate: no offensive or
/// defensive foul assists.
use ffb_model::model::SkillUse;
use ffb_model::util::util_player::UtilPlayer;
use crate::injury::modification::{InjuryContextModification, ModificationParams};
use crate::injury::modification::bb2025::reroll_armour_modification::RerollArmourModification;

pub struct LoneFoulerModification {
    inner: RerollArmourModification,
}

const VALID: &[&str] = &["Foul", "FoulForSpp", "FoulWithChainsaw", "FoulForSppWithChainsaw"];

impl LoneFoulerModification {
    pub fn new() -> Self { Self { inner: RerollArmourModification::with_types(VALID) } }
}

impl Default for LoneFoulerModification {
    fn default() -> Self { Self::new() }
}

impl InjuryContextModification for LoneFoulerModification {
    fn skill_use(&self) -> SkillUse { self.inner.skill_use() }
    fn valid_types(&self) -> &'static [&'static str] { VALID }
    fn skill_id(&self) -> Option<u16> { self.inner.skill_id() }
    fn set_skill_id(&mut self, id: u16) { self.inner.set_skill_id(id); }

    /// Java: no offensive OR defensive assists AND super.tryArmourRollModification (not broken).
    fn try_armour_roll_modification(&self, params: &ModificationParams) -> bool {
        let attacker_id = match params.new_context.attacker_id.as_deref() {
            Some(id) => id,
            None => return false,
        };
        let defender_id = match params.new_context.defender_id.as_deref() {
            Some(id) => id,
            None => return false,
        };
        let offensive = UtilPlayer::find_offensive_foul_assists(params.game, attacker_id, defender_id);
        let defensive = UtilPlayer::find_defensive_foul_assists(params.game, attacker_id, defender_id);
        let no_assists = offensive == 0 && defensive == 0;
        no_assists && self.inner.try_armour_roll_modification(params)
    }

    fn modify_armour_internal(&self, params: &mut ModificationParams) -> bool {
        self.inner.modify_armour_internal(params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_foul_types() {
        let m = LoneFoulerModification::new();
        assert!(m.is_valid_type("Foul"));
        assert!(m.is_valid_type("FoulForSpp"));
        assert!(m.is_valid_type("FoulWithChainsaw"));
        assert!(!m.is_valid_type("Block"));
    }

    #[test]
    fn skill_use_is_reroll_armour() {
        assert_eq!(LoneFoulerModification::new().skill_use(), SkillUse::RE_ROLL_ARMOUR);
    }
}
