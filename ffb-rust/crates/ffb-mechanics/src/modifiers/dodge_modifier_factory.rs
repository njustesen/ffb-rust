use ffb_model::enums::Rules;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::util_player::UtilPlayer;
use crate::modifiers::dodge_context::DodgeContext;
use crate::modifiers::dodge_modifier::DodgeModifier;
use crate::modifiers::dodge_modifier_collection::DodgeModifierCollection;
use crate::modifiers::modifier_type::ModifierType;

/// 1:1 translation of com.fumbbl.ffb.factory.DodgeModifierFactory.
///
/// BB2025 uses the base DodgeModifierCollection (tackle zones only — no prehensile tail).
/// TACKLEZONE selection counts opposing players with tackle zones at the target coordinate.
pub struct DodgeModifierFactory {
    collection: DodgeModifierCollection,
}

impl DodgeModifierFactory {
    /// Construct a factory for the given rules edition.
    /// BB2025 and all editions without an edition-specific subclass use the base collection.
    pub fn for_rules(_rules: Rules) -> Self {
        Self { collection: DodgeModifierCollection::new() }
    }

    /// 1:1 translation of DodgeModifierFactory.forName.
    pub fn for_name(&self, name: &str) -> Option<&DodgeModifier> {
        self.collection.get_modifiers().iter().find(|m| m.get_name() == name)
    }

    /// Returns the modifiers applicable to the given context.
    /// 1:1 translation of GenerifiedModifierFactory.findModifiers + DodgeModifierFactory overrides.
    ///
    /// REGULAR modifiers filtered by predicate (base BB2025 collection has none).
    /// TACKLEZONE: one modifier selected by counting opponents with TZs at target coordinate.
    /// PREHENSILE_TAIL: 0 for BB2025 (no such modifiers in base collection).
    pub fn find_applicable<'a>(&'a self, context: &DodgeContext<'_>) -> Vec<&'a DodgeModifier> {
        let mut result: Vec<&'a DodgeModifier> = self.collection.get_modifiers().iter()
            .filter(|m| m.get_type() == ModifierType::REGULAR && m.applies_to_context(context))
            .collect();

        // Java: isAffectedByTackleZones → !player.hasSkillProperty(ignoreTacklezonesWhenDodging)
        let player = context.acting_player.player_id.as_deref()
            .and_then(|id| context.game.player(id));
        let affected_by_tackle_zones = player
            .map(|p| !p.has_skill_property(NamedProperties::IGNORE_TACKLEZONES_WHEN_DODGING))
            .unwrap_or(true);

        if affected_by_tackle_zones {
            // Java: count opponents with TZs at target coordinate, excluding hasNoTacklezoneForDodging
            let count = self.count_tacklezones(context) as i32;
            if count > 0 {
                if let Some(tz_mod) = self.collection.get_modifiers().iter()
                    .find(|m| m.get_type() == ModifierType::TACKLEZONE && m.get_modifier() == count)
                {
                    result.push(tz_mod);
                }
            }
        }

        result
    }

    /// Count opposing players with tackle zones at the target coordinate.
    /// Java: GenerifiedModifierFactory.numberOfTacklezones with DodgeModifierFactory override.
    fn count_tacklezones(&self, context: &DodgeContext<'_>) -> usize {
        let acting_player_id = match context.acting_player.player_id.as_deref() {
            Some(id) => id,
            None => return 0,
        };
        let other_team = UtilPlayer::find_other_team(context.game, acting_player_id);
        let adjacent = UtilPlayer::find_adjacent_players_with_tacklezones(
            context.game,
            other_team,
            context.target_coordinate,
            false,
        );
        adjacent.iter()
            .filter(|id| {
                context.game.player(id).map(|p| !p.has_skill_property(NamedProperties::HAS_NO_TACKLEZONE_FOR_DODGING)).unwrap_or(false)
            })
            .count()
    }

    /// 1:1 translation of AgilityMechanic.minimumRollDodge (BB2025).
    /// `max(2, agility + sum(modifier))`.
    pub fn minimum_roll(agility: i32, modifiers: &[&DodgeModifier]) -> i32 {
        let total: i32 = modifiers.iter().map(|m| m.get_modifier()).sum();
        (agility + total).max(2)
    }
}

impl Default for DodgeModifierFactory {
    fn default() -> Self {
        Self::for_rules(Rules::Bb2025)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::{ActingPlayer, Game, Team};
    use ffb_model::types::FieldCoordinate;
    use crate::modifiers::dodge_context::DodgeContext;

    fn empty_team(id: &str) -> Team {
        Team {
            id: id.into(), name: id.into(), race: "Human".into(),
            roster_id: "human".into(), coach: "c".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0,
            dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![],
        }
    }

    fn make_game() -> Game {
        Game::new(empty_team("home"), empty_team("away"), Rules::Bb2025)
    }

    #[test]
    fn no_opponents_no_modifiers() {
        let game = make_game();
        let factory = DodgeModifierFactory::for_rules(Rules::Bb2025);
        let acting = ActingPlayer::default();
        let src = FieldCoordinate::new(5, 5);
        let tgt = FieldCoordinate::new(6, 5);
        let ctx = DodgeContext::new(&game, &acting, src, tgt);
        let mods = factory.find_applicable(&ctx);
        assert!(mods.is_empty(), "No opponents should yield no dodge modifiers");
    }

    #[test]
    fn minimum_roll_no_modifiers() {
        // agility 3, no modifiers → max(2, 3) = 3
        assert_eq!(DodgeModifierFactory::minimum_roll(3, &[]), 3);
    }

    #[test]
    fn minimum_roll_with_one_tackle_zone() {
        let m = DodgeModifier::new("1 Tacklezone", 1, ModifierType::TACKLEZONE);
        // agility 3 + 1 = 4
        assert_eq!(DodgeModifierFactory::minimum_roll(3, &[&m]), 4);
    }

    #[test]
    fn minimum_roll_never_below_two() {
        assert_eq!(DodgeModifierFactory::minimum_roll(1, &[]), 2);
    }

    #[test]
    fn for_name_returns_tackle_zone_modifier() {
        let factory = DodgeModifierFactory::for_rules(Rules::Bb2025);
        assert!(factory.for_name("1 Tacklezone").is_some());
        assert!(factory.for_name("8 Tacklezones").is_some());
        assert!(factory.for_name("NonExistent").is_none());
    }
}
