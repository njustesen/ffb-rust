use ffb_model::enums::Rules;
use crate::modifiers::bb2016::jump_up_modifier_collection::JumpUpModifierCollection as Bb2016Collection;
use crate::modifiers::jump_up_context::JumpUpContext;
use crate::modifiers::jump_up_modifier::JumpUpModifier;
use crate::modifiers::mixed::jump_up_modifier_collection::JumpUpModifierCollection as MixedCollection;

/// 1:1 translation of com.fumbbl.ffb.factory.JumpUpModifierFactory.
///
/// Java's `GenerifiedModifierFactory.findModifiers` also folds in tacklezone/disturbing-presence
/// effects and skill/card-registered modifiers, but `JumpUpModifierFactory` hardcodes both
/// `isAffectedByTackleZones`/`isAffectedByDisturbingPresence` to `false`, and no skill/card in
/// the Java source registers a `JumpUpModifier` — so `find_modifiers` here reduces to exactly
/// what Java's collapsed logic actually returns: the edition's `JumpUpModifierCollection`,
/// filtered by `appliesToContext` (REGULAR modifiers only, matching the Java collections which
/// register nothing else).
///
/// Java: `mixed/JumpUpModifierCollection` (`@RulesCollection(BB2020, BB2025)`) vs
/// `bb2016/JumpUpModifierCollection` (`@RulesCollection(BB2016)`), selected via reflection
/// `Scanner`; here selected explicitly by `Rules`, matching `DodgeModifierFactory::for_rules`'s
/// convention.
enum Collection {
    Bb2016(Bb2016Collection),
    Mixed(MixedCollection),
}

pub struct JumpUpModifierFactory {
    collection: Collection,
}

impl JumpUpModifierFactory {
    /// BB2016 uses its own collection ("Jump Up" -2); BB2020/BB2025 share the mixed collection
    /// ("Jump Up" -1).
    pub fn for_rules(rules: Rules) -> Self {
        let collection = match rules {
            Rules::Bb2016 => Collection::Bb2016(Bb2016Collection::new()),
            _ => Collection::Mixed(MixedCollection::new()),
        };
        Self { collection }
    }

    /// 1:1 translation of the collapsed `JumpUpModifierFactory.findModifiers` (see doc comment).
    pub fn find_modifiers<'a>(&'a self, context: &JumpUpContext<'_>) -> Vec<&'a JumpUpModifier> {
        match &self.collection {
            Collection::Bb2016(c) => c.find_applicable(context),
            Collection::Mixed(c) => c.find_applicable(context),
        }
    }
}

impl Default for JumpUpModifierFactory {
    fn default() -> Self {
        Self::for_rules(Rules::Bb2025)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::{ActingPlayer, Game, Team};

    fn empty_team(id: &str) -> Team {
        Team {
            id: id.into(), name: id.into(), race: "human".into(),
            roster_id: "human".into(), coach: "c".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0,
            dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![],
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_game(rules: Rules) -> Game {
        Game::new(empty_team("home"), empty_team("away"), rules)
    }

    #[test]
    fn bb2016_selects_bb2016_collection() {
        let game = make_game(Rules::Bb2016);
        let factory = JumpUpModifierFactory::for_rules(Rules::Bb2016);
        let acting = ActingPlayer::default();
        let ctx = JumpUpContext::new(&game, &acting);
        let mods = factory.find_modifiers(&ctx);
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].get_name(), "Jump Up");
        assert_eq!(mods[0].get_modifier(), -2);
    }

    #[test]
    fn bb2020_selects_mixed_collection() {
        let game = make_game(Rules::Bb2020);
        let factory = JumpUpModifierFactory::for_rules(Rules::Bb2020);
        let acting = ActingPlayer::default();
        let ctx = JumpUpContext::new(&game, &acting);
        let mods = factory.find_modifiers(&ctx);
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].get_name(), "Jump Up");
        assert_eq!(mods[0].get_modifier(), -1);
    }

    #[test]
    fn bb2025_selects_mixed_collection() {
        let game = make_game(Rules::Bb2025);
        let factory = JumpUpModifierFactory::for_rules(Rules::Bb2025);
        let acting = ActingPlayer::default();
        let ctx = JumpUpContext::new(&game, &acting);
        let mods = factory.find_modifiers(&ctx);
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].get_modifier(), -1);
    }

    #[test]
    fn default_uses_bb2025_rules() {
        let factory = JumpUpModifierFactory::default();
        let game = make_game(Rules::Bb2025);
        let acting = ActingPlayer::default();
        let ctx = JumpUpContext::new(&game, &acting);
        assert_eq!(factory.find_modifiers(&ctx).len(), 1);
    }
}
