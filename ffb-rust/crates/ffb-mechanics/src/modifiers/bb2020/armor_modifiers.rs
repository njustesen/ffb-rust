use ffb_model::model::SpecialEffect;
use ffb_model::option::game_option_id;
use ffb_model::util::util_player::UtilPlayer;
use crate::modifiers::armor_modifier::ArmorModifier;
use crate::modifiers::armor_modifiers::ArmorModifiers as ArmorModifiersTrait;
use crate::modifiers::foul_assist_armor_modifier::FoulAssistArmorModifier;
use crate::modifiers::special_effect_armour_modifier::SpecialEffectArmourModifier;
use crate::modifiers::static_armour_modifier::StaticArmourModifier;

/// 1:1 translation of com.fumbbl.ffb.factory.bb2020.ArmorModifiers.
/// BB2020: same as BB2016 but Bomb is in legacy_modifiers (only included when use_all=true).
pub struct Bb2020ArmorModifiers {
    use_all: bool,
}

impl Bb2020ArmorModifiers {
    pub fn new() -> Self { Self { use_all: false } }
}

impl Default for Bb2020ArmorModifiers {
    fn default() -> Self { Self::new() }
}

impl ArmorModifiersTrait for Bb2020ArmorModifiers {
    fn get_name(&self) -> &str { "Bb2020ArmorModifiers" }

    fn values(&self) -> Vec<Box<dyn ArmorModifier>> {
        if self.use_all { self.all_values() } else { base_modifiers() }
    }

    fn all_values(&self) -> Vec<Box<dyn ArmorModifier>> {
        let mut v = legacy_modifiers();
        v.extend(base_modifiers());
        v
    }

    fn set_use_all(&mut self, use_all: bool) {
        self.use_all = use_all;
    }
}

fn legacy_modifiers() -> Vec<Box<dyn ArmorModifier>> {
    vec![
        Box::new(SpecialEffectArmourModifier::new("Bomb", 1, false, SpecialEffect::BOMB)),
    ]
}

fn base_modifiers() -> Vec<Box<dyn ArmorModifier>> {
    vec![
        Box::new(FoulAssistArmorModifier::new("1 Offensive Assist", 1, true)),
        Box::new(FoulAssistArmorModifier::new("2 Offensive Assists", 2, true)),
        Box::new(FoulAssistArmorModifier::new("3 Offensive Assists", 3, true)),
        Box::new(FoulAssistArmorModifier::new("4 Offensive Assists", 4, true)),
        Box::new(FoulAssistArmorModifier::new("5 Offensive Assists", 5, true)),
        Box::new(FoulAssistArmorModifier::new("6 Offensive Assists", 6, true)),
        Box::new(FoulAssistArmorModifier::new("7 Offensive Assists", 7, true)),
        Box::new(FoulAssistArmorModifier::new("1 Defensive Assist", -1, true)),
        Box::new(FoulAssistArmorModifier::new("2 Defensive Assists", -2, true)),
        Box::new(FoulAssistArmorModifier::new("3 Defensive Assists", -3, true)),
        Box::new(FoulAssistArmorModifier::new("4 Defensive Assists", -4, true)),
        Box::new(FoulAssistArmorModifier::new("5 Defensive Assists", -5, true)),
        Box::new(
            StaticArmourModifier::new("Foul", 1, false).with_predicate(|ctx| {
                ctx.is_foul()
                    && (ctx.game.options.is_enabled(game_option_id::FOUL_BONUS)
                        || (ctx.game.options.is_enabled(game_option_id::FOUL_BONUS_OUTSIDE_TACKLEZONE)
                            && ctx.attacker.map_or(false, |a| UtilPlayer::find_tacklezones(ctx.game, &a.id) < 1)))
            }),
        ),
        Box::new(SpecialEffectArmourModifier::new("Fireball", 1, false, SpecialEffect::FIREBALL)),
        Box::new(SpecialEffectArmourModifier::new("Lightning", 1, false, SpecialEffect::LIGHTNING)),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_excludes_bomb_by_default() {
        let m = Bb2020ArmorModifiers::new();
        let names: Vec<String> = m.values().into_iter().map(|m| m.get_name().to_string()).collect::<Vec<String>>();
        assert!(!names.iter().any(|n| n.as_str() == "Bomb"));
    }

    #[test]
    fn all_values_includes_bomb() {
        let m = Bb2020ArmorModifiers::new();
        let names: Vec<String> = m.all_values().into_iter().map(|m| m.get_name().to_string()).collect::<Vec<String>>();
        assert!(names.iter().any(|n| n.as_str() == "Bomb"));
    }

    #[test]
    fn set_use_all_includes_bomb_in_values() {
        let mut m = Bb2020ArmorModifiers::new();
        m.set_use_all(true);
        let names: Vec<String> = m.values().into_iter().map(|m| m.get_name().to_string()).collect::<Vec<String>>();
        assert!(names.iter().any(|n| n.as_str() == "Bomb"));
    }

    #[test]
    fn base_count_is_fifteen() {
        assert_eq!(Bb2020ArmorModifiers::new().values().len(), 15);
    }
}
