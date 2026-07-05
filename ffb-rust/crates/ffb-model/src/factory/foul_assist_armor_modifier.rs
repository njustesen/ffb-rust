/// 1:1 translation of com.fumbbl.ffb.factory.FoulAssistArmorModifier.
/// Structural stub — implementation is in ffb-mechanics::modifiers::foul_assist_armor_modifier
/// (ffb-model cannot depend on ffb-mechanics; real impl lives where ArmorModifier trait is).
pub struct FoulAssistArmorModifier;

impl FoulAssistArmorModifier {
    pub fn new() -> Self { Self }
}

impl Default for FoulAssistArmorModifier {
    fn default() -> Self { Self::new() }
}
