use ffb_model::model::SpecialEffect;
use ffb_model::option::game_option_id;
use ffb_model::util::util_player::UtilPlayer;
use crate::modifiers::armor_modifier::ArmorModifier;
use crate::modifiers::armor_modifiers::ArmorModifiers as ArmorModifiersTrait;
use crate::modifiers::foul_assist_armor_modifier::FoulAssistArmorModifier;
use crate::modifiers::special_effect_armour_modifier::SpecialEffectArmourModifier;
use crate::modifiers::static_armour_modifier::StaticArmourModifier;

/// 1:1 translation of com.fumbbl.ffb.factory.bb2025.ArmorModifiers.
/// BB2025: same as BB2020 but no Bomb at all; set_use_all is a no-op.
pub struct Bb2025ArmorModifiers;

impl ArmorModifiersTrait for Bb2025ArmorModifiers {
    fn get_name(&self) -> &str { "Bb2025ArmorModifiers" }

    fn values(&self) -> Vec<Box<dyn ArmorModifier>> {
        all_modifiers()
    }

    fn all_values(&self) -> Vec<Box<dyn ArmorModifier>> {
        all_modifiers()
    }

    fn set_use_all(&mut self, _use_all: bool) {}
}

fn all_modifiers() -> Vec<Box<dyn ArmorModifier>> {
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
    fn values_has_correct_count() {
        assert_eq!(Bb2025ArmorModifiers.values().len(), 15);
    }

    #[test]
    fn all_values_same_as_values() {
        assert_eq!(Bb2025ArmorModifiers.all_values().len(), Bb2025ArmorModifiers.values().len());
    }

    #[test]
    fn no_bomb_modifier() {
        let names: Vec<String> = Bb2025ArmorModifiers.values().into_iter().map(|m| m.get_name().to_string()).collect::<Vec<String>>();
        assert!(!names.iter().any(|n| n.as_str() == "Bomb"));
    }

    #[test]
    fn has_fireball_and_lightning() {
        let names: Vec<String> = Bb2025ArmorModifiers.values().into_iter().map(|m| m.get_name().to_string()).collect::<Vec<String>>();
        assert!(names.iter().any(|n| n.as_str() == "Fireball"));
        assert!(names.iter().any(|n| n.as_str() == "Lightning"));
    }
}
